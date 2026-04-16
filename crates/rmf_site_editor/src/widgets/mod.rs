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

//! The site editor allows you to insert your own egui widgets into the UI.
//! Simple examples of custom widgets can be found in the docs for
//! [`PropertiesTilePlugin`] and [`InspectionPlugin`].
//!
//! There are three categories of widgets that the site editor provides
//! out-of-the-box support for inserting, but the widget system itself is
//! highly extensible, allowing you to define your own categories of widgets.
//!
//! The three categories provided out of the box include:
//! - [Panel widget][1]: Add a new panel to the UI.
//! - Tile widget: Add a tile into a [panel of tiles][2] such as the [`PropertiesPanel`]. Use [`PropertiesTilePlugin`] to make a new tile widget that goes inside of the standard `PropertiesPanel`.
//! - [`InspectionPlugin`]: Add a widget to the [`MainInspector`] to display more information about the currently selected entity.
//!
//! In our terminology, there are two kinds of panels:
//! - Side panels: A vertical column widget on the left or right side of the screen.
//!   - [`PropertiesPanel`] is usually a side panel placed on the right side of the screen.
//!   - [`FuelAssetBrowser`] is a side panel typically placed on the left side of the screen.
//!   - [`Diagnostics`] is a side panel that interactively flags issues that have been found in the site.
//! - Top / Bottom Panels:
//!   - The [`MenuBarPlugin`] provides a menu bar at the top of the screen.
//!     - Create an entity with a [`Menu`] component to create a new menu inside the menu bar.
//!     - Add an entity with a [`MenuItem`] component as a child to a menu entity to add a new item into a menu.
//!     - The [`FileMenu`], [`ToolMenu`], and [`ViewMenu`] are resources that provide access to various standard menus.
//!   - The [`ConsoleWidgetPlugin`] provides a console at the bottom of the screen to display information, warning, and error messages.
//!
//! [1]: crate::widgets::PanelWidget
//! [2]: crate::widgets::show_panel_of_tiles

use crate::AppState;
use bevy::{ecs::system::SystemState, prelude::*};
use bevy_egui::{egui, EguiContexts};

pub mod about_dialog;
pub use about_dialog::*;

pub mod building_preview;
use building_preview::*;

pub mod console;
use console::*;

pub mod creation;
use creation::*;

pub mod diagnostics;
use diagnostics::*;

pub mod edit_menu;
use edit_menu::*;

pub mod feature_guide;
pub use feature_guide::*;

pub mod fuel_asset_browser;
pub use fuel_asset_browser::*;

pub mod icons;
pub use icons::*;

pub mod inspector;
pub use inspector::*;

pub mod move_layer;
pub use move_layer::*;

pub mod sdf_export_dialog;
pub use sdf_export_dialog::*;

pub mod sdf_export_menu;
use rmf_site_egui::*;
use rmf_site_picking::{Hover, SelectionServiceStages, UiFocused};
pub use sdf_export_menu::*;

pub mod nearby_elements;
use nearby_elements::*;

pub mod saved_views;
use saved_views::*;

pub mod minimap;
pub use minimap::*;

pub mod search_bar;
use search_bar::*;

pub mod site_diff;
pub use site_diff::*;

pub mod selector_widget;
pub use selector_widget::*;

pub mod status_bar;
pub use status_bar::*;

pub mod tasks;
pub use tasks::*;

pub mod user_camera_display;
pub use user_camera_display::*;

pub mod view_groups;
use view_groups::*;

pub mod view_layers;
use view_layers::*;

pub mod view_levels;
use view_levels::*;

pub mod view_model_instances;
use view_model_instances::*;

pub mod view_scenarios;
use view_scenarios::*;

pub mod view_lights;
use view_lights::*;

pub mod view_nav_graphs;
use view_nav_graphs::*;

pub mod notifications;
pub use notifications::*;

pub mod workspace;
use workspace::*;

pub mod prelude {
    //! This module gives easy access to the traits, structs, and plugins that
    //! we expect downstream users are likely to want easy access to if they are
    //! implementing and inserting their own widgets.

    pub use super::{Inspect, InspectionPlugin};
    pub use bevy::ecs::{
        system::{SystemParam, SystemState},
        world::World,
    };
    pub use bevy_egui::egui::Ui;
}

/// This plugins produces the standard properties panel. This is the panel which
/// includes widgets to display and edit all the properties in a site that we
/// expect are needed by common use cases of the editor.
#[derive(Default)]
pub struct StandardPropertiesPanelPlugin {}

impl Plugin for StandardPropertiesPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PropertiesPanelPlugin::new(PanelSide::Right),
            ViewLevelsPlugin::default(),
            ViewScenariosPlugin::default(),
            ViewModelInstancesPlugin::default(),
            ViewNavGraphsPlugin::default(),
            ViewLayersPlugin::default(),
            StandardTasksPlugin::default(),
            SearchBarPlugin::default(),
            StandardInspectorPlugin::default(),
            ViewGroupsPlugin::default(),
            ViewLightsPlugin::default(),
            BuildingPreviewPlugin::default(),
            SavedViewsPlugin::default(),
            NearbyElementsPlugin::default(),
        ));
    }
}

/// This plugin provides the standard UI layout that was designed for the common
/// use cases of the site editor.
#[derive(Default)]
pub struct StandardUiPlugin {}

impl Plugin for StandardUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CanvasTooltips>()
            .add_plugins((
                IconsPlugin::default(),
                MenuBarPlugin::default(),
                StandardPropertiesPanelPlugin::default(),
                FuelAssetBrowserPlugin,
                DiagnosticsPlugin::default(),
                ConsoleWidgetPlugin::default(),
                WorkspaceMenuPlugin::default(),
                EditMenuPlugin::default(),
                UserCameraDisplayPlugin::default(),
                StandardCreationPlugin::default(),
                NotificationsPlugin,
                StatusBarPlugin,
                #[cfg(not(target_arch = "wasm32"))]
                SdfExportMenuPlugin::default(),
                #[cfg(not(target_arch = "wasm32"))]
                SdfExportDialogPlugin,
                #[cfg(not(target_arch = "wasm32"))]
                NavGraphIoPlugin::default(),
            ))
            .add_plugins((
                MinimapPlugin::default(),
                SiteDiffPlugin::default(),
                FeatureGuidePlugin::default(),
                AboutDialogPlugin::default(),
            ))
            .add_systems(Startup, init_ui_style)
            .add_systems(
                Update,
                site_ui_layout
                    .in_set(RenderUiSet)
                    .run_if(AppState::in_displaying_mode())
                    .after(SelectionServiceStages::SelectFlush),
            )
            .add_systems(
                PostUpdate,
                (resolve_light_export_file,).run_if(AppState::in_displaying_mode()),
            );
    }
}

/// This set is for systems that impact rendering the UI using egui. The
/// [`UserCameraDisplay`] resource waits until after this set is finished before
/// computing the user camera area.
#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone)]
pub struct RenderUiSet;

/// This system renders all UI panels in the application and makes sure that the
/// UI rendering works correctly with the picking system, and any other systems
/// as needed.
pub fn site_ui_layout(
    world: &mut World,
    panel_widgets: &mut QueryState<(Entity, &mut PanelWidget)>,
    egui_context_state: &mut SystemState<EguiContexts>,
) {
    render_panels(world, panel_widgets, egui_context_state);

    let mut egui_context = egui_context_state.get_mut(world);
    let mut ctx = egui_context.ctx_mut().clone();

    let ui_has_focus = if let Some(picking_blocker) = world.get_resource::<UiFocused>() {
        picking_blocker.0.clone()
    } else {
        false
    };

    if ui_has_focus {
        // If the UI has focus and there were no hover events emitted by the UI,
        // then we should emit a None hover event
        let mut hover = world.resource_mut::<Events<Hover>>();
        if hover.is_empty() {
            hover.send(Hover(None));
        }
    } else {
        // If the UI does not have focus then render the CanvasTooltips.
        world.resource_mut::<CanvasTooltips>().render(&mut ctx);
    }
}

fn init_ui_style(mut egui_context: EguiContexts) {
    let mut visuals = egui::Visuals::dark();

    // Text — brighter than egui default
    visuals.override_text_color = Some(egui::Color32::from_rgb(230, 230, 230));

    // Panel/window backgrounds — match the Bevy viewport dark grey
    let panel_bg = egui::Color32::from_rgb(28, 28, 30);
    let widget_bg = egui::Color32::from_rgb(44, 44, 46);
    let widget_bg_hover = egui::Color32::from_rgb(58, 58, 62);
    let widget_bg_active = egui::Color32::from_rgb(72, 72, 76);
    let stroke_color = egui::Color32::from_rgb(80, 80, 84);
    let accent = egui::Color32::from_rgb(0, 122, 255);

    visuals.panel_fill = panel_bg;
    visuals.window_fill = panel_bg;
    visuals.extreme_bg_color = egui::Color32::from_rgb(18, 18, 20);

    visuals.widgets.noninteractive.bg_fill = widget_bg;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, stroke_color);
    visuals.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 180, 180));

    visuals.widgets.inactive.bg_fill = widget_bg;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, stroke_color);

    visuals.widgets.hovered.bg_fill = widget_bg_hover;
    visuals.widgets.hovered.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(120, 120, 128));

    visuals.widgets.active.bg_fill = widget_bg_active;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.5, accent);

    visuals.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(0, 122, 255, 80);
    visuals.selection.stroke = egui::Stroke::new(1.0, accent);

    visuals.hyperlink_color = accent;
    visuals.faint_bg_color = egui::Color32::from_rgb(36, 36, 38);

    visuals.window_stroke = egui::Stroke::new(1.0, stroke_color);

    egui_context.ctx_mut().set_visuals(visuals);
}
