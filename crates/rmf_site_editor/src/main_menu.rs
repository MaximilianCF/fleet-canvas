/*
 * Copyright (C) 2022 Open Source Robotics Foundation
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

use super::demo_world::*;
use crate::{site::LoadSite, AppState, Autoload, WorkspaceLoader};
use bevy::{app::AppExit, prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn egui_ui(
    mut egui_context: EguiContexts,
    mut _exit: EventWriter<AppExit>,
    mut workspace_loader: WorkspaceLoader,
    mut _app_state: ResMut<State<AppState>>,
    autoload: Option<ResMut<Autoload>>,
    primary_windows: Query<Entity, With<PrimaryWindow>>,
    #[cfg(not(target_arch = "wasm32"))] prefs: Option<Res<crate::user_preferences::UserPreferences>>,
) {
    if let Some(mut autoload) = autoload {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(filename) = autoload.filename.take() {
                let _ = workspace_loader.load_from_path(filename);
            }
        }
        return;
    }

    let Some(ctx) = primary_windows
        .single()
        .ok()
        .and_then(|w| egui_context.try_ctx_for_entity_mut(w))
    else {
        return;
    };

    let panel_fill = egui::Color32::from_rgb(30, 30, 35);

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(panel_fill))
        .show(ctx, |ui| {
            let available = ui.available_size();

            // Center everything vertically
            let content_height = 320.0;
            let top_pad = ((available.y - content_height) / 2.0).max(40.0);

            ui.add_space(top_pad);

            ui.vertical_centered(|ui| {
                // Title
                ui.label(
                    egui::RichText::new("RMF Site Editor")
                        .size(32.0)
                        .strong()
                        .color(egui::Color32::from_rgb(240, 240, 240)),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!("v{VERSION}  --  Desktop Edition"))
                        .size(13.0)
                        .color(egui::Color32::from_rgb(140, 140, 150)),
                );

                ui.add_space(40.0);

                // Action buttons in a centered column
                let button_width = 260.0;
                let button_height = 36.0;

                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing.y = 10.0;

                    let new_btn = egui::Button::new(egui::RichText::new("New Project").size(15.0))
                        .min_size(egui::vec2(button_width, button_height));
                    if ui.add(new_btn).clicked() {
                        workspace_loader.create_empty_from_dialog();
                    }

                    let open_btn = egui::Button::new(egui::RichText::new("Open File").size(15.0))
                        .min_size(egui::vec2(button_width, button_height));
                    if ui.add(open_btn).clicked() {
                        workspace_loader.load_from_dialog();
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    if let Some(prefs) = &prefs {
                        if let Some(last_file) = &prefs.last_file {
                            let filename = last_file
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("recent file");
                            let label = format!("Open Recent: {filename}");
                            let recent_btn =
                                egui::Button::new(egui::RichText::new(&label).size(14.0))
                                    .min_size(egui::vec2(button_width, button_height))
                                    .fill(egui::Color32::from_rgb(45, 50, 55));
                            let path = last_file.clone();
                            if ui.add(recent_btn).clicked() {
                                let _ = workspace_loader.load_from_path(path);
                            }
                        }
                    }

                    let demo_btn =
                        egui::Button::new(egui::RichText::new("Load Demo Map").size(15.0))
                            .min_size(egui::vec2(button_width, button_height))
                            .fill(egui::Color32::from_rgb(45, 45, 55));
                    if ui.add(demo_btn).clicked() {
                        workspace_loader
                            .load_site(async move { LoadSite::from_data(&demo_office(), None) });
                    }
                });

                ui.add_space(30.0);

                // Keyboard hints
                ui.label(
                    egui::RichText::new("Ctrl+N  New  |  Ctrl+O  Open  |  Ctrl+S  Save")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(100, 100, 110)),
                );

                #[cfg(not(target_arch = "wasm32"))]
                {
                    ui.add_space(20.0);
                    let exit_btn = egui::Button::new(
                        egui::RichText::new("Exit")
                            .size(13.0)
                            .color(egui::Color32::from_rgb(160, 160, 170)),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 70, 80)))
                    .min_size(egui::vec2(100.0, 28.0));
                    if ui.add(exit_btn).clicked() {
                        _exit.write(AppExit::Success);
                    }
                }
            });
        });
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, egui_ui.run_if(in_state(AppState::MainMenu)));
    }
}
