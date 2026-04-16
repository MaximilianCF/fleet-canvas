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

//! Inspector tile that shows inline metrics for the selected lane: length,
//! directionality, and estimated travel time.

use crate::widgets::{prelude::*, Inspect};
use bevy::prelude::*;
use bevy_egui::egui::Ui;
use rmf_site_egui::WidgetSystem;
use rmf_site_format::{AnchorParams, Category, Edge, LaneMarker, ReverseLane};

/// Default robot travel speed (m/s) used when fleet-specific speed is
/// unavailable.
const DEFAULT_SPEED_MPS: f32 = 0.7;

#[derive(SystemParam)]
pub struct InspectLaneBadges<'w, 's> {
    lanes: Query<'w, 's, (&'static Edge<Entity>, Option<&'static ReverseLane>), With<LaneMarker>>,
    anchors: AnchorParams<'w, 's>,
}

impl<'w, 's> WidgetSystem<Inspect> for InspectLaneBadges<'w, 's> {
    fn show(
        Inspect { selection, .. }: Inspect,
        ui: &mut Ui,
        state: &mut SystemState<Self>,
        world: &mut World,
    ) {
        let params = state.get(world);

        let Ok((edge, reverse)) = params.lanes.get(selection) else {
            return;
        };

        let [a, b] = edge.array();
        let Ok(pa) = params.anchors.point(a, Category::General) else {
            return;
        };
        let Ok(pb) = params.anchors.point(b, Category::General) else {
            return;
        };

        let length = pa.distance(pb);
        let is_bidirectional = !matches!(reverse, Some(ReverseLane::Disable));
        let direction_label = if is_bidirectional {
            "\u{2194} bidirectional"
        } else {
            "\u{2192} one-way"
        };
        let travel_time = length / DEFAULT_SPEED_MPS;

        ui.separator();
        ui.label("Lane Metrics");
        ui.indent("lane_metrics", |ui| {
            ui.label(format!("Length:      {length:.1} m"));
            ui.label(format!("Direction:   {direction_label}"));
            ui.label(format!(
                "Travel time: ~{travel_time:.1} s  (@ {DEFAULT_SPEED_MPS} m/s)"
            ));
        });
    }
}
