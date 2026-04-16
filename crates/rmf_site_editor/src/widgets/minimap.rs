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

//! A small corner minimap that draws a simplified top-down view of the
//! current level: walls as lines, lanes as colored lines, and a camera
//! position indicator.

use crate::site::{Category, CurrentLevel, LevelElevation};
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::relationship::AncestorIter;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use rmf_site_format::{AnchorParams, Edge, LaneMarker, WallMarker};

const MINIMAP_SIZE: f32 = 200.0;
const MINIMAP_MARGIN: f32 = 12.0;

#[derive(Resource, Debug)]
pub struct MinimapDisplay {
    pub show: bool,
}

impl Default for MinimapDisplay {
    fn default() -> Self {
        Self { show: true }
    }
}

#[derive(Default)]
pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MinimapDisplay>()
            .add_systems(Update, render_minimap);
    }
}

fn render_minimap(
    mut contexts: EguiContexts,
    display: Res<MinimapDisplay>,
    current_level: Res<CurrentLevel>,
    levels: Query<Entity, With<LevelElevation>>,
    child_of: Query<&ChildOf>,
    walls: Query<(Entity, &Edge<Entity>), With<WallMarker>>,
    lanes: Query<(Entity, &Edge<Entity>), With<LaneMarker>>,
    anchors: AnchorParams,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
) {
    if !display.show {
        return;
    }
    let Some(level) = current_level.0 else {
        return;
    };
    if levels.get(level).is_err() {
        return;
    }

    // Collect 2D points for walls and lanes on this level.
    let mut wall_segs: Vec<(Vec2, Vec2)> = Vec::new();
    let mut lane_segs: Vec<(Vec2, Vec2)> = Vec::new();
    let mut all_pts: Vec<Vec2> = Vec::new();

    for (wall, edge) in walls.iter() {
        if !AncestorIter::new(&child_of, wall).any(|p| p == level) {
            continue;
        }
        let [a, b] = edge.array();
        let Ok(pa) = anchors.point_in_parent_frame_of(a, Category::General, level) else {
            continue;
        };
        let Ok(pb) = anchors.point_in_parent_frame_of(b, Category::General, level) else {
            continue;
        };
        let a2 = pa.truncate();
        let b2 = pb.truncate();
        wall_segs.push((a2, b2));
        all_pts.push(a2);
        all_pts.push(b2);
    }

    for (lane, edge) in lanes.iter() {
        if !AncestorIter::new(&child_of, lane).any(|p| p == level) {
            continue;
        }
        let [a, b] = edge.array();
        let Ok(pa) = anchors.point_in_parent_frame_of(a, Category::General, level) else {
            continue;
        };
        let Ok(pb) = anchors.point_in_parent_frame_of(b, Category::General, level) else {
            continue;
        };
        let a2 = pa.truncate();
        let b2 = pb.truncate();
        lane_segs.push((a2, b2));
        all_pts.push(a2);
        all_pts.push(b2);
    }

    if all_pts.is_empty() {
        return;
    }

    // Compute bounding box with padding.
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for p in &all_pts {
        min = min.min(*p);
        max = max.max(*p);
    }
    let extent = max - min;
    let pad = extent.max_element() * 0.1;
    min -= Vec2::splat(pad);
    max += Vec2::splat(pad);
    let extent = max - min;

    let scale = if extent.max_element() > 0.0 {
        (MINIMAP_SIZE - 4.0) / extent.max_element()
    } else {
        1.0
    };

    let to_minimap = |p: Vec2| -> egui::Pos2 {
        let rel = p - min;
        egui::pos2(rel.x * scale + 2.0, (extent.y - rel.y) * scale + 2.0)
    };

    let ctx = contexts.ctx_mut();

    egui::Area::new(egui::Id::new("minimap"))
        .anchor(
            egui::Align2::RIGHT_BOTTOM,
            [-MINIMAP_MARGIN, -MINIMAP_MARGIN],
        )
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::default()
                .fill(egui::Color32::from_rgba_unmultiplied(20, 20, 30, 200))
                .corner_radius(6.0)
                .inner_margin(4.0)
                .show(ui, |ui| {
                    let (response, painter) = ui.allocate_painter(
                        egui::vec2(MINIMAP_SIZE, MINIMAP_SIZE),
                        egui::Sense::hover(),
                    );
                    let origin = response.rect.min;

                    // Draw walls (grey).
                    for (a, b) in &wall_segs {
                        let pa = to_minimap(*a);
                        let pb = to_minimap(*b);
                        painter.line_segment(
                            [origin + pa.to_vec2(), origin + pb.to_vec2()],
                            egui::Stroke::new(1.5, egui::Color32::from_gray(160)),
                        );
                    }

                    // Draw lanes (cyan, thinner).
                    for (a, b) in &lane_segs {
                        let pa = to_minimap(*a);
                        let pb = to_minimap(*b);
                        painter.line_segment(
                            [origin + pa.to_vec2(), origin + pb.to_vec2()],
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 180, 220)),
                        );
                    }

                    // Draw camera position indicator.
                    if let Ok(cam_tf) = cameras.single() {
                        let cam_pos = cam_tf.translation();
                        let cam2d = Vec2::new(cam_pos.x, cam_pos.y);
                        let cam_px = to_minimap(cam2d);
                        let center = origin + cam_px.to_vec2();
                        painter.circle_filled(center, 4.0, egui::Color32::from_rgb(255, 80, 80));
                        painter.circle_stroke(
                            center,
                            4.0,
                            egui::Stroke::new(1.0, egui::Color32::WHITE),
                        );
                    }
                });
        });
}
