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

use crate::*;

use bevy_egui::egui;
use smallvec::SmallVec;
use tracing::error;

/// Input type for [`WidgetSystem`]s that can be put into a "Panel of Tiles"
/// widget, such as the [`PropertiesPanel`]. See [`PropertiesTilePlugin`] for a
/// usage example.
pub struct Tile {
    /// The entity of the tile widget which is being rendered. This lets you
    /// store additional component data inside the entity which may be relevant
    /// to your widget.
    pub id: Entity,
    /// What kind of panel is this tile inside of. Use this if you want your
    /// widget layout to be different based on what kind of panel it was placed
    /// in.
    pub panel: PanelSide,
}

/// Assigns a tile widget to a named tab in the tabbed panel.
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PanelTab(pub String);

/// Tracks the currently active tab in the properties panel.
#[derive(Resource, Clone, Debug)]
pub struct ActivePanelTab {
    pub tab: String,
}

impl Default for ActivePanelTab {
    fn default() -> Self {
        Self {
            tab: "Inspect".to_string(),
        }
    }
}

/// Tab order and labels for the tabbed panel.
pub const PANEL_TAB_ORDER: &[&str] = &["Inspect", "Site", "Nav", "Tasks"];

/// Reusable widget that defines a panel with "tiles" where each tile is a child widget.
pub fn show_panel_of_tiles(
    In(PanelWidgetInput { id, context }): In<PanelWidgetInput>,
    world: &mut World,
) {
    let children: Option<SmallVec<[Entity; 16]>> = world
        .get::<Children>(id)
        .map(|children| children.iter().collect());

    let Some(children) = children else {
        return;
    };
    if children.is_empty() {
        return;
    }

    let Some(side) = world.get::<PanelSide>(id) else {
        error!("Side component missing for panel_of_tiles_widget {id:?}");
        return;
    };

    let side = *side;
    let config = world.get::<PanelConfig>(id).cloned().unwrap_or_default();

    // Check if any children have PanelTab — if so, use tabbed rendering
    let has_tabs = children
        .iter()
        .any(|&child| world.get::<PanelTab>(child).is_some());

    side.get_panel()
        .map_vertical(|panel| {
            panel
                .resizable(config.resizable)
                .default_width(config.default_dimension)
        })
        .map_horizontal(|panel| {
            panel
                .resizable(config.resizable)
                .default_height(config.default_dimension)
        })
        .show(&context, |ui| {
            if has_tabs {
                render_tabbed_panel(ui, world, &children, side, id, &config);
            } else {
                egui::ScrollArea::new(config.enable_scroll())
                    .auto_shrink(config.auto_shrink())
                    .show(ui, |ui| {
                        side.align(ui, |ui| render_tiles(ui, world, &children, side, id));
                    });
            }
        });
}

fn render_tabbed_panel(
    ui: &mut Ui,
    world: &mut World,
    children: &[Entity],
    side: PanelSide,
    id: Entity,
    config: &PanelConfig,
) {
    let active_tab = world
        .get_resource::<ActivePanelTab>()
        .map(|t| t.tab.clone())
        .unwrap_or_else(|| "Inspect".to_string());

    // Render tab bar
    let mut new_tab = active_tab.clone();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for &tab_name in PANEL_TAB_ORDER {
            let selected = active_tab == tab_name;
            let button = egui::Button::new(egui::RichText::new(tab_name).strong().size(13.0));
            let button = if selected {
                button.fill(ui.visuals().selection.bg_fill)
            } else {
                button.fill(egui::Color32::TRANSPARENT)
            };
            let response = ui.add_sized(
                egui::vec2(ui.available_width() / PANEL_TAB_ORDER.len() as f32, 28.0),
                button,
            );
            if response.clicked() {
                new_tab = tab_name.to_string();
            }
        }
    });

    if new_tab != active_tab {
        if let Some(mut tab_res) = world.get_resource_mut::<ActivePanelTab>() {
            tab_res.tab = new_tab.clone();
        }
    }

    ui.separator();

    // Filter children by active tab; also render children without a tab (untagged)
    let active = world
        .get_resource::<ActivePanelTab>()
        .map(|t| t.tab.clone())
        .unwrap_or_else(|| "Inspect".to_string());

    let filtered: SmallVec<[Entity; 16]> = children
        .iter()
        .filter(|&&child| {
            world
                .get::<PanelTab>(child)
                .map(|t| t.0 == active)
                .unwrap_or(false)
        })
        .copied()
        .collect();

    // Render filtered tiles in scroll area
    egui::ScrollArea::new(config.enable_scroll())
        .auto_shrink(config.auto_shrink())
        .show(ui, |ui| {
            ui.add_space(4.0);
            side.align(ui, |ui| render_tiles(ui, world, &filtered, side, id));
        });
}

fn render_tiles(ui: &mut Ui, world: &mut World, children: &[Entity], side: PanelSide, id: Entity) {
    for &child in children {
        let tile = Tile {
            id: child,
            panel: side,
        };
        if let Err(err) = world.try_show_in(child, tile, ui) {
            error!(
                "Could not render child widget {child:?} in \
                                tile panel {id:?} on side {side:?}: {err:?}"
            );
        }
    }
}
