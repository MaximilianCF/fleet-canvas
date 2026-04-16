/*
 * Copyright (C) 2024 Open Source Robotics Foundation
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

use crate::site::{Category, FlashHighlight, NameInSite, SiteID};
use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use bevy_egui::egui::{self, Ui};
use rmf_site_camera::PanToElement;
use rmf_site_egui::*;
use rmf_site_picking::{Select, Selection};

const MAX_SEARCH_RESULTS: usize = 20;

#[derive(Resource, Default)]
pub struct SearchBarState {
    pub query: String,
}

#[derive(Default)]
pub struct SearchBarPlugin;

impl Plugin for SearchBarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SearchBarState>()
            .add_plugins(PropertiesTilePlugin::<SearchBar>::new().tab("Inspect"));
    }
}

#[derive(SystemParam)]
pub struct SearchBar<'w, 's> {
    entities: Query<
        'w,
        's,
        (
            Entity,
            &'static NameInSite,
            Option<&'static Category>,
            Option<&'static SiteID>,
        ),
    >,
    search_state: ResMut<'w, SearchBarState>,
    selection: Res<'w, Selection>,
    select: EventWriter<'w, Select>,
    pan_to: ResMut<'w, PanToElement>,
    commands: Commands<'w, 's>,
}

impl<'w, 's> WidgetSystem<Tile> for SearchBar<'w, 's> {
    fn show(_: Tile, ui: &mut Ui, state: &mut SystemState<Self>, world: &mut World) {
        let mut params = state.get_mut(world);

        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut params.search_state.query);
            if !params.search_state.query.is_empty() && ui.small_button("x").clicked() {
                params.search_state.query.clear();
            }
        });

        let query = params.search_state.query.trim().to_lowercase();
        if query.is_empty() {
            return;
        }

        ui.separator();

        // Support #ID syntax to jump to a specific SiteID
        if let Some(id_str) = query.strip_prefix('#') {
            if let Ok(target_id) = id_str.trim().parse::<u32>() {
                let mut found = false;
                for (e, name, cat, sid) in params.entities.iter() {
                    if sid.is_some_and(|s| s.0 == target_id) {
                        let cat_label = cat.map(|c| c.label()).unwrap_or("");
                        let is_selected = params.selection.0.is_some_and(|s| s == e);
                        let label = format!("{} #{} [{}]", name.0, target_id, cat_label);
                        if ui.selectable_label(is_selected, &label).clicked() {
                            params.select.write(Select::new(Some(e)));
                            params.pan_to.target = Some(e);
                            params.pan_to.interruptible = true;
                            params.commands.entity(e).insert(FlashHighlight::new(1.5));
                        }
                        found = true;
                    }
                }
                if !found {
                    ui.label(
                        egui::RichText::new(format!("No entity with ID #{}", target_id)).weak(),
                    );
                }
                ui.separator();
                return;
            }
        }

        let mut results: Vec<(Entity, String, String, String)> = params
            .entities
            .iter()
            .filter(|(_, name, _, _)| name.0.to_lowercase().contains(&query))
            .map(
                |(e, name, cat, sid): (Entity, &NameInSite, Option<&Category>, Option<&SiteID>)| {
                    let cat_label = cat.map(|c| c.label()).unwrap_or("");
                    let id_str = sid.map(|id| format!(" #{}", id.0)).unwrap_or_default();
                    (e, name.0.clone(), cat_label.to_string(), id_str)
                },
            )
            .collect();

        results.sort_by(|a, b| a.1.cmp(&b.1));
        let total = results.len();
        results.truncate(MAX_SEARCH_RESULTS);

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for (entity, name, cat_label, id_str) in &results {
                    let is_selected = params.selection.0.is_some_and(|s| s == *entity);
                    let label = format!("{}{} [{}]", name, id_str, cat_label);

                    let response = ui.selectable_label(is_selected, &label);
                    if response.clicked() {
                        params.select.write(Select::new(Some(*entity)));
                        params.pan_to.target = Some(*entity);
                        params.pan_to.interruptible = true;
                        params
                            .commands
                            .entity(*entity)
                            .insert(FlashHighlight::new(1.5));
                    }
                }
            });

        if total > MAX_SEARCH_RESULTS {
            ui.label(
                egui::RichText::new(format!("... and {} more", total - MAX_SEARCH_RESULTS))
                    .small()
                    .weak(),
            );
        }

        ui.separator();
    }
}
