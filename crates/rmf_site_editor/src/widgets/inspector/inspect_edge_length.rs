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

//! Inspector tile showing the Euclidean length of a selected edge entity
//! (wall or measurement). Lanes are covered by `inspect_lane_badges.rs`.

use crate::widgets::{prelude::*, Inspect};
use bevy::prelude::*;
use bevy_egui::egui::Ui;
use rmf_site_egui::WidgetSystem;
use rmf_site_format::{AnchorParams, Category, Edge};

#[derive(SystemParam)]
pub struct InspectEdgeLength<'w, 's> {
    edges: Query<'w, 's, &'static Edge<Entity>>,
    anchors: AnchorParams<'w, 's>,
}

impl<'w, 's> WidgetSystem<Inspect> for InspectEdgeLength<'w, 's> {
    fn show(
        Inspect { selection, .. }: Inspect,
        ui: &mut Ui,
        state: &mut SystemState<Self>,
        world: &mut World,
    ) {
        let params = state.get(world);
        let Ok(edge) = params.edges.get(selection) else {
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
        ui.label(format!("Length: {length:.2} m"));
    }
}
