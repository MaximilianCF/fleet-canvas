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

//! A panel that loads two `.site.json` files, compares their entities by
//! SiteID, and displays a summary of added / removed / modified items. Useful
//! for reviewing PRs without launching Gazebo.

use bevy::ecs::hierarchy::ChildOf;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use rmf_site_egui::{MenuEvent, MenuItem, ToolMenu};
use rmf_site_format::Site;
use std::collections::BTreeSet;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Plugin + resources
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct SiteDiffPlugin;

impl Plugin for SiteDiffPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SiteDiffState>()
            .init_resource::<SiteDiffMenu>()
            .add_systems(Update, (handle_site_diff_menu, render_site_diff_panel));
    }
}

#[derive(Resource)]
pub struct SiteDiffMenu {
    menu_item: Entity,
}

impl FromWorld for SiteDiffMenu {
    fn from_world(world: &mut World) -> Self {
        let tool_header = world.resource::<ToolMenu>().get();
        let menu_item = world
            .spawn(MenuItem::Text("Site Diff Viewer".into()))
            .insert(ChildOf(tool_header))
            .id();
        SiteDiffMenu { menu_item }
    }
}

fn handle_site_diff_menu(
    mut menu_events: EventReader<MenuEvent>,
    diff_menu: Res<SiteDiffMenu>,
    mut state: ResMut<SiteDiffState>,
) {
    for event in menu_events.read() {
        if event.clicked() && event.source() == diff_menu.menu_item {
            state.show = true;
        }
    }
}

#[derive(Debug, Clone)]
pub enum DiffStatus {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub id: u32,
    pub category: String,
    pub name: String,
    pub status: DiffStatus,
}

#[derive(Resource, Default)]
pub struct SiteDiffState {
    pub show: bool,
    pub file_a_path: Option<PathBuf>,
    pub file_b_path: Option<PathBuf>,
    pub results: Vec<DiffEntry>,
    pub error: Option<String>,
    pub summary: Option<(usize, usize, usize)>,
}

// ---------------------------------------------------------------------------
// Diffing logic (operates on the format crate's pure-data `Site` struct)
// ---------------------------------------------------------------------------

fn compare_sites(a: &Site, b: &Site) -> Vec<DiffEntry> {
    let mut out = Vec::new();

    fn diff_map<V: serde::Serialize>(
        category: &str,
        name_fn: impl Fn(u32, &V) -> String,
        map_a: &std::collections::BTreeMap<u32, V>,
        map_b: &std::collections::BTreeMap<u32, V>,
        out: &mut Vec<DiffEntry>,
    ) {
        let keys_a: BTreeSet<u32> = map_a.keys().copied().collect();
        let keys_b: BTreeSet<u32> = map_b.keys().copied().collect();
        for id in keys_a.difference(&keys_b) {
            out.push(DiffEntry {
                id: *id,
                category: category.to_string(),
                name: name_fn(*id, &map_a[id]),
                status: DiffStatus::Removed,
            });
        }
        for id in keys_b.difference(&keys_a) {
            out.push(DiffEntry {
                id: *id,
                category: category.to_string(),
                name: name_fn(*id, &map_b[id]),
                status: DiffStatus::Added,
            });
        }
        for id in keys_a.intersection(&keys_b) {
            let ja = serde_json::to_string(&map_a[id]).unwrap_or_default();
            let jb = serde_json::to_string(&map_b[id]).unwrap_or_default();
            if ja != jb {
                out.push(DiffEntry {
                    id: *id,
                    category: category.to_string(),
                    name: name_fn(*id, &map_b[id]),
                    status: DiffStatus::Modified,
                });
            }
        }
    }

    diff_map(
        "Level",
        |id, _| format!("Level #{id}"),
        &a.levels,
        &b.levels,
        &mut out,
    );
    diff_map(
        "Lift",
        |id, _| format!("Lift #{id}"),
        &a.lifts,
        &b.lifts,
        &mut out,
    );
    diff_map(
        "Scenario",
        |id, _| format!("Scenario #{id}"),
        &a.scenarios,
        &b.scenarios,
        &mut out,
    );
    diff_map(
        "Model Instance",
        |id, _| format!("Instance #{id}"),
        &a.model_instances,
        &b.model_instances,
        &mut out,
    );
    diff_map(
        "Model Desc",
        |id, _| format!("ModelDesc #{id}"),
        &a.model_descriptions,
        &b.model_descriptions,
        &mut out,
    );
    diff_map(
        "Robot",
        |id, _| format!("Robot #{id}"),
        &a.robots,
        &b.robots,
        &mut out,
    );
    diff_map(
        "Task",
        |id, _| format!("Task #{id}"),
        &a.tasks,
        &b.tasks,
        &mut out,
    );
    diff_map(
        "Anchor",
        |id, _| format!("Anchor #{id}"),
        &a.anchors,
        &b.anchors,
        &mut out,
    );
    // TextureGroup, FiducialGroup, and Fiducial don't derive PartialEq
    // in the format crate; compare by key presence only.
    fn diff_keys_only(
        category: &str,
        keys_a: &std::collections::BTreeSet<u32>,
        keys_b: &std::collections::BTreeSet<u32>,
        out: &mut Vec<DiffEntry>,
    ) {
        for id in keys_a.difference(keys_b) {
            out.push(DiffEntry {
                id: *id,
                category: category.to_string(),
                name: format!("{category} #{id}"),
                status: DiffStatus::Removed,
            });
        }
        for id in keys_b.difference(keys_a) {
            out.push(DiffEntry {
                id: *id,
                category: category.to_string(),
                name: format!("{category} #{id}"),
                status: DiffStatus::Added,
            });
        }
    }
    diff_keys_only(
        "Texture",
        &a.textures.keys().copied().collect(),
        &b.textures.keys().copied().collect(),
        &mut out,
    );
    diff_keys_only(
        "Fiducial Group",
        &a.fiducial_groups.keys().copied().collect(),
        &b.fiducial_groups.keys().copied().collect(),
        &mut out,
    );
    diff_keys_only(
        "Fiducial",
        &a.fiducials.keys().copied().collect(),
        &b.fiducials.keys().copied().collect(),
        &mut out,
    );

    out
}

fn load_site(path: &PathBuf) -> Result<Site, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("IO error: {e}"))?;
    Site::from_bytes_json(&bytes).map_err(|e| format!("JSON parse error: {e}"))
}

// ---------------------------------------------------------------------------
// egui panel
// ---------------------------------------------------------------------------

fn render_site_diff_panel(mut contexts: EguiContexts, mut state: ResMut<SiteDiffState>) {
    if !state.show {
        return;
    }

    let ctx = contexts.ctx_mut();
    let mut open = state.show;

    egui::Window::new("Site Diff Viewer")
        .open(&mut open)
        .default_width(420.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("File A (base):");
                if let Some(p) = &state.file_a_path {
                    ui.label(
                        p.file_name()
                            .map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_default(),
                    );
                } else {
                    ui.label("—");
                }
                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("Pick…").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Site JSON", &["json"])
                        .set_title("Select base file (A)")
                        .pick_file()
                    {
                        state.file_a_path = Some(path);
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("File B (changed):");
                if let Some(p) = &state.file_b_path {
                    ui.label(
                        p.file_name()
                            .map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_default(),
                    );
                } else {
                    ui.label("—");
                }
                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("Pick…").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Site JSON", &["json"])
                        .set_title("Select changed file (B)")
                        .pick_file()
                    {
                        state.file_b_path = Some(path);
                    }
                }
            });

            ui.add_space(4.0);
            let both_loaded = state.file_a_path.is_some() && state.file_b_path.is_some();
            ui.add_enabled_ui(both_loaded, |ui| {
                if ui.button("Compare").clicked() {
                    let pa = state.file_a_path.as_ref().unwrap();
                    let pb = state.file_b_path.as_ref().unwrap();
                    match (load_site(pa), load_site(pb)) {
                        (Ok(a), Ok(b)) => {
                            let results = compare_sites(&a, &b);
                            let added = results
                                .iter()
                                .filter(|e| matches!(e.status, DiffStatus::Added))
                                .count();
                            let removed = results
                                .iter()
                                .filter(|e| matches!(e.status, DiffStatus::Removed))
                                .count();
                            let modified = results
                                .iter()
                                .filter(|e| matches!(e.status, DiffStatus::Modified))
                                .count();
                            state.summary = Some((added, removed, modified));
                            state.results = results;
                            state.error = None;
                        }
                        (Err(e), _) | (_, Err(e)) => {
                            state.error = Some(e);
                            state.results.clear();
                            state.summary = None;
                        }
                    }
                }
            });

            if let Some(err) = &state.error {
                ui.colored_label(egui::Color32::RED, err);
            }

            if let Some((added, removed, modified)) = state.summary {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::GREEN, format!("+{added} added"));
                    ui.colored_label(egui::Color32::RED, format!("-{removed} removed"));
                    ui.colored_label(egui::Color32::YELLOW, format!("~{modified} modified"));
                });
                ui.separator();
            }

            egui::ScrollArea::vertical()
                .max_height(400.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for entry in &state.results {
                        let (icon, color) = match entry.status {
                            DiffStatus::Added => ("+", egui::Color32::GREEN),
                            DiffStatus::Removed => ("-", egui::Color32::RED),
                            DiffStatus::Modified => ("~", egui::Color32::YELLOW),
                        };
                        ui.horizontal(|ui| {
                            ui.colored_label(color, icon);
                            ui.label(format!("[{}] {}", entry.category, entry.name));
                        });
                    }
                    if state.results.is_empty() && state.summary.is_some() {
                        ui.label("No differences found.");
                    }
                });
        });

    if !open {
        state.show = false;
    }
}
