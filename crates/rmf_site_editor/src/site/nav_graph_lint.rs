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

//! Nav graph validation: connectivity linter and clearance check.
//!
//! These validators run in response to a [`ValidateWorkspace`] event and emit
//! [`Issue`] entities that the diagnostics panel surfaces like any other issue.

use crate::site::{Category, LevelElevation, NavGraphMarker};
use crate::{Issue, ValidateWorkspace};
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::relationship::AncestorIter;
use bevy::prelude::*;
use rmf_site_format::{
    AnchorParams, AssociatedGraphs, Edge, IssueKey, LaneMarker, LocationTags, NameInSite, Point,
    WallMarker,
};
use std::collections::{BTreeSet, HashMap};
use uuid::Uuid;

/// Lanes that form a disconnected component inside a single nav graph.
pub const DISCONNECTED_NAV_GRAPH_ISSUE_UUID: Uuid =
    Uuid::from_u128(0x6f4a2d1c7b3e4f5aa8e9c1d2b3e4f50au128);

/// A [`Location`] whose anchor is not an endpoint of any lane in its nav graph.
pub const ISOLATED_LOCATION_ISSUE_UUID: Uuid =
    Uuid::from_u128(0x9a2b7e1d4c3f48e2b1a6d5f4c3e21d7bu128);

/// A lane segment whose corridor is narrower than the safe clearance from walls.
pub const LANE_CLEARANCE_ISSUE_UUID: Uuid =
    Uuid::from_u128(0x3d7e5a9c2b1f44e1a9c8b7d6f5e4c3d2u128);

/// Minimum safe distance in meters from any lane segment to a wall segment.
/// Derived from a typical RMF delivery robot footprint (~0.5 m diameter) plus
/// a small safety margin. Made a constant for now; a future improvement is to
/// read this per-fleet from `robot_properties`.
pub const DEFAULT_LANE_CLEARANCE_M: f32 = 0.35;

// ---------------------------------------------------------------------------
// Connectivity linter
// ---------------------------------------------------------------------------

/// Simple union-find over entities using path compression.
#[derive(Default)]
struct UnionFind {
    parent: HashMap<Entity, Entity>,
}

impl UnionFind {
    fn find(&mut self, e: Entity) -> Entity {
        let p = *self.parent.entry(e).or_insert(e);
        if p == e {
            return e;
        }
        let root = self.find(p);
        self.parent.insert(e, root);
        root
    }

    fn union(&mut self, a: Entity, b: Entity) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra != rb {
            self.parent.insert(ra, rb);
        }
    }
}

/// For every nav graph in the workspace, build an undirected graph from its
/// associated lanes (endpoints = anchor entities, edges = lanes). If the
/// resulting graph has more than one connected component, emit one issue
/// per non-largest component listing the offending lanes.
///
/// Also emits an issue for each [`Location`] that is associated with a nav
/// graph but whose anchor is not touched by any lane in that graph — these
/// are "floating" locations that the fleet adapter will never be able to
/// route to or from.
pub fn check_for_disconnected_nav_graph_components(
    mut commands: Commands,
    mut validate_events: EventReader<ValidateWorkspace>,
    child_of: Query<&ChildOf>,
    graphs: Query<(Entity, Option<&NameInSite>), With<NavGraphMarker>>,
    lanes: Query<
        (Entity, &Edge<Entity>, &AssociatedGraphs<Entity>),
        With<LaneMarker>,
    >,
    locations: Query<(Entity, &Point<Entity>, &AssociatedGraphs<Entity>), With<LocationTags>>,
) {
    const HINT_DISCONNECTED: &str =
        "The RMF fleet adapter cannot route between disconnected lane components. \
         Either connect these lanes to the main graph with an additional lane, or \
         move them to their own nav graph.";
    const HINT_ISOLATED_LOCATION: &str =
        "This location is associated with a nav graph but no lane in that graph \
         touches its anchor. The fleet adapter will not be able to dispatch tasks \
         to it. Connect a lane to this location's anchor.";

    for root in validate_events.read() {
        let root_entity = **root;
        // Collect nav graphs that belong to this workspace.
        let workspace_graphs: Vec<(Entity, String)> = graphs
            .iter()
            .filter(|(e, _)| AncestorIter::new(&child_of, *e).any(|p| p == root_entity))
            .map(|(e, name)| {
                let label = name.map(|n| n.0.clone()).unwrap_or_else(|| format!("{e:?}"));
                (e, label)
            })
            .collect();

        for (graph_entity, graph_name) in &workspace_graphs {
            // Gather all lanes in this workspace that include this graph.
            let graph_lanes: Vec<(Entity, Entity, Entity)> = lanes
                .iter()
                .filter(|(lane, _, associated)| {
                    AncestorIter::new(&child_of, *lane).any(|p| p == root_entity)
                        && associated.includes(*graph_entity)
                })
                .map(|(lane, edge, _)| {
                    let a = edge.array();
                    (lane, a[0], a[1])
                })
                .collect();

            if graph_lanes.len() < 2 {
                continue;
            }

            let mut uf = UnionFind::default();
            for (_, a, b) in &graph_lanes {
                uf.union(*a, *b);
            }

            // Group lanes by their root component.
            let mut components: HashMap<Entity, Vec<Entity>> = HashMap::new();
            for (lane, a, _) in &graph_lanes {
                let root_anchor = uf.find(*a);
                components.entry(root_anchor).or_default().push(*lane);
            }

            if components.len() <= 1 {
                continue;
            }

            // Identify the largest component and flag every other one.
            let largest = components
                .values()
                .map(|v| v.len())
                .max()
                .unwrap_or(0);
            let mut largest_skipped = false;
            for (_, lane_list) in components.iter() {
                if !largest_skipped && lane_list.len() == largest {
                    largest_skipped = true;
                    continue;
                }
                let entities: BTreeSet<Entity> = lane_list.iter().copied().collect();
                let issue = Issue {
                    key: IssueKey {
                        entities,
                        kind: DISCONNECTED_NAV_GRAPH_ISSUE_UUID,
                    },
                    brief: format!(
                        "Nav graph '{}' has an isolated component of {} lane(s)",
                        graph_name,
                        lane_list.len()
                    ),
                    hint: HINT_DISCONNECTED.to_string(),
                };
                let id = commands.spawn(issue).id();
                commands.entity(root_entity).add_child(id);
            }
        }

        // Check for locations not reachable by any lane in a graph they belong to.
        // We collect the set of anchors touched by lanes per graph once to avoid
        // re-scanning lanes inside the location loop.
        let mut anchors_in_graph: HashMap<Entity, BTreeSet<Entity>> = HashMap::new();
        for (graph_entity, _) in &workspace_graphs {
            let mut set = BTreeSet::new();
            for (lane, edge, associated) in lanes.iter() {
                if !AncestorIter::new(&child_of, lane).any(|p| p == root_entity) {
                    continue;
                }
                if !associated.includes(*graph_entity) {
                    continue;
                }
                let [a, b] = edge.array();
                set.insert(a);
                set.insert(b);
            }
            anchors_in_graph.insert(*graph_entity, set);
        }

        for (loc_entity, point, associated) in locations.iter() {
            if !AncestorIter::new(&child_of, loc_entity).any(|p| p == root_entity) {
                continue;
            }
            for (graph_entity, graph_name) in &workspace_graphs {
                if !associated.includes(*graph_entity) {
                    continue;
                }
                let Some(set) = anchors_in_graph.get(graph_entity) else {
                    continue;
                };
                if !set.contains(&point.0) {
                    let issue = Issue {
                        key: IssueKey {
                            entities: [loc_entity].into(),
                            kind: ISOLATED_LOCATION_ISSUE_UUID,
                        },
                        brief: format!(
                            "Location is isolated in nav graph '{graph_name}'"
                        ),
                        hint: HINT_ISOLATED_LOCATION.to_string(),
                    };
                    let id = commands.spawn(issue).id();
                    commands.entity(root_entity).add_child(id);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Clearance check
// ---------------------------------------------------------------------------

/// Squared 2D distance from point `p` to segment `[a, b]`.
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

/// Minimum distance between two 2D line segments. Returns 0 if they intersect.
fn segment_segment_distance(a0: Vec2, a1: Vec2, b0: Vec2, b1: Vec2) -> f32 {
    // If the segments intersect properly, distance is zero.
    let d1 = a1 - a0;
    let d2 = b1 - b0;
    let denom = d1.x * d2.y - d1.y * d2.x;
    if denom.abs() > f32::EPSILON {
        let delta = b0 - a0;
        let t = (delta.x * d2.y - delta.y * d2.x) / denom;
        let u = (delta.x * d1.y - delta.y * d1.x) / denom;
        if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
            return 0.0;
        }
    }
    // Otherwise the minimum is at one of the four endpoint-to-segment distances.
    let d = [
        point_segment_dist_sq(a0, b0, b1),
        point_segment_dist_sq(a1, b0, b1),
        point_segment_dist_sq(b0, a0, a1),
        point_segment_dist_sq(b1, a0, a1),
    ];
    d.iter().copied().fold(f32::INFINITY, f32::min).sqrt()
}

/// For each lane, compute the minimum 2D distance to any wall on the same
/// level. If that distance is below [`DEFAULT_LANE_CLEARANCE_M`], emit an
/// issue. Doors are intentionally *not* treated as obstacles (robots are
/// expected to pass through them), but lane segments that pass within the
/// clearance threshold of a solid wall are flagged as unsafe.
pub fn check_lane_clearance_to_walls(
    mut commands: Commands,
    mut validate_events: EventReader<ValidateWorkspace>,
    child_of: Query<&ChildOf>,
    levels: Query<Entity, With<LevelElevation>>,
    anchors: AnchorParams,
    lanes: Query<(Entity, &Edge<Entity>), With<LaneMarker>>,
    walls: Query<(Entity, &Edge<Entity>), With<WallMarker>>,
) {
    const HINT: &str =
        "This lane passes within the safety clearance of a wall. A robot \
         following the nominal centerline may collide or be unable to navigate \
         safely. Move the lane farther from the wall, or suppress this issue \
         if you have confirmed the local geometry permits passage.";

    for root in validate_events.read() {
        let root_entity = **root;

        // Bucket walls by level. Each entry is (wall_entity, start_2d, end_2d).
        let mut walls_by_level: HashMap<Entity, Vec<(Entity, Vec2, Vec2)>> = HashMap::new();
        for (wall, edge) in walls.iter() {
            let Some(level) =
                AncestorIter::new(&child_of, wall).find(|p| levels.get(*p).is_ok())
            else {
                continue;
            };
            if !AncestorIter::new(&child_of, level).any(|p| p == root_entity) {
                continue;
            }
            let [a, b] = edge.array();
            let Ok(pa) = anchors.point_in_parent_frame_of(a, Category::General, level) else {
                continue;
            };
            let Ok(pb) = anchors.point_in_parent_frame_of(b, Category::General, level) else {
                continue;
            };
            walls_by_level
                .entry(level)
                .or_default()
                .push((wall, pa.truncate(), pb.truncate()));
        }

        for (lane, edge) in lanes.iter() {
            let [la, lb] = edge.array();
            // Resolve the lane's level via its start anchor. Lanes that cross
            // levels (lift lanes) are handled specially by RMF; skip them.
            let Some(level) =
                AncestorIter::new(&child_of, la).find(|p| levels.get(*p).is_ok())
            else {
                continue;
            };
            let lb_level = AncestorIter::new(&child_of, lb).find(|p| levels.get(*p).is_ok());
            if lb_level != Some(level) {
                continue;
            }
            if !AncestorIter::new(&child_of, level).any(|p| p == root_entity) {
                continue;
            }
            let Ok(pa) = anchors.point_in_parent_frame_of(la, Category::General, level) else {
                continue;
            };
            let Ok(pb) = anchors.point_in_parent_frame_of(lb, Category::General, level) else {
                continue;
            };
            let la2 = pa.truncate();
            let lb2 = pb.truncate();

            let Some(level_walls) = walls_by_level.get(&level) else {
                continue;
            };

            let mut worst: Option<(Entity, f32)> = None;
            for (wall_entity, wa, wb) in level_walls {
                let d = segment_segment_distance(la2, lb2, *wa, *wb);
                if d < DEFAULT_LANE_CLEARANCE_M {
                    match worst {
                        None => worst = Some((*wall_entity, d)),
                        Some((_, best)) if d < best => worst = Some((*wall_entity, d)),
                        _ => {}
                    }
                }
            }

            if let Some((wall_entity, d)) = worst {
                let issue = Issue {
                    key: IssueKey {
                        entities: [lane, wall_entity].into(),
                        kind: LANE_CLEARANCE_ISSUE_UUID,
                    },
                    brief: format!(
                        "Lane is {d:.2} m from a wall (< {DEFAULT_LANE_CLEARANCE_M:.2} m safety threshold)"
                    ),
                    hint: HINT.to_string(),
                };
                let id = commands.spawn(issue).id();
                commands.entity(root_entity).add_child(id);
            }
        }
    }
}
