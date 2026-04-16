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

//! Door/lift reachability validator.
//!
//! After the connectivity linter flags disconnected components, this validator
//! goes one step further: it checks whether a door or lift exists that
//! *could* bridge two disconnected components but lacks the lane connections
//! to do so. This gives the user an actionable "why" instead of just "what."

use crate::site::{Category, LevelElevation, NavGraphMarker};
use crate::{Issue, ValidateWorkspace};
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::relationship::AncestorIter;
use bevy::prelude::*;
use rmf_site_format::{
    AnchorParams, AssociatedGraphs, DoorMarker, Edge, IssueKey, LaneMarker, LocationTags,
    NameInSite, Point,
};
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use uuid::Uuid;

/// A door whose anchors touch two disconnected lane components — the likely
/// cause of the disconnection.
pub const DOOR_LIFT_REACHABILITY_ISSUE_UUID: Uuid =
    Uuid::from_u128(0x7f3a1b2c4d5e6f708192a3b4c5d6e7f8u128);

/// Spatial threshold: a door anchor within this distance of a lane anchor
/// is considered "near" that component. Doors and lanes may not share the
/// exact same anchor entity, so we fall back to spatial proximity.
const DOOR_PROXIMITY_M: f32 = 0.5;

/// For each nav graph, identify disconnected lane components (reusing a
/// simple BFS on shared anchor endpoints). Then for every door on the same
/// level, check if it bridges two different components — meaning the user
/// probably forgot to draw a lane through the door. Emits one issue per
/// such door.
pub fn check_door_lift_reachability(
    mut commands: Commands,
    mut validate_events: EventReader<ValidateWorkspace>,
    child_of: Query<&ChildOf>,
    graphs: Query<(Entity, Option<&NameInSite>), With<NavGraphMarker>>,
    lanes: Query<(Entity, &Edge<Entity>, &AssociatedGraphs<Entity>), With<LaneMarker>>,
    doors: Query<(Entity, &Edge<Entity>, Option<&NameInSite>), With<DoorMarker>>,
    levels: Query<Entity, With<LevelElevation>>,
    anchors: AnchorParams,
    locations: Query<(Entity, &Point<Entity>, &AssociatedGraphs<Entity>), With<LocationTags>>,
) {
    const HINT: &str = "A door exists between two disconnected parts of this nav graph \
                        but no lane passes through it. Add a lane whose endpoints touch \
                        both sides of the door to connect the graph.";

    for root in validate_events.read() {
        let root_entity = **root;

        let workspace_graphs: Vec<(Entity, String)> = graphs
            .iter()
            .filter(|(e, _)| AncestorIter::new(&child_of, *e).any(|p| p == root_entity))
            .map(|(e, name)| {
                let label = name
                    .map(|n| n.0.clone())
                    .unwrap_or_else(|| format!("{e:?}"));
                (e, label)
            })
            .collect();

        for (graph_entity, graph_name) in &workspace_graphs {
            // Collect lane anchor endpoints for this graph, grouped by level.
            let mut lane_anchors_by_level: HashMap<Entity, Vec<(Entity, Entity, Entity)>> =
                HashMap::new();
            for (lane, edge, associated) in lanes.iter() {
                if !AncestorIter::new(&child_of, lane).any(|p| p == root_entity) {
                    continue;
                }
                if !associated.includes(*graph_entity) {
                    continue;
                }
                let [a, b] = edge.array();
                let Some(level) = AncestorIter::new(&child_of, a).find(|p| levels.get(*p).is_ok())
                else {
                    continue;
                };
                lane_anchors_by_level
                    .entry(level)
                    .or_default()
                    .push((lane, a, b));
            }

            // For each level, build connected components via BFS on anchor adjacency.
            for (level, level_lanes) in &lane_anchors_by_level {
                let mut adj: HashMap<Entity, HashSet<Entity>> = HashMap::new();
                for (_, a, b) in level_lanes {
                    adj.entry(*a).or_default().insert(*b);
                    adj.entry(*b).or_default().insert(*a);
                }

                // BFS to assign component IDs.
                let mut component_of: HashMap<Entity, usize> = HashMap::new();
                let mut comp_id = 0usize;
                for anchor in adj.keys() {
                    if component_of.contains_key(anchor) {
                        continue;
                    }
                    let mut queue = VecDeque::new();
                    queue.push_back(*anchor);
                    component_of.insert(*anchor, comp_id);
                    while let Some(cur) = queue.pop_front() {
                        if let Some(neighbors) = adj.get(&cur) {
                            for n in neighbors {
                                if !component_of.contains_key(n) {
                                    component_of.insert(*n, comp_id);
                                    queue.push_back(*n);
                                }
                            }
                        }
                    }
                    comp_id += 1;
                }

                if comp_id <= 1 {
                    continue; // Single component, nothing to check.
                }

                // Precompute anchor positions for spatial proximity fallback.
                let anchor_positions: HashMap<Entity, Vec2> = adj
                    .keys()
                    .filter_map(|a| {
                        anchors
                            .point_in_parent_frame_of(*a, Category::General, *level)
                            .ok()
                            .map(|p| (*a, p.truncate()))
                    })
                    .collect();

                // Check each door on this level: does it bridge two components?
                for (door_entity, door_edge, door_name) in doors.iter() {
                    if !AncestorIter::new(&child_of, door_entity).any(|p| p == *level) {
                        continue;
                    }
                    let [da, db] = door_edge.array();
                    let comp_a =
                        find_component(da, &component_of, &anchor_positions, &anchors, *level);
                    let comp_b =
                        find_component(db, &component_of, &anchor_positions, &anchors, *level);

                    let (Some(ca), Some(cb)) = (comp_a, comp_b) else {
                        continue;
                    };
                    if ca == cb {
                        continue;
                    }

                    let door_label = door_name
                        .map(|n| n.0.clone())
                        .unwrap_or_else(|| format!("{door_entity:?}"));
                    let issue = Issue {
                        key: IssueKey {
                            entities: [door_entity].into(),
                            kind: DOOR_LIFT_REACHABILITY_ISSUE_UUID,
                        },
                        brief: format!(
                            "Door '{}' bridges disconnected parts of graph '{}' — missing lane through door",
                            door_label, graph_name
                        ),
                        hint: HINT.to_string(),
                    };
                    let id = commands.spawn(issue).id();
                    commands.entity(root_entity).add_child(id);
                }
            }

            // Cross-level check: flag locations that are on a level with no
            // lanes in this graph at all (likely a missing lift cabin lane).
            for (loc, point, associated) in locations.iter() {
                if !AncestorIter::new(&child_of, loc).any(|p| p == root_entity) {
                    continue;
                }
                if !associated.includes(*graph_entity) {
                    continue;
                }
                let Some(level) =
                    AncestorIter::new(&child_of, point.0).find(|p| levels.get(*p).is_ok())
                else {
                    continue;
                };
                if lane_anchors_by_level.contains_key(&level) {
                    continue;
                }
                let loc_name = "location";
                let issue = Issue {
                    key: IssueKey {
                        entities: [loc].into(),
                        kind: DOOR_LIFT_REACHABILITY_ISSUE_UUID,
                    },
                    brief: format!(
                        "A {loc_name} is on a level with no lanes in graph '{graph_name}' — check lift cabin lane tags"
                    ),
                    hint: "This level has a location associated with this nav graph but no \
                           lanes on the same level belong to the graph. If the location is \
                           reachable via a lift, ensure the lift cabin lanes are tagged with \
                           this graph."
                        .to_string(),
                };
                let id = commands.spawn(issue).id();
                commands.entity(root_entity).add_child(id);
            }
        }
    }
}

/// Find which component a door anchor belongs to. First try exact entity
/// match; if the door anchor isn't itself a lane anchor, fall back to
/// spatial proximity.
fn find_component(
    door_anchor: Entity,
    component_of: &HashMap<Entity, usize>,
    anchor_positions: &HashMap<Entity, Vec2>,
    anchors: &AnchorParams,
    level: Entity,
) -> Option<usize> {
    if let Some(c) = component_of.get(&door_anchor) {
        return Some(*c);
    }
    let Ok(door_pos) = anchors.point_in_parent_frame_of(door_anchor, Category::General, level)
    else {
        return None;
    };
    let dp = door_pos.truncate();
    let threshold_sq = DOOR_PROXIMITY_M * DOOR_PROXIMITY_M;
    let mut best: Option<(usize, f32)> = None;
    for (anchor, pos) in anchor_positions {
        let d = dp.distance_squared(*pos);
        if d <= threshold_sq {
            let comp = *component_of.get(anchor)?;
            match best {
                None => best = Some((comp, d)),
                Some((_, bd)) if d < bd => best = Some((comp, d)),
                _ => {}
            }
        }
    }
    best.map(|(c, _)| c)
}
