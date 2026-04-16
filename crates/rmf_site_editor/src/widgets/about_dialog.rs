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

//! Help > About dialog and Help menu.

use bevy::ecs::hierarchy::ChildOf;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use rmf_site_egui::{Menu, MenuEvent, MenuItem};

use super::FeatureGuideState;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Resource, Default)]
pub struct AboutDialogState {
    pub show: bool,
}

#[derive(Resource)]
pub struct HelpMenu {
    menu: Entity,
    about: Entity,
    feature_guide: Entity,
}

impl FromWorld for HelpMenu {
    fn from_world(world: &mut World) -> Self {
        let menu = world.spawn(Menu::from_title("Help".to_string())).id();
        let feature_guide = world
            .spawn(MenuItem::Text("Feature Guide".into()))
            .insert(ChildOf(menu))
            .id();
        let about = world
            .spawn(MenuItem::Text("About Fleet Canvas".into()))
            .insert(ChildOf(menu))
            .id();
        HelpMenu {
            menu,
            about,
            feature_guide,
        }
    }
}

fn handle_help_menu(
    mut menu_events: EventReader<MenuEvent>,
    help_menu: Res<HelpMenu>,
    mut about: ResMut<AboutDialogState>,
    mut guide: ResMut<FeatureGuideState>,
) {
    for event in menu_events.read() {
        if event.clicked() && event.source() == help_menu.about {
            about.show = true;
        } else if event.clicked() && event.source() == help_menu.feature_guide {
            guide.show = true;
        }
    }
}

fn render_about_dialog(mut contexts: EguiContexts, mut state: ResMut<AboutDialogState>) {
    if !state.show {
        return;
    }

    let mut open = state.show;
    egui::Window::new("About Fleet Canvas")
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(380.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                ui.heading("Fleet Canvas");
                ui.label(
                    egui::RichText::new(format!("v{VERSION}"))
                        .small()
                        .color(egui::Color32::from_rgb(140, 180, 220)),
                );
                ui.add_space(12.0);
                ui.label("Visual editor for Open-RMF robot fleet deployment sites.");
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Built with Bevy 0.16 + egui")
                        .small()
                        .color(egui::Color32::from_rgb(160, 160, 170)),
                );
                ui.label(
                    egui::RichText::new("Platform: Linux desktop")
                        .small()
                        .color(egui::Color32::from_rgb(160, 160, 170)),
                );
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("\u{00A9} 2026 MaximilianCF \u{2014} Apache-2.0 license")
                        .small(),
                );
                ui.label(
                    egui::RichText::new(
                        "Based on open-rmf/rmf_site by Open Source Robotics Foundation",
                    )
                    .small()
                    .color(egui::Color32::from_rgb(140, 140, 160)),
                );
                ui.add_space(8.0);
                ui.hyperlink_to(
                    "GitHub \u{2197}",
                    "https://github.com/MaximilianCF/fleet-canvas",
                );
            });
        });

    if !open {
        state.show = false;
    }
}

#[derive(Default)]
pub struct AboutDialogPlugin;

impl Plugin for AboutDialogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AboutDialogState>()
            .init_resource::<HelpMenu>()
            .add_systems(Update, (handle_help_menu, render_about_dialog));
    }
}
