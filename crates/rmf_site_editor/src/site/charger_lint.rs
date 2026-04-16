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

//! Charger waypoint validators and bidirectional-lane checks for
//! non-differential robots. These lessons come from real field experience
//! deploying Open-RMF with Dubin/VDA5050 kinematic robots.

use crate::site::NavGraphMarker;
use crate::{Issue, ValidateWorkspace};
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::relationship::AncestorIter;
use bevy::prelude::*;
use rmf_site_format::{
    AssociatedGraphs, DifferentialDrive, Edge, IssueKey, LaneMarker, LocationTags, NameInSite,
    ReverseLane,
};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

/// Charger location has an empty name or duplicates another charger's name.
pub const CHARGER_NAME_ISSUE_UUID: Uuid = Uuid::from_u128(0xc4a3b2d1e0f94a8bb7c6d5e4f3a2b1c0u128);

/// Bidirectional lane used by a robot whose kinematics cannot reverse.
pub const BIDIR_NON_DIFF_ISSUE_UUID: Uuid = Uuid::from_u128(0xd5b4c3e2f1a04b9cc8d7e6f5a4b3c2d1u128);

// ---------------------------------------------------------------------------
// Charger waypoint name validator
// ---------------------------------------------------------------------------

/// Verify that every charger [`Location`] has a unique non-empty name per
/// nav graph. The RMF fleet adapter matches chargers by name between its
/// `config.yaml` and the nav graph; empty or duplicate names break task
/// dispatch silently.
pub fn check_charger_waypoints(
    mut commands: Commands,
    mut validate_events: EventReader<ValidateWorkspace>,
    child_of: Query<&ChildOf>,
    locations: Query<(
        Entity,
        &LocationTags,
        &NameInSite,
        &AssociatedGraphs<Entity>,
    )>,
    graphs: Query<(Entity, Option<&NameInSite>), With<NavGraphMarker>>,
) {
    const HINT_EMPTY: &str = "Set a name matching the 'charger' field in your fleet adapter \
                              config.yaml. Without a name, the fleet adapter cannot dispatch \
                              charge tasks to this waypoint.";
    const HINT_DUP: &str = "Each charger must have a unique name matching config.yaml. \
                            Duplicated names cause ambiguous task dispatch.";

    for root in validate_events.read() {
        let root_entity = **root;

        // Collect charger locations belonging to this workspace.
        let chargers: Vec<(Entity, String, &AssociatedGraphs<Entity>)> = locations
            .iter()
            .filter(|(e, tags, _, _)| {
                AncestorIter::new(&child_of, *e).any(|p| p == root_entity)
                    && tags.0.iter().any(|t| t.is_charger())
            })
            .map(|(e, _, name, graphs)| (e, name.0.clone(), graphs))
            .collect();

        // Empty-name check (graph-independent).
        for (loc, name, _) in &chargers {
            if name.trim().is_empty() {
                let issue = Issue {
                    key: IssueKey {
                        entities: [*loc].into(),
                        kind: CHARGER_NAME_ISSUE_UUID,
                    },
                    brief:
                        "Charger location has no name \u{2014} fleet adapter will fail to find it"
                            .to_string(),
                    hint: HINT_EMPTY.to_string(),
                };
                let id = commands.spawn(issue).id();
                commands.entity(root_entity).add_child(id);
            }
        }

        // Per-graph duplicate-name check.
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
            let mut by_name: BTreeMap<String, BTreeSet<Entity>> = BTreeMap::new();
            for (loc, name, assoc) in &chargers {
                let trimmed = name.trim();
                if trimmed.is_empty() || !assoc.includes(*graph_entity) {
                    continue;
                }
                by_name.entry(trimmed.to_string()).or_default().insert(*loc);
            }
            for (name, entities) in by_name {
                if entities.len() > 1 {
                    let issue = Issue {
                        key: IssueKey {
                            entities,
                            kind: CHARGER_NAME_ISSUE_UUID,
                        },
                        brief: format!("Duplicate charger name '{name}' in graph '{graph_name}'"),
                        hint: HINT_DUP.to_string(),
                    };
                    let id = commands.spawn(issue).id();
                    commands.entity(root_entity).add_child(id);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Bidirectional-lane / non-reversible-robot validator
// ---------------------------------------------------------------------------

/// Flag bidirectional lanes in nav graphs whose robots cannot reverse.
/// A `DifferentialDrive` component with `bidirectional: false` indicates a
/// Dubin/Ackermann/Tugger-like robot that cannot traverse a lane in reverse.
pub fn check_bidirectional_lanes_for_non_diff(
    mut commands: Commands,
    mut validate_events: EventReader<ValidateWorkspace>,
    child_of: Query<&ChildOf>,
    lanes: Query<
        (
            Entity,
            &Edge<Entity>,
            &AssociatedGraphs<Entity>,
            &ReverseLane,
        ),
        With<LaneMarker>,
    >,
    drives: Query<(Entity, &DifferentialDrive, Option<&NameInSite>)>,
) {
    const HINT: &str = "Robots with Dubin/Ackermann/Tugger kinematics cannot reverse. \
                        Set this lane to one-way (ReverseLane::Disable) or create two \
                        parallel one-way lanes for two-way traffic.";

    for root in validate_events.read() {
        let root_entity = **root;

        // Collect non-reversible robot names for reporting.
        let non_reversible: Vec<String> = drives
            .iter()
            .filter(|(e, d, _)| {
                !d.bidirectional && AncestorIter::new(&child_of, *e).any(|p| p == root_entity)
            })
            .map(|(e, _, name)| {
                name.map(|n| n.0.clone())
                    .unwrap_or_else(|| format!("{e:?}"))
            })
            .collect();

        if non_reversible.is_empty() {
            continue;
        }

        let label = if non_reversible.len() == 1 {
            format!("non-reversible robot '{}'", non_reversible[0])
        } else {
            format!("{} non-reversible robots", non_reversible.len())
        };

        for (lane, _edge, _assoc, reverse) in lanes.iter() {
            if !AncestorIter::new(&child_of, lane).any(|p| p == root_entity) {
                continue;
            }
            if matches!(reverse, ReverseLane::Disable) {
                continue;
            }
            // Bidirectional lane in a site that contains a non-reversible robot.
            let issue = Issue {
                key: IssueKey {
                    entities: [lane].into(),
                    kind: BIDIR_NON_DIFF_ISSUE_UUID,
                },
                brief: format!("Bidirectional lane in site used by {label}"),
                hint: HINT.to_string(),
            };
            let id = commands.spawn(issue).id();
            commands.entity(root_entity).add_child(id);
        }
    }
}
