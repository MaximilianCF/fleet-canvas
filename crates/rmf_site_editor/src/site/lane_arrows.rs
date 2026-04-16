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

//! Lane direction arrows rendered as Bevy gizmos while Graph View is
//! active. Bidirectional lanes get two opposing arrows at 40 % and 60 %
//! of the lane length; one-way lanes get a single arrow at the midpoint
//! pointing in the forward direction.

use crate::site::{Category, LevelElevation, NavGraphViewMode};
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::relationship::AncestorIter;
use bevy::prelude::*;
use rmf_site_format::{AnchorParams, Edge, LaneMarker, ReverseLane};

const ARROW_Z: f32 = 0.06;
const ARROW_HALF_LEN: f32 = 0.18;
const ARROW_WIDTH: f32 = 0.10;
const ARROW_COLOR: Color = Color::srgb(0.95, 0.95, 0.40);

/// Run condition: only draw arrows while Graph View is active.
pub fn graph_view_active(mode: Res<NavGraphViewMode>) -> bool {
    mode.active
}

/// Draw a single arrowhead at `mid` pointing along `dir` (unit vector).
fn draw_arrow(gizmos: &mut Gizmos, mid: Vec2, dir: Vec2) {
    let perp = Vec2::new(-dir.y, dir.x);
    let tip = mid + dir * ARROW_HALF_LEN;
    let left = mid - dir * ARROW_HALF_LEN * 0.5 + perp * ARROW_WIDTH;
    let right = mid - dir * ARROW_HALF_LEN * 0.5 - perp * ARROW_WIDTH;
    let p = |v: Vec2| Vec3::new(v.x, v.y, ARROW_Z);
    gizmos.line(p(tip), p(left), ARROW_COLOR);
    gizmos.line(p(tip), p(right), ARROW_COLOR);
}

/// For every visible lane on a known level, draw direction arrows
/// reflecting its [`ReverseLane`] state. Skips lanes that cross levels.
pub fn draw_lane_direction_arrows(
    child_of: Query<&ChildOf>,
    levels: Query<Entity, With<LevelElevation>>,
    lanes: Query<(Entity, &Edge<Entity>, &ReverseLane), With<LaneMarker>>,
    anchors: AnchorParams,
    mut gizmos: Gizmos,
) {
    for (lane, edge, reverse) in lanes.iter() {
        let [a, b] = edge.array();
        let Some(level) = AncestorIter::new(&child_of, a).find(|p| levels.get(*p).is_ok()) else {
            continue;
        };
        let b_level = AncestorIter::new(&child_of, b).find(|p| levels.get(*p).is_ok());
        if b_level != Some(level) {
            continue;
        }
        let Ok(pa) = anchors.point_in_parent_frame_of(a, Category::General, level) else {
            continue;
        };
        let Ok(pb) = anchors.point_in_parent_frame_of(b, Category::General, level) else {
            continue;
        };
        let p_a = pa.truncate();
        let p_b = pb.truncate();
        let delta = p_b - p_a;
        let len = delta.length();
        if len < 1e-3 {
            continue;
        }
        let dir = delta / len;
        let _ = lane; // arrow colour is uniform; lane entity kept for future per-lane coloring

        match reverse {
            ReverseLane::Disable => {
                // One-way: single forward arrow at midpoint.
                let mid = p_a + delta * 0.5;
                draw_arrow(&mut gizmos, mid, dir);
            }
            _ => {
                // Bidirectional: two arrows at 40 % and 60 %, opposing.
                let arrow_a = p_a + delta * 0.4;
                let arrow_b = p_a + delta * 0.6;
                draw_arrow(&mut gizmos, arrow_a, -dir);
                draw_arrow(&mut gizmos, arrow_b, dir);
            }
        }
    }
}
