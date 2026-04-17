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

//! Real-time curvature feedback while a lane is being drawn. Detects the
//! preview lane entity (carrying [`LaneMarker`] + [`Pending`]), looks up an
//! incoming lane at its start anchor, and renders the implied turning arc
//! as a gizmo + floating egui label. Colour encodes Dubin viability against
//! [`DEFAULT_MIN_TURN_RADIUS_M`].

use crate::interaction::Cursor;
use crate::site::{Category, LevelElevation, Pending};
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::relationship::AncestorIter;
use bevy::prelude::*;
use rmf_site_camera::{active_camera_maybe, ActiveCameraQuery};
use rmf_site_format::{AnchorParams, Edge, LaneMarker};

/// Default minimum turn radius in meters for non-differential robots.
/// Matches a typical delivery robot footprint.
pub const DEFAULT_MIN_TURN_RADIUS_M: f32 = 0.5;

const ARC_Z: f32 = 0.06;
const ARC_SEGMENTS: usize = 12;

#[derive(Clone, Copy)]
struct CurvatureReadout {
    world_pos: Vec3,
    radius: f32,
    color: Color,
}

/// Shared readout between the gizmo and label systems so the label does not
/// have to redo the geometry search.
#[derive(Resource, Default)]
struct CurvatureLabel(Option<CurvatureReadout>);

fn color_for_radius(radius: f32) -> Color {
    let min = DEFAULT_MIN_TURN_RADIUS_M;
    if radius >= 2.0 * min {
        Color::srgb(0.2, 0.9, 0.2)
    } else if radius >= min {
        Color::srgb(1.0, 0.85, 0.0)
    } else {
        Color::srgb(1.0, 0.2, 0.2)
    }
}

/// Resolve an anchor entity to a 2D world position.
///
/// Uses the anchor's own `GlobalTransform` so the result is valid even for
/// the cursor-placement anchor, which is parented to the `CursorFrame`
/// rather than a level (see `cursor.rs`).
fn anchor_xy(anchor: Entity, anchors: &AnchorParams) -> Option<Vec2> {
    anchors
        .point(anchor, Category::General)
        .ok()
        .map(|p| p.truncate())
}

/// Find the level entity (if any) that the given anchor is parented under.
/// Used to ensure the incoming lane's start anchor lives on the same level
/// as the preview start, not for coordinate resolution.
fn anchor_level(
    anchor: Entity,
    child_of: &Query<&ChildOf>,
    levels: &Query<Entity, With<LevelElevation>>,
) -> Option<Entity> {
    AncestorIter::new(child_of, anchor).find(|p| levels.get(*p).is_ok())
}

/// If a lane draw is in progress AND the start anchor already has an incoming
/// lane, compute the implied turn radius and draw an arc gizmo. Writes the
/// result into [`CurvatureLabel`] for the companion label system.
#[allow(clippy::too_many_arguments)]
fn draw_curvature_gizmo(
    preview: Query<&Edge<Entity>, (With<LaneMarker>, With<Pending>)>,
    existing: Query<&Edge<Entity>, (With<LaneMarker>, Without<Pending>)>,
    cursor: Res<Cursor>,
    anchors: AnchorParams,
    child_of: Query<&ChildOf>,
    levels: Query<Entity, With<LevelElevation>>,
    mut gizmos: Gizmos,
    mut label: ResMut<CurvatureLabel>,
) {
    label.0 = None;

    let Ok(edge) = preview.single() else {
        return;
    };
    let start_anchor = edge.left();
    let cursor_anchor = edge.right();

    // Before the first click, left() still points at the cursor placement
    // entity. Nothing meaningful to show yet.
    if start_anchor == cursor.level_anchor_placement {
        return;
    }

    let Some(p_start) = anchor_xy(start_anchor, &anchors) else {
        return;
    };
    let Some(p_cursor) = anchor_xy(cursor_anchor, &anchors) else {
        return;
    };

    // Find any existing lane whose END anchor matches the preview start.
    // Skip the preview itself (it is Pending, filtered above).
    let prev_start_anchor = existing
        .iter()
        .find(|e| e.end() == start_anchor && e.start() != start_anchor)
        .map(|e| e.start());
    let Some(prev_start_anchor) = prev_start_anchor else {
        return;
    };
    let Some(p_prev) = anchor_xy(prev_start_anchor, &anchors) else {
        return;
    };

    // Only draw if both real anchors are on the same level (or both orphaned).
    // The cursor-placement anchor is intentionally excluded from this check
    // because it lives under the CursorFrame, not a level.
    if anchor_level(start_anchor, &child_of, &levels)
        != anchor_level(prev_start_anchor, &child_of, &levels)
    {
        return;
    }

    let v_in_raw = p_start - p_prev;
    let v_out_raw = p_cursor - p_start;
    if v_in_raw.length_squared() < 1e-6 || v_out_raw.length_squared() < 1e-6 {
        return;
    }
    let v_in = v_in_raw.normalize();
    let v_out = v_out_raw.normalize();

    // Deflection angle: 0 when perfectly straight, π for a U-turn.
    let cos_a = v_in.dot(v_out).clamp(-1.0, 1.0);
    let deflection = cos_a.acos();
    if deflection.abs() < 0.01 {
        return;
    }

    let chord = v_in_raw.length().min(v_out_raw.length());
    let radius = chord / (2.0 * (deflection / 2.0).sin().abs());

    let color = color_for_radius(radius);

    // Turn direction: cross(v_in, v_out) sign picks the side the center sits on.
    let sign = if v_in.x * v_out.y - v_in.y * v_out.x > 0.0 {
        1.0_f32
    } else {
        -1.0_f32
    };
    let perp = Vec2::new(-v_in.y, v_in.x) * sign;
    let center = p_start + perp * radius;
    let start_angle = (p_start - center).to_angle();
    let arc_angle = deflection * sign;

    for i in 0..ARC_SEGMENTS {
        let t0 = i as f32 / ARC_SEGMENTS as f32;
        let t1 = (i + 1) as f32 / ARC_SEGMENTS as f32;
        let a0 = start_angle + arc_angle * t0;
        let a1 = start_angle + arc_angle * t1;
        let p0 = center + Vec2::from_angle(a0) * radius;
        let p1 = center + Vec2::from_angle(a1) * radius;
        gizmos.line(
            Vec3::new(p0.x, p0.y, ARC_Z),
            Vec3::new(p1.x, p1.y, ARC_Z),
            color,
        );
    }

    label.0 = Some(CurvatureReadout {
        world_pos: Vec3::new(p_start.x + 0.3, p_start.y + 0.3, ARC_Z),
        radius,
        color,
    });
}

/// Floating label showing the numeric radius near the start anchor.
fn draw_curvature_label(
    label: Res<CurvatureLabel>,
    mut contexts: bevy_egui::EguiContexts,
    cameras: Query<(&Camera, &GlobalTransform)>,
    active_cam: ActiveCameraQuery,
) {
    let Some(readout) = label.0 else {
        return;
    };
    let Ok(cam_entity) = active_camera_maybe(&active_cam) else {
        return;
    };
    let Ok((camera, cam_tf)) = cameras.get(cam_entity) else {
        return;
    };
    let Ok(screen) = camera.world_to_viewport(cam_tf, readout.world_pos) else {
        return;
    };

    let ctx = contexts.ctx_mut();
    let min = DEFAULT_MIN_TURN_RADIUS_M;
    let (text, marker) = if readout.radius >= 2.0 * min {
        (format!("R = {:.2} m  OK", readout.radius), None)
    } else if readout.radius >= min {
        (
            format!("R = {:.2} m  marginal", readout.radius),
            Some(format!("min: {:.2} m", min)),
        )
    } else {
        (
            format!("R = {:.2} m  too tight", readout.radius),
            Some(format!("min: {:.2} m", min)),
        )
    };
    let [r, g, b, _] = readout.color.to_srgba().to_u8_array();
    let egui_color = bevy_egui::egui::Color32::from_rgb(r, g, b);

    bevy_egui::egui::Area::new(bevy_egui::egui::Id::new("curvature_label"))
        .fixed_pos(bevy_egui::egui::pos2(screen.x, screen.y))
        .interactable(false)
        .show(ctx, |ui| {
            bevy_egui::egui::Frame::popup(ui.style())
                .fill(bevy_egui::egui::Color32::from_rgba_premultiplied(
                    20, 20, 25, 210,
                ))
                .show(ui, |ui| {
                    ui.label(
                        bevy_egui::egui::RichText::new(text)
                            .color(egui_color)
                            .strong(),
                    );
                    if let Some(m) = marker {
                        ui.label(
                            bevy_egui::egui::RichText::new(m)
                                .color(bevy_egui::egui::Color32::LIGHT_GRAY)
                                .small(),
                        );
                    }
                });
        });
}

pub struct CurvatureGizmoPlugin;

impl Plugin for CurvatureGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurvatureLabel>()
            .add_systems(Update, (draw_curvature_gizmo, draw_curvature_label).chain());
    }
}
