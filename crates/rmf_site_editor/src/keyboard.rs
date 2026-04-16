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

use crate::{
    interaction::{
        CopyProperties, MeasureTool, MultiSelection, PasteProperties, SnapGridConfig, SnapToGrid,
    },
    site::{AlignSiteDrawings, Delete, ToggleNavGraphView, ViewMenuItems},
    undo::{RedoRequest, UndoRequest},
    widgets::{Notifications, SdfExportDialogState},
    CreateNewWorkspace, CurrentWorkspace, DebugMode, WorkspaceLoader, WorkspaceSaver,
};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;
use crossflow::*;
use rmf_site_camera::resources::ProjectionMode;
use rmf_site_egui::MenuItem;
use rmf_site_picking::Selection;

/// plugin for managing input settings for rmf_site_editor
pub struct EditorInputPlugin;

impl Plugin for EditorInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugMode>().add_systems(
            Last,
            (
                handle_keyboard_input,
                handle_keyboard_extras,
                handle_keyboard_tools,
            ),
        );
    }
}

fn handle_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut egui_context: EguiContexts,
    mut new_workspace: EventWriter<CreateNewWorkspace>,
    mut projection_mode: ResMut<ProjectionMode>,
    primary_windows: Query<Entity, With<PrimaryWindow>>,
    mut workspace_loader: WorkspaceLoader,
    mut workspace_saver: WorkspaceSaver,
    mut undo_request: EventWriter<UndoRequest>,
    mut redo_request: EventWriter<RedoRequest>,
    mut snap: ResMut<SnapToGrid>,
    mut notifications: ResMut<Notifications>,
    mut grid_config: ResMut<SnapGridConfig>,
    mut sdf_dialog: ResMut<SdfExportDialogState>,
) {
    let Some(egui_context) = primary_windows
        .single()
        .ok()
        .and_then(|w| egui_context.try_ctx_for_entity_mut(w))
    else {
        return;
    };
    let ui_has_focus = egui_context.wants_pointer_input()
        || egui_context.wants_keyboard_input()
        || egui_context.is_pointer_over_area();

    if ui_has_focus {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::F2) {
        *projection_mode = ProjectionMode::Orthographic;
        notifications.success("Projection: Orthographic");
    }

    if keyboard_input.just_pressed(KeyCode::F3) {
        *projection_mode = ProjectionMode::Perspective;
        notifications.success("Projection: Perspective");
    }

    // Delete and debug mode handled in handle_keyboard_extras

    if keyboard_input.just_pressed(KeyCode::KeyG) {
        if keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            // Cycle grid size
            let idx = SnapToGrid::PRESETS
                .iter()
                .position(|&s| (s - snap.grid_size).abs() < 1e-4)
                .map(|i| (i + 1) % SnapToGrid::PRESETS.len())
                .unwrap_or(0);
            snap.grid_size = SnapToGrid::PRESETS[idx];
            notifications.success(format!("Grid size: {}m", snap.grid_size));
        } else if keyboard_input.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]) {
            // Toggle reference grid
            grid_config.visible = !grid_config.visible;
            let state = if grid_config.visible { "ON" } else { "OFF" };
            notifications.success(format!("Reference grid: {state}"));
        } else {
            snap.enabled = !snap.enabled;
            let state = if snap.enabled { "ON" } else { "OFF" };
            notifications.success(format!("Snap to grid: {state} ({}m)", snap.grid_size));
        }
    }

    // Ctrl keybindings
    if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        if keyboard_input.just_pressed(KeyCode::KeyS) {
            if keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
                workspace_saver.save_to_dialog();
            } else {
                workspace_saver.save_to_default_file();
            }
        }

        // Ctrl+T align handled in handle_keyboard_extras

        if keyboard_input.just_pressed(KeyCode::KeyE) {
            sdf_dialog.show = true;
        }

        // TODO(luca) pop up a confirmation prompt if the current file is not saved, or create a
        // gui to switch between open workspaces
        if keyboard_input.just_pressed(KeyCode::KeyN) {
            new_workspace.write(CreateNewWorkspace);
        }

        if keyboard_input.just_pressed(KeyCode::KeyO) {
            workspace_loader.load_from_dialog();
        }

        if keyboard_input.just_pressed(KeyCode::KeyZ) {
            if keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
                redo_request.write(RedoRequest);
            } else {
                undo_request.write(UndoRequest);
            }
        }

        if keyboard_input.just_pressed(KeyCode::KeyY) {
            redo_request.write(RedoRequest);
        }
    }
}

/// Separate system for debug mode, align, graph view, and delete to stay within Bevy's 16-param limit.
fn handle_keyboard_extras(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut egui_context: EguiContexts,
    mut debug_mode: ResMut<DebugMode>,
    mut align_site: EventWriter<AlignSiteDrawings>,
    mut toggle_graph_view: EventWriter<ToggleNavGraphView>,
    current_workspace: Res<CurrentWorkspace>,
    primary_windows: Query<Entity, With<PrimaryWindow>>,
    mut notifications: ResMut<Notifications>,
    view_menu: Option<Res<ViewMenuItems>>,
    mut menu_items: Query<&mut MenuItem>,
    selection: Res<Selection>,
    mut multi: ResMut<MultiSelection>,
    mut delete: EventWriter<Delete>,
) {
    let Some(egui_context) = primary_windows
        .single()
        .ok()
        .and_then(|w| egui_context.try_ctx_for_entity_mut(w))
    else {
        return;
    };
    if egui_context.wants_keyboard_input() {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Delete)
        || keyboard_input.just_pressed(KeyCode::Backspace)
    {
        let all = multi.all_with_primary(&selection);
        if all.is_empty() {
            notifications.error("No entity selected to delete");
        } else {
            let count = all.len();
            for entity in all {
                delete.write(Delete::new(entity));
            }
            multi.clear();
            if count > 1 {
                notifications.success(format!("Deleted {count} entities"));
            }
        }
    }

    if keyboard_input.just_pressed(KeyCode::KeyD) {
        debug_mode.0 = !debug_mode.0;
        let state = if debug_mode.0 { "ON" } else { "OFF" };
        notifications.success(format!("Debug mode: {state}"));
    }

    if keyboard_input.just_pressed(KeyCode::F4) {
        toggle_graph_view.write(ToggleNavGraphView);
        // Sync the menu checkbox
        if let Some(view_menu) = &view_menu {
            if let Ok(mut item) = menu_items.get_mut(view_menu.graph_view) {
                if let Some(value) = MenuItem::checkbox_value_mut(&mut item) {
                    *value = !*value;
                }
            }
        }
        notifications.success("Graph View toggled");
    }

    if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        if keyboard_input.just_pressed(KeyCode::KeyT) {
            if let Some(site) = current_workspace.root {
                align_site.write(AlignSiteDrawings(site));
            }
        }
    }
}

/// Handles measurement tool toggle, properties copy/paste, and Escape for measurement cancel.
fn handle_keyboard_tools(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut egui_context: EguiContexts,
    primary_windows: Query<Entity, With<PrimaryWindow>>,
    mut measure_tool: ResMut<MeasureTool>,
    mut copy_events: EventWriter<CopyProperties>,
    mut paste_events: EventWriter<PasteProperties>,
    mut notifications: ResMut<Notifications>,
) {
    let Some(egui_context) = primary_windows
        .single()
        .ok()
        .and_then(|w| egui_context.try_ctx_for_entity_mut(w))
    else {
        return;
    };
    if egui_context.wants_keyboard_input() {
        return;
    }

    // [M] toggle measurement tool
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        measure_tool.toggle();
        let state = if measure_tool.active { "ON" } else { "OFF" };
        notifications.success(format!("Measure tool: {state}"));
    }

    // Escape cancels measurement in progress
    if keyboard_input.just_pressed(KeyCode::Escape) && measure_tool.active {
        if measure_tool.point_a.is_some() {
            measure_tool.reset();
        } else {
            measure_tool.active = false;
            notifications.success("Measure tool: OFF");
        }
    }

    // Ctrl+Shift+C: copy properties
    if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        && keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
        && keyboard_input.just_pressed(KeyCode::KeyC)
    {
        copy_events.write(CopyProperties);
    }

    // Ctrl+Shift+V: paste properties
    if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        && keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
        && keyboard_input.just_pressed(KeyCode::KeyV)
    {
        paste_events.write(PasteProperties);
    }
}

pub fn keyboard_just_pressed_stream(
    In(ContinuousService { key }): ContinuousServiceInput<(), (), StreamOf<KeyCode>>,
    mut orders: ContinuousQuery<(), (), StreamOf<KeyCode>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let Some(mut orders) = orders.get_mut(&key) else {
        return;
    };

    if orders.is_empty() {
        return;
    }

    for key_code in keyboard_input.get_just_pressed() {
        orders.for_each(|order| order.streams().send(*key_code));
    }
}
