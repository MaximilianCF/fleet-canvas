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

//! Paginated feature guide that shows on first run and can be reopened
//! from Help > Feature Guide.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

const TOTAL_PAGES: usize = 7;
const TOUR_SENTINEL: &str = "tour_completed";

#[derive(Resource)]
pub struct FeatureGuideState {
    pub show: bool,
    pub page: usize,
}

impl Default for FeatureGuideState {
    fn default() -> Self {
        Self {
            show: false,
            page: 0,
        }
    }
}

fn tour_sentinel_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("rmf_site_editor").join(TOUR_SENTINEL))
}

fn tour_completed() -> bool {
    tour_sentinel_path().is_some_and(|p| p.exists())
}

fn mark_tour_completed() {
    if let Some(path) = tour_sentinel_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, "done");
    }
}

fn check_first_run(mut state: ResMut<FeatureGuideState>) {
    if !tour_completed() {
        state.show = true;
        state.page = 0;
    }
}

struct Page {
    icon: &'static str,
    title: &'static str,
    body: &'static str,
    tip: &'static str,
}

const PAGES: [Page; TOTAL_PAGES] = [
    Page {
        icon: "\u{1F5FA}",
        title: "Welcome to Fleet Canvas",
        body: "Design and validate robot fleet deployment sites.\n\
               Draw multi-level buildings, navigation graphs, and export\n\
               directly to Gazebo SDF with ROS 2 launch files.",
        tip: "Tip: use the demo map to explore the editor features.",
    },
    Page {
        icon: "\u{1F3D7}",
        title: "Site Editing",
        body: "Draw walls, floors, doors and lifts to define your building.\n\
               Place anchors with snap-to-grid for precise geometry.",
        tip: "[G] snap  [Shift+G] cycle size  [Alt+G] grid overlay\n\
              Shift during wall draw: ortho constraint (0/45/90\u{00B0})",
    },
    Page {
        icon: "\u{1F517}",
        title: "Navigation Graphs",
        body: "Create lanes for robot and human traffic.\n\
               Use Graph View [F4] to isolate and inspect nav elements.\n\
               Lane badges show length, direction and travel-time on selection.",
        tip: "Validate with the Diagnostics panel \u{2014} catches dead-ends,\n\
              clearance issues and unreachable locations automatically.",
    },
    Page {
        icon: "\u{1F3AC}",
        title: "Scenario Preview",
        body: "Load a MAPF plan to visualize robot trajectories.\n\
               The bottom scrubber lets you play, pause and seek through\n\
               the scenario timeline.",
        tip: "Heatmap overlay shows congestion hotspots.\n\
              Red pulse marks lane conflicts between agents.",
    },
    Page {
        icon: "\u{1F4E6}",
        title: "Export & Integration",
        body: "Export to Gazebo SDF with [Ctrl+E] \u{2014} choose which elements\n\
               to include in the options dialog.\n\
               A ROS 2 launch.py is generated automatically alongside the SDF.",
        tip: "Nav graph export is available from the File menu.",
    },
    Page {
        icon: "\u{1F4A1}",
        title: "Tips & Help",
        body: "Help > Feature Guide \u{2014} reopen this guide anytime.\n\
               [M] measurement tool\n\
               View > Presets \u{2014} hide/show element categories instantly.",
        tip: "Search bar in Inspect tab supports #ID jump syntax.\n\
              Auto-backup runs every 2 min \u{2192} ~/.cache/rmf_site_editor/",
    },
    Page {
        icon: "\u{1F916}",
        title: "Non-Differential Robots",
        body: "Robots with Dubin, Ackermann or Tugger kinematics cannot reverse\n\
               or rotate in place. Authoring nav graphs for these robots needs care.\n\
               \n\
               LANES\n\
               \u{2022} Make ALL lanes one-way \u{2014} bidirectional implies reversing.\n\
               \u{2022} For two-way corridors: two parallel one-way lanes.\n\
               \u{2022} Diagnostics flags bidirectional lanes on non-reversible graphs.\n\
               \n\
               CORNERS\n\
               \u{2022} Never connect two straight lanes at 90\u{00B0} directly.\n\
               \u{2022} Add 2\u{2013}3 intermediate waypoints per corner forming an arc.\n\
               \u{2022} Each arc segment should respect the minimum turn radius.\n\
               \n\
               DRAWING\n\
               \u{2022} Place ALL waypoints first, then draw lanes in loop sequence.\n\
               \u{2022} Click origin \u{2192} destination (order defines direction).\n\
               \u{2022} Uncheck 'bidirectional' immediately after each lane is created.\n\
               \u{2022} Never drag waypoints on top of each other \u{2014} use Diagnostics\n\
               \u{2003} Merge button if duplicate anchors are detected.\n\
               \n\
               CHARGER\n\
               \u{2022} The charger location name must match exactly the 'charger'\n\
               \u{2003} field in your fleet adapter config.yaml.\n\
               \u{2022} Diagnostics flags empty or duplicate charger names.",
        tip: "Run Diagnostics after every nav graph change.",
    },
];

fn render_feature_guide(mut contexts: EguiContexts, mut state: ResMut<FeatureGuideState>) {
    if !state.show {
        return;
    }

    let mut open = state.show;
    egui::Window::new("Feature Guide")
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(560.0)
        .show(contexts.ctx_mut(), |ui| {
            let page = &PAGES[state.page.min(TOTAL_PAGES - 1)];

            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                ui.label(egui::RichText::new(page.icon).size(36.0));
                ui.heading(page.title);
            });

            ui.add_space(8.0);
            egui::ScrollArea::vertical()
                .max_height(340.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.label(page.body);
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(page.tip)
                            .small()
                            .color(egui::Color32::from_rgb(100, 200, 180)),
                    );
                });

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(4.0);

            // Page dots.
            ui.horizontal(|ui| {
                for i in 0..TOTAL_PAGES {
                    let dot = if i == state.page {
                        "\u{25CF}"
                    } else {
                        "\u{25CB}"
                    };
                    ui.label(
                        egui::RichText::new(dot)
                            .size(10.0)
                            .color(egui::Color32::from_rgb(140, 140, 160)),
                    );
                }
            });

            ui.horizontal(|ui| {
                if state.page > 0 && ui.button("\u{2190} Back").clicked() {
                    state.page -= 1;
                }

                if state.page < TOTAL_PAGES - 1 {
                    if ui.button("Next \u{2192}").clicked() {
                        state.page += 1;
                    }
                    if ui.button("Skip tour").clicked() {
                        state.show = false;
                        mark_tour_completed();
                    }
                } else if ui.button("Start editing!").clicked() {
                    state.show = false;
                    mark_tour_completed();
                }
            });
        });

    if !open {
        state.show = false;
        mark_tour_completed();
    }
}

#[derive(Default)]
pub struct FeatureGuidePlugin;

impl Plugin for FeatureGuidePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FeatureGuideState>()
            .add_systems(Startup, check_first_run)
            .add_systems(Update, render_feature_guide);
    }
}
