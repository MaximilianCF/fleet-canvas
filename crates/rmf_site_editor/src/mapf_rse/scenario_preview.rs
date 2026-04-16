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

//! Scenario preview: bottom-panel scrubber, lane usage heatmap, and lane
//! conflict flash overlay. All three features read from the existing
//! `MAPFDebugInfo::Success` trajectory data produced by the negotiation
//! planner, and render via immediate-mode Bevy gizmos so no lane material
//! state is touched.

use crate::mapf_rse::{DebuggerSettings, MAPFDebugInfo, NegotiationDebugData};
use crate::site::{Category, LevelElevation};
use crate::CurrentWorkspace;
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::relationship::AncestorIter;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use rmf_site_format::{AnchorParams, Edge, LaneMarker, NameOfSite};
use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct ScenarioPreviewPlugin;

impl Plugin for ScenarioPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LaneUsageMap>()
            .init_resource::<LaneConflictSet>()
            .init_resource::<ScenarioPreviewMode>()
            .add_systems(
                Update,
                (
                    recompute_scenario_analysis.run_if(plan_changed),
                    draw_heatmap_gizmos,
                    draw_conflict_gizmos,
                    scenario_scrubber_panel,
                ),
            );
    }
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Usage count per lane entity, accumulated across every agent trajectory in
/// the current successful MAPF solution. `max_count` is the largest value in
/// `counts` and is used to normalize the heatmap color ramp.
#[derive(Resource, Default, Debug)]
pub struct LaneUsageMap {
    pub counts: HashMap<Entity, u32>,
    pub max_count: u32,
}

impl LaneUsageMap {
    fn clear(&mut self) {
        self.counts.clear();
        self.max_count = 0;
    }

    fn intensity(&self, lane: Entity) -> f32 {
        if self.max_count == 0 {
            return 0.0;
        }
        let c = *self.counts.get(&lane).unwrap_or(&0) as f32;
        (c / self.max_count as f32).clamp(0.0, 1.0)
    }
}

/// Lanes where two agents cross within both a spatial and temporal threshold.
/// Displayed as a pulsing red outline by the conflict gizmo system.
#[derive(Resource, Default, Debug)]
pub struct LaneConflictSet {
    pub lanes: HashSet<Entity>,
}

/// User-facing toggles. `show_panel` controls the bottom scrubber widget;
/// the two gizmo overlays are gated by `heatmap` and `conflict`. Defaults
/// mean: panel visible, heatmap on, conflict preview on, so a plan is
/// immediately informative without hunting through menus.
#[derive(Resource, Debug)]
pub struct ScenarioPreviewMode {
    pub show_panel: bool,
    pub heatmap: bool,
    pub conflict: bool,
}

impl Default for ScenarioPreviewMode {
    fn default() -> Self {
        Self {
            show_panel: true,
            heatmap: true,
            conflict: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Analysis: sample trajectories into per-lane usage and conflict sets
// ---------------------------------------------------------------------------

/// Spatial threshold (meters) used when projecting a trajectory waypoint onto
/// the nearest lane. A waypoint farther than this from any lane is ignored
/// for both heatmap and conflict analysis.
const PROJECTION_DISTANCE_M: f32 = 1.5;

/// Two agents whose waypoints are within this time window of each other are
/// considered concurrent for conflict analysis.
const CONFLICT_TIME_WINDOW_S: f32 = 2.0;

/// Two concurrent waypoints closer than this distance are flagged as a
/// conflict on their shared nearest lane.
const CONFLICT_DISTANCE_M: f32 = 1.0;

/// Run condition: recompute only when the MAPF plan actually changes.
fn plan_changed(changed: Query<(), Changed<MAPFDebugInfo>>) -> bool {
    !changed.is_empty()
}

/// For every lane on the current workspace, return the 2D segment
/// `(start, end)` in the level's local frame, bucketed by the level entity.
/// Same pattern used by the nav_graph_lint clearance check.
fn collect_lane_segments(
    root: Entity,
    child_of: &Query<&ChildOf>,
    levels: &Query<Entity, With<LevelElevation>>,
    lanes: &Query<(Entity, &Edge<Entity>), With<LaneMarker>>,
    anchors: &AnchorParams,
) -> HashMap<Entity, Vec<(Entity, Vec2, Vec2)>> {
    let mut out: HashMap<Entity, Vec<(Entity, Vec2, Vec2)>> = HashMap::new();
    for (lane, edge) in lanes.iter() {
        let [a, b] = edge.array();
        let Some(level) = AncestorIter::new(child_of, a).find(|p| levels.get(*p).is_ok()) else {
            continue;
        };
        let b_level = AncestorIter::new(child_of, b).find(|p| levels.get(*p).is_ok());
        if b_level != Some(level) {
            continue;
        }
        if !AncestorIter::new(child_of, level).any(|p| p == root) {
            continue;
        }
        let Ok(pa) = anchors.point_in_parent_frame_of(a, Category::General, level) else {
            continue;
        };
        let Ok(pb) = anchors.point_in_parent_frame_of(b, Category::General, level) else {
            continue;
        };
        out.entry(level)
            .or_default()
            .push((lane, pa.truncate(), pb.truncate()));
    }
    out
}

/// 2D point-to-segment squared distance.
fn point_segment_dist_sq(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_squared();
    if len_sq <= f32::EPSILON {
        return (p - a).length_squared();
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    let proj = a + ab * t;
    (p - proj).length_squared()
}

/// Return the lane entity whose segment is nearest to `p`, as long as the
/// distance is within `PROJECTION_DISTANCE_M`. Ignores lanes on other levels.
fn nearest_lane(p: Vec2, lanes: &[(Entity, Vec2, Vec2)]) -> Option<Entity> {
    let threshold_sq = PROJECTION_DISTANCE_M * PROJECTION_DISTANCE_M;
    let mut best: Option<(Entity, f32)> = None;
    for (lane, a, b) in lanes {
        let d = point_segment_dist_sq(p, *a, *b);
        if d <= threshold_sq {
            match best {
                None => best = Some((*lane, d)),
                Some((_, cur)) if d < cur => best = Some((*lane, d)),
                _ => {}
            }
        }
    }
    best.map(|(e, _)| e)
}

/// Rebuild `LaneUsageMap` and `LaneConflictSet` from the current successful
/// MAPF plan. Runs whenever `MAPFDebugInfo` changes.
fn recompute_scenario_analysis(
    mut usage: ResMut<LaneUsageMap>,
    mut conflicts: ResMut<LaneConflictSet>,
    open_sites: Query<Entity, With<NameOfSite>>,
    current_workspace: Res<CurrentWorkspace>,
    mapf_info: Query<&MAPFDebugInfo>,
    child_of: Query<&ChildOf>,
    levels: Query<Entity, With<LevelElevation>>,
    lanes: Query<(Entity, &Edge<Entity>), With<LaneMarker>>,
    anchors: AnchorParams,
) {
    usage.clear();
    conflicts.lanes.clear();

    let Some(site) = current_workspace.to_site(&open_sites) else {
        return;
    };
    let Ok(info) = mapf_info.get(site) else {
        return;
    };
    let MAPFDebugInfo::Success { solution, .. } = info else {
        return;
    };

    let lane_segments = collect_lane_segments(site, &child_of, &levels, &lanes, &anchors);
    // Flatten all per-level lane lists into one big list. The robots in a
    // MAPF plan share an occupancy grid for a single level, so for now we do
    // not disambiguate by level during projection — the closest lane wins.
    let flat: Vec<(Entity, Vec2, Vec2)> = lane_segments
        .into_values()
        .flatten()
        .collect();
    if flat.is_empty() {
        return;
    }

    // Collect per-agent samples: Vec<(lane, time_seconds, position)>.
    // We project each trajectory waypoint onto the nearest lane.
    let mut agent_samples: Vec<Vec<(Entity, f32, Vec2)>> = Vec::new();
    for proposal in solution.proposals.iter() {
        let mut samples: Vec<(Entity, f32, Vec2)> = Vec::new();
        for wp in proposal.1.meta.trajectory.iter() {
            let x = wp.position.translation.x as f32;
            let y = wp.position.translation.y as f32;
            let p = Vec2::new(x, y);
            let Some(lane) = nearest_lane(p, &flat) else {
                continue;
            };
            let t = wp.time.duration_from_zero().as_secs_f32();
            samples.push((lane, t, p));
            *usage.counts.entry(lane).or_insert(0) += 1;
        }
        agent_samples.push(samples);
    }
    usage.max_count = usage.counts.values().copied().max().unwrap_or(0);

    // Conflict detection: pairs of agents whose samples land on the same
    // lane with overlapping time windows and near-simultaneous proximity.
    for i in 0..agent_samples.len() {
        for j in (i + 1)..agent_samples.len() {
            for (lane_a, ta, pa) in &agent_samples[i] {
                for (lane_b, tb, pb) in &agent_samples[j] {
                    if lane_a != lane_b {
                        continue;
                    }
                    if (ta - tb).abs() > CONFLICT_TIME_WINDOW_S {
                        continue;
                    }
                    if pa.distance(*pb) > CONFLICT_DISTANCE_M {
                        continue;
                    }
                    conflicts.lanes.insert(*lane_a);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Gizmo rendering
// ---------------------------------------------------------------------------

/// Convert a [0,1] intensity into a blue → red heat color. Linear
/// interpolation in RGB through green, good enough for a diagnostic view.
fn heat_color(t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    // Five-stop gradient: blue → cyan → green → yellow → red.
    let stops = [
        (0.00, Vec3::new(0.10, 0.30, 1.00)),
        (0.25, Vec3::new(0.10, 0.80, 0.90)),
        (0.50, Vec3::new(0.30, 0.90, 0.30)),
        (0.75, Vec3::new(0.95, 0.85, 0.20)),
        (1.00, Vec3::new(0.95, 0.20, 0.15)),
    ];
    for pair in stops.windows(2) {
        let (t0, c0) = pair[0];
        let (t1, c1) = pair[1];
        if t <= t1 {
            let k = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
            let c = c0.lerp(c1, k);
            return Color::srgb(c.x, c.y, c.z);
        }
    }
    Color::srgb(0.95, 0.20, 0.15)
}

fn draw_heatmap_gizmos(
    mode: Res<ScenarioPreviewMode>,
    usage: Res<LaneUsageMap>,
    open_sites: Query<Entity, With<NameOfSite>>,
    current_workspace: Res<CurrentWorkspace>,
    child_of: Query<&ChildOf>,
    levels: Query<Entity, With<LevelElevation>>,
    lanes: Query<(Entity, &Edge<Entity>), With<LaneMarker>>,
    anchors: AnchorParams,
    mut gizmos: Gizmos,
) {
    if !mode.heatmap || usage.max_count == 0 {
        return;
    }
    let Some(site) = current_workspace.to_site(&open_sites) else {
        return;
    };
    let segments = collect_lane_segments(site, &child_of, &levels, &lanes, &anchors);
    for (_level, segs) in segments.iter() {
        for (lane, a, b) in segs {
            let intensity = usage.intensity(*lane);
            if intensity <= 0.0 {
                continue;
            }
            let color = heat_color(intensity);
            // Lift the gizmo slightly above lanes (they draw at z≈0) so it
            // stays visible without z-fighting.
            let z = 0.05;
            let p0 = Vec3::new(a.x, a.y, z);
            let p1 = Vec3::new(b.x, b.y, z);
            gizmos.line(p0, p1, color);
        }
    }
}

fn draw_conflict_gizmos(
    mode: Res<ScenarioPreviewMode>,
    conflicts: Res<LaneConflictSet>,
    time: Res<Time>,
    open_sites: Query<Entity, With<NameOfSite>>,
    current_workspace: Res<CurrentWorkspace>,
    child_of: Query<&ChildOf>,
    levels: Query<Entity, With<LevelElevation>>,
    lanes: Query<(Entity, &Edge<Entity>), With<LaneMarker>>,
    anchors: AnchorParams,
    mut gizmos: Gizmos,
) {
    if !mode.conflict || conflicts.lanes.is_empty() {
        return;
    }
    let Some(site) = current_workspace.to_site(&open_sites) else {
        return;
    };
    // Pulse between dim and bright red once per second.
    let phase = (time.elapsed_secs() * std::f32::consts::TAU).sin() * 0.5 + 0.5;
    let color = Color::srgb(1.0, 0.25 + 0.5 * phase, 0.25 + 0.5 * phase);
    let segments = collect_lane_segments(site, &child_of, &levels, &lanes, &anchors);
    for (_level, segs) in segments.iter() {
        for (lane, a, b) in segs {
            if !conflicts.lanes.contains(lane) {
                continue;
            }
            let z = 0.08;
            let p0 = Vec3::new(a.x, a.y, z);
            let p1 = Vec3::new(b.x, b.y, z);
            gizmos.line(p0, p1, color);
            // A small cross marker at the midpoint to catch the eye even
            // when the lane is off-camera or hidden by a wall texture.
            let mid = (p0 + p1) * 0.5;
            let d = 0.25;
            gizmos.line(
                Vec3::new(mid.x - d, mid.y - d, z),
                Vec3::new(mid.x + d, mid.y + d, z),
                color,
            );
            gizmos.line(
                Vec3::new(mid.x - d, mid.y + d, z),
                Vec3::new(mid.x + d, mid.y - d, z),
                color,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Bottom-panel scrubber widget
// ---------------------------------------------------------------------------

/// Render the bottom scrubber panel. Mirrors the scrubber inside the MAPF
/// Debug Panel but is always visible whenever a successful plan exists, and
/// lives at the bottom of the screen where timelines belong.
fn scenario_scrubber_panel(
    mut contexts: EguiContexts,
    mut mode: ResMut<ScenarioPreviewMode>,
    mut debug_data: ResMut<NegotiationDebugData>,
    mut debugger_settings: ResMut<DebuggerSettings>,
    usage: Res<LaneUsageMap>,
    conflicts: Res<LaneConflictSet>,
    open_sites: Query<Entity, With<NameOfSite>>,
    current_workspace: Res<CurrentWorkspace>,
    mapf_info: Query<&MAPFDebugInfo>,
) {
    if !mode.show_panel {
        return;
    }
    let Some(site) = current_workspace.to_site(&open_sites) else {
        return;
    };
    let Ok(info) = mapf_info.get(site) else {
        return;
    };
    let MAPFDebugInfo::Success {
        longest_plan_duration_s,
        ..
    } = info
    else {
        return;
    };
    let duration = *longest_plan_duration_s;

    let ctx = contexts.ctx_mut();

    egui::TopBottomPanel::bottom("scenario_scrubber_panel")
        .resizable(false)
        .min_height(56.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Scenario preview");
                ui.separator();

                // Play / pause toggle — flips between 0 (paused) and 1x.
                let playing = debugger_settings.playback_speed > 0.0;
                let label = if playing { "⏸ Pause" } else { "▶ Play" };
                if ui.button(label).clicked() {
                    if playing {
                        debugger_settings.playback_speed = 0.0;
                    } else {
                        debugger_settings.playback_speed = 1.0;
                    }
                }

                if ui.button("⏮").on_hover_text("Rewind to 0").clicked() {
                    debug_data.time = 0.0;
                }

                ui.separator();
                ui.label(format!(
                    "t = {:>5.2} / {:>5.2} s",
                    debug_data.time, duration
                ));

                ui.add(
                    egui::Slider::new(&mut debug_data.time, 0.0..=duration)
                        .show_value(false)
                        .clamping(egui::SliderClamping::Always),
                );

                ui.separator();
                ui.label("Speed:");
                for &s in &[0.5_f32, 1.0, 2.0, 4.0] {
                    let selected = (debugger_settings.playback_speed - s).abs() < 1e-3;
                    if ui.selectable_label(selected, format!("{s}×")).clicked() {
                        debugger_settings.playback_speed = s;
                    }
                }

                ui.separator();
                ui.checkbox(&mut mode.heatmap, "Heatmap");
                ui.checkbox(&mut mode.conflict, "Conflicts");
            });

            ui.horizontal(|ui| {
                ui.label(format!("Lanes used: {}", usage.counts.len()));
                ui.separator();
                ui.label(format!("Peak traversals: {}", usage.max_count));
                ui.separator();
                let conflict_text = if conflicts.lanes.is_empty() {
                    "Conflicts: none".to_string()
                } else {
                    format!("Conflicts: {} lane(s)", conflicts.lanes.len())
                };
                ui.label(conflict_text);
            });
        });
}
