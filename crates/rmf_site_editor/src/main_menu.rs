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

/// Draw an animated blueprint-style background on the main menu.
fn draw_blueprint_background(ui: &mut egui::Ui, time: f64) {
    let painter = ui.painter();
    let rect = ui.max_rect();

    // --- Grid lines ---
    let grid_spacing = 40.0;
    let major_every = 4; // every 4th line is major
    let drift = (time * 3.0) % grid_spacing as f64; // slow drift

    let minor_color = egui::Color32::from_rgba_premultiplied(60, 70, 90, 12);
    let major_color = egui::Color32::from_rgba_premultiplied(70, 90, 120, 20);

    // Vertical lines
    let mut i = 0;
    let mut x = rect.left() - grid_spacing + drift as f32;
    while x < rect.right() + grid_spacing {
        let color = if i % major_every == 0 {
            major_color
        } else {
            minor_color
        };
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            egui::Stroke::new(0.5, color),
        );
        x += grid_spacing;
        i += 1;
    }

    // Horizontal lines
    i = 0;
    let mut y = rect.top() - grid_spacing + drift as f32 * 0.7;
    while y < rect.bottom() + grid_spacing {
        let color = if i % major_every == 0 {
            major_color
        } else {
            minor_color
        };
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(0.5, color),
        );
        y += grid_spacing;
        i += 1;
    }

    // --- Floating "room" outlines (blueprint floor plan feel) ---
    let shapes: &[(f32, f32, f32, f32, f64, f64)] = &[
        // (rel_x, rel_y, w, h, speed_x, speed_y)
        (0.12, 0.18, 120.0, 80.0, 0.15, 0.08),
        (0.75, 0.25, 90.0, 60.0, -0.10, 0.12),
        (0.20, 0.70, 100.0, 70.0, 0.08, -0.10),
        (0.80, 0.72, 70.0, 100.0, -0.12, -0.06),
        (0.45, 0.15, 140.0, 50.0, 0.05, 0.14),
        (0.55, 0.82, 80.0, 90.0, -0.08, -0.11),
    ];

    let room_stroke = egui::Stroke::new(
        0.8,
        egui::Color32::from_rgba_premultiplied(80, 120, 180, 18),
    );
    let door_color = egui::Color32::from_rgba_premultiplied(100, 160, 220, 22);

    for (rx, ry, w, h, sx, sy) in shapes {
        let cx = rect.left() + rect.width() * rx + (time * sx * 10.0).sin() as f32 * 15.0;
        let cy = rect.top() + rect.height() * ry + (time * sy * 10.0).cos() as f32 * 12.0;

        let room = egui::Rect::from_center_size(egui::pos2(cx, cy), egui::vec2(*w, *h));
        painter.rect_stroke(room, 0.0, room_stroke, egui::StrokeKind::Outside);

        // Small "door" gap on one side
        let door_x = room.left() + w * 0.3;
        painter.line_segment(
            [
                egui::pos2(door_x, room.top()),
                egui::pos2(door_x + 12.0, room.top() - 6.0),
            ],
            egui::Stroke::new(0.8, door_color),
        );
    }

    // --- Waypoint dots (nav graph nodes) ---
    let waypoints: &[(f32, f32, f64, f64)] = &[
        (0.30, 0.40, 0.20, 0.15),
        (0.65, 0.50, -0.12, 0.18),
        (0.40, 0.60, 0.16, -0.10),
        (0.50, 0.35, -0.08, -0.14),
        (0.15, 0.50, 0.10, 0.20),
        (0.85, 0.45, -0.15, 0.05),
        (0.35, 0.85, 0.12, -0.08),
        (0.70, 0.15, -0.06, 0.16),
    ];

    let dot_color = egui::Color32::from_rgba_premultiplied(100, 180, 255, 20);
    let lane_color = egui::Color32::from_rgba_premultiplied(80, 140, 200, 12);

    let positions: Vec<egui::Pos2> = waypoints
        .iter()
        .map(|(rx, ry, sx, sy)| {
            let x = rect.left() + rect.width() * rx + (time * sx * 8.0).sin() as f32 * 20.0;
            let y = rect.top() + rect.height() * ry + (time * sy * 8.0).cos() as f32 * 18.0;
            egui::pos2(x, y)
        })
        .collect();

    // Draw lanes between some waypoints
    let connections: &[(usize, usize)] = &[
        (0, 1),
        (1, 3),
        (0, 2),
        (2, 3),
        (4, 0),
        (5, 1),
        (2, 6),
        (3, 7),
    ];
    for (a, b) in connections {
        painter.line_segment(
            [positions[*a], positions[*b]],
            egui::Stroke::new(1.0, lane_color),
        );
    }

    for pos in &positions {
        painter.circle_filled(*pos, 3.5, dot_color);
    }

    // Request continuous repaint for animation
    ui.ctx().request_repaint();
}

fn egui_ui(
    mut egui_context: EguiContexts,
    mut _exit: EventWriter<AppExit>,
    mut workspace_loader: WorkspaceLoader,
    mut _app_state: ResMut<State<AppState>>,
    autoload: Option<ResMut<Autoload>>,
    primary_windows: Query<Entity, With<PrimaryWindow>>,
    #[cfg(not(target_arch = "wasm32"))] prefs: Option<
        Res<crate::user_preferences::UserPreferences>,
    >,
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
            let time = ui.input(|i| i.time);
            draw_blueprint_background(ui, time);

            let available = ui.available_size();

            // Center everything vertically
            let content_height = 320.0;
            let top_pad = ((available.y - content_height) / 2.0).max(40.0);

            ui.add_space(top_pad);

            ui.vertical_centered(|ui| {
                // Title
                ui.label(
                    egui::RichText::new("Fleet Canvas")
                        .size(32.0)
                        .strong()
                        .color(egui::Color32::from_rgb(240, 240, 240)),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!("v{VERSION}  \u{00B7}  Desktop Edition"))
                        .size(13.0)
                        .color(egui::Color32::from_rgb(140, 140, 160)),
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
