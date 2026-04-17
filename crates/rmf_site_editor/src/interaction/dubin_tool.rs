/*
 * Copyright (C) 2026 Open Source Robotics Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
*/

#![cfg(not(target_arch = "wasm32"))]

//! Dubin Curve Tool — an interaction mode that generates arc waypoints
//! along the shortest Dubin path between two user-specified poses.
//!
//! Workflow (activate with `B`):
//! 1. Click an existing anchor: locks the departure position.
//! 2. Drag: the drag vector defines the departure heading.
//! 3. Click: locks departure heading, enters arrival mode.
//! 4. Move cursor: live preview of the shortest Dubin path.
//! 5. Click: commits N intermediate anchors wired by one-way lanes.

use crate::interaction::{Cursor, GizmoBlockers, IntersectGroundPlaneParams};
use crate::site::{AnchorBundle, CurrentLevel};
use bevy::prelude::*;
use dubins_paths::{DubinsPath, PosRot};
use rmf_site_format::{Anchor, AssociatedGraphs, Edge, Lane, Motion, ReverseLane};

/// Where the user is in the Dubin tool click-drag-click sequence.
#[derive(Resource, Default, PartialEq)]
pub enum DubinToolState {
    #[default]
    Idle,
    /// First anchor selected, dragging to define departure heading.
    DepartureHeading {
        anchor: Entity,
        pos: Vec2,
        heading: f32,
    },
    /// Departure locked. Moving cursor to select arrival pose.
    ArrivalHeading {
        start_anchor: Entity,
        start_pos: Vec2,
        start_heading: f32,
        end_pos: Vec2,
        end_heading: f32,
    },
}

#[derive(Resource)]
pub struct DubinToolConfig {
    /// Minimum turning radius in meters.
    pub min_radius: f32,
    /// Number of intermediate waypoints along the arc (excluding start/end).
    pub samples: usize,
}

impl Default for DubinToolConfig {
    fn default() -> Self {
        Self {
            min_radius: 0.5,
            samples: 6,
        }
    }
}

#[derive(Event)]
pub struct DubinMaterializeRequest {
    pub start_anchor: Entity,
    pub start_pos: Vec2,
    pub start_heading: f32,
    pub end_pos: Vec2,
    pub end_heading: f32,
}

const ANCHOR_PICK_RADIUS: f32 = 0.3;
const PREVIEW_Z: f32 = 0.07;
const ARROW_Z: f32 = 0.08;

fn cursor_xy(intersect: &IntersectGroundPlaneParams) -> Option<Vec2> {
    intersect
        .ground_plane_intersection()
        .map(|tf| tf.translation.truncate())
}

fn nearest_anchor<'a>(
    p: Vec2,
    anchors: impl Iterator<Item = (Entity, &'a Anchor, &'a GlobalTransform)>,
) -> Option<Entity> {
    let mut best: Option<(Entity, f32)> = None;
    for (e, _anchor, tf) in anchors {
        let ap = tf.translation().truncate();
        let d = ap.distance(p);
        if d <= ANCHOR_PICK_RADIUS && best.is_none_or(|(_, bd)| d < bd) {
            best = Some((e, d));
        }
    }
    best.map(|(e, _)| e)
}

#[allow(clippy::too_many_arguments)]
pub fn update_dubin_tool(
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut contexts: bevy_egui::EguiContexts,
    mut state: ResMut<DubinToolState>,
    mut materialize: EventWriter<DubinMaterializeRequest>,
    intersect: IntersectGroundPlaneParams,
    anchors: Query<(Entity, &Anchor, &GlobalTransform)>,
    active: Res<DubinActive>,
) {
    if !active.0 {
        return;
    }
    if contexts.ctx_mut().wants_pointer_input() {
        return;
    }
    let Some(cursor_p) = cursor_xy(&intersect) else {
        return;
    };
    let clicked = mouse.just_pressed(MouseButton::Left);
    // Consume the click so the standard anchor-selection / lane-draw
    // workflow does not also spawn an edge at the same cursor position.
    if clicked {
        mouse.clear_just_pressed(MouseButton::Left);
    }

    match &*state {
        DubinToolState::Idle => {
            if clicked {
                if let Some(anchor_e) = nearest_anchor(cursor_p, anchors.iter()) {
                    let Ok((_, _, tf)) = anchors.get(anchor_e) else {
                        return;
                    };
                    let pos = tf.translation().truncate();
                    *state = DubinToolState::DepartureHeading {
                        anchor: anchor_e,
                        pos,
                        heading: 0.0,
                    };
                }
            }
        }
        DubinToolState::DepartureHeading { anchor, pos, .. } => {
            let anchor = *anchor;
            let pos = *pos;
            let delta = cursor_p - pos;
            let heading = if delta.length_squared() > 1e-4 {
                delta.y.atan2(delta.x)
            } else {
                0.0
            };
            if clicked {
                *state = DubinToolState::ArrivalHeading {
                    start_anchor: anchor,
                    start_pos: pos,
                    start_heading: heading,
                    end_pos: cursor_p,
                    end_heading: heading,
                };
            } else {
                *state = DubinToolState::DepartureHeading {
                    anchor,
                    pos,
                    heading,
                };
            }
        }
        DubinToolState::ArrivalHeading {
            start_anchor,
            start_pos,
            start_heading,
            end_pos,
            ..
        } => {
            let start_anchor = *start_anchor;
            let start_pos = *start_pos;
            let start_heading = *start_heading;
            let _ = end_pos;
            // Derive arrival heading from (start -> cursor) rather than the
            // per-frame cursor delta. The delta approach collapses to a
            // near-zero vector whenever the user pauses the mouse, producing
            // an unstable atan2 and a flickering preview.
            let to_cursor = cursor_p - start_pos;
            let end_heading = if to_cursor.length_squared() > 1e-4 {
                to_cursor.to_angle()
            } else {
                start_heading
            };
            if clicked {
                materialize.write(DubinMaterializeRequest {
                    start_anchor,
                    start_pos,
                    start_heading,
                    end_pos: cursor_p,
                    end_heading,
                });
                *state = DubinToolState::Idle;
            } else {
                *state = DubinToolState::ArrivalHeading {
                    start_anchor,
                    start_pos,
                    start_heading,
                    end_pos: cursor_p,
                    end_heading,
                };
            }
        }
    }
}

pub fn draw_dubin_preview(
    state: Res<DubinToolState>,
    config: Res<DubinToolConfig>,
    mut gizmos: Gizmos,
) {
    match &*state {
        DubinToolState::Idle => {}
        DubinToolState::DepartureHeading { pos, heading, .. } => {
            let dir = Vec2::new(heading.cos(), heading.sin());
            let tip = *pos + dir * 0.5;
            gizmos.line(
                Vec3::new(pos.x, pos.y, ARROW_Z),
                Vec3::new(tip.x, tip.y, ARROW_Z),
                Color::srgb(0.0, 0.8, 1.0),
            );
        }
        DubinToolState::ArrivalHeading {
            start_pos,
            start_heading,
            end_pos,
            end_heading,
            ..
        } => {
            let q0 = PosRot::from_floats(start_pos.x, start_pos.y, *start_heading);
            let q1 = PosRot::from_floats(end_pos.x, end_pos.y, *end_heading);
            let Ok(path) = DubinsPath::shortest_from(q0, q1, config.min_radius) else {
                return;
            };
            let total = path.length();
            let n = (config.samples * 4).max(16);
            let mut prev: Option<Vec3> = None;
            let preview_color = Color::srgb(0.0, 0.8, 1.0);
            for i in 0..=n {
                let t = total * i as f32 / n as f32;
                let pt = path.sample(t);
                let cur = Vec3::new(pt.x(), pt.y(), PREVIEW_Z);
                if let Some(p) = prev {
                    gizmos.line(p, cur, preview_color);
                }
                prev = Some(cur);
            }
            let arrow_color = Color::srgb(0.0, 1.0, 0.5);
            for i in 0..=config.samples {
                let t = total * i as f32 / config.samples.max(1) as f32;
                let pt = path.sample(t);
                let pos = Vec2::new(pt.x(), pt.y());
                let dir = Vec2::new(pt.rot().cos(), pt.rot().sin());
                let tip = pos + dir * 0.18;
                gizmos.line(
                    Vec3::new(pos.x, pos.y, ARROW_Z),
                    Vec3::new(tip.x, tip.y, ARROW_Z),
                    arrow_color,
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn materialize_dubin_path(
    mut events: EventReader<DubinMaterializeRequest>,
    config: Res<DubinToolConfig>,
    current_level: Res<CurrentLevel>,
    level_tfs: Query<&GlobalTransform>,
    mut commands: Commands,
    mut cursor: ResMut<Cursor>,
    mut visibility: Query<&mut Visibility>,
) {
    for req in events.read() {
        let q0 = PosRot::from_floats(req.start_pos.x, req.start_pos.y, req.start_heading);
        let q1 = PosRot::from_floats(req.end_pos.x, req.end_pos.y, req.end_heading);
        let Ok(path) = DubinsPath::shortest_from(q0, q1, config.min_radius) else {
            warn!("Dubin tool: no path found between selected poses");
            continue;
        };
        let total = path.length();
        let parent_tf = current_level
            .0
            .and_then(|l| level_tfs.get(l).ok().copied())
            .unwrap_or_default();

        let mut chain: Vec<Entity> = Vec::with_capacity(config.samples);
        chain.push(req.start_anchor);
        for i in 1..=config.samples {
            let t = total * i as f32 / (config.samples + 1) as f32;
            let pt = path.sample(t);
            let bundle = AnchorBundle::new(Anchor::Translate2D([pt.x(), pt.y()]))
                .parent_transform(&parent_tf);
            let anchor_e = commands.spawn(bundle).id();
            if let Some(level) = current_level.0 {
                commands.entity(level).add_child(anchor_e);
            }
            chain.push(anchor_e);
        }

        for pair in chain.windows(2) {
            let mut lane: Lane<Entity> = Edge::new(pair[0], pair[1]).into();
            lane.reverse = ReverseLane::Disable;
            lane.forward = Motion::default();
            lane.graphs = AssociatedGraphs::All;
            // Lanes are parented to the site automatically by
            // `assign_orphan_nav_elements_to_site` on the next tick.
            commands.spawn(lane);
        }

        // Remove the tool-active cursor mode; the user can continue editing.
        cursor.remove_mode(DUBIN_TOOL_MODE_LABEL, &mut visibility);
    }
}

const DUBIN_TOOL_MODE_LABEL: &str = "dubin_tool";

/// Tracks whether the tool is armed (`B` pressed). Separate from the state
/// machine so that `Idle` is a valid armed state (between clicks).
#[derive(Resource, Default)]
pub struct DubinActive(pub bool);

#[allow(clippy::too_many_arguments)]
pub fn handle_dubin_activation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: bevy_egui::EguiContexts,
    mut active: ResMut<DubinActive>,
    mut state: ResMut<DubinToolState>,
    mut cursor: ResMut<Cursor>,
    mut visibility: Query<&mut Visibility>,
    mut gizmo_blockers: ResMut<GizmoBlockers>,
) {
    let was_active = active.0;
    if !contexts.ctx_mut().wants_keyboard_input() {
        if keyboard.just_pressed(KeyCode::KeyB) {
            active.0 = !active.0;
            *state = DubinToolState::Idle;
            if active.0 {
                cursor.add_mode(DUBIN_TOOL_MODE_LABEL, &mut visibility);
            } else {
                cursor.remove_mode(DUBIN_TOOL_MODE_LABEL, &mut visibility);
            }
        }
        if active.0 && keyboard.just_pressed(KeyCode::Escape) {
            if *state == DubinToolState::Idle {
                active.0 = false;
                cursor.remove_mode(DUBIN_TOOL_MODE_LABEL, &mut visibility);
            } else {
                *state = DubinToolState::Idle;
            }
        }
    }
    // Mirror DubinActive onto GizmoBlockers so the standard anchor-selection
    // and lane-draw workflows ignore interactions while this tool owns input.
    // Only update on a transition so we don't clobber blockers owned by other
    // tools.
    if active.0 != was_active {
        gizmo_blockers.selecting = active.0;
    }
}

pub fn draw_dubin_settings_panel(
    mut config: ResMut<DubinToolConfig>,
    mut state: ResMut<DubinToolState>,
    mut active: ResMut<DubinActive>,
    mut contexts: bevy_egui::EguiContexts,
) {
    if !active.0 {
        return;
    }
    let ctx = contexts.ctx_mut();
    bevy_egui::egui::Window::new("Dubin Curve Tool [B]")
        .anchor(
            bevy_egui::egui::Align2::RIGHT_TOP,
            bevy_egui::egui::vec2(-10.0, 70.0),
        )
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Min radius (m):");
                ui.add(
                    bevy_egui::egui::DragValue::new(&mut config.min_radius)
                        .speed(0.05)
                        .range(0.1..=5.0)
                        .fixed_decimals(2),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Waypoints:");
                let mut samples = config.samples as i32;
                ui.add(
                    bevy_egui::egui::DragValue::new(&mut samples)
                        .speed(1)
                        .range(2..=20),
                );
                config.samples = samples.max(2) as usize;
            });
            let stage = match *state {
                DubinToolState::Idle => "Click an anchor to begin",
                DubinToolState::DepartureHeading { .. } => {
                    "Drag to set departure heading, then click"
                }
                DubinToolState::ArrivalHeading { .. } => "Move cursor to aim, click to commit",
            };
            ui.separator();
            ui.label(
                bevy_egui::egui::RichText::new(stage)
                    .color(bevy_egui::egui::Color32::from_rgb(80, 200, 255)),
            );
            if ui.button("Cancel (Esc)").clicked() {
                *state = DubinToolState::Idle;
                active.0 = false;
            }
        });
}

pub struct DubinToolPlugin;

impl Plugin for DubinToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DubinToolState>()
            .init_resource::<DubinToolConfig>()
            .init_resource::<DubinActive>()
            .add_event::<DubinMaterializeRequest>()
            .add_systems(
                Update,
                (
                    handle_dubin_activation,
                    update_dubin_tool,
                    draw_dubin_preview,
                    materialize_dubin_path,
                    draw_dubin_settings_panel,
                )
                    .chain(),
            );
    }
}
