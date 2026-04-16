use crate::{AppState, WorkspaceSaver, widgets::RenderUiSet};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowCloseRequested};
use bevy_egui::{EguiContexts, egui};

const BASE_TITLE: &str = "RMF Site Editor";

#[derive(Resource, Default)]
pub struct SiteChanged(pub bool);

#[derive(Resource, Default)]
pub struct ExitConfirmationDialog {
    pub visible: bool,
}

pub struct ExitConfirmationPlugin;

impl Plugin for ExitConfirmationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SiteChanged>()
            .init_resource::<ExitConfirmationDialog>()
            .add_systems(Update, handle_exit_requests)
            .add_systems(Update, update_window_title)
            .add_systems(Update, show_exit_confirmation_dialog.after(RenderUiSet));
    }
}

fn handle_exit_requests(
    mut exit_confirmation_dialog: ResMut<ExitConfirmationDialog>,
    mut close_events: EventReader<WindowCloseRequested>,
    mut app_exit: EventWriter<AppExit>,
    site_changed: Res<SiteChanged>,
    app_state: Res<State<AppState>>,
) {
    for _ in close_events.read() {
        if app_state.get() == &AppState::MainMenu {
            app_exit.write(AppExit::Success);
        }

        if site_changed.0 == true {
            exit_confirmation_dialog.visible = true;
        } else {
            app_exit.write(AppExit::Success);
        }
    }
}

fn update_window_title(
    site_changed: Res<SiteChanged>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !site_changed.is_changed() {
        return;
    }
    if let Ok(mut window) = windows.single_mut() {
        window.title = if site_changed.0 {
            format!("* {BASE_TITLE}")
        } else {
            BASE_TITLE.to_string()
        };
    }
}

fn show_exit_confirmation_dialog(
    mut contexts: EguiContexts,
    mut exit_confirmation_dialog: ResMut<ExitConfirmationDialog>,
    mut app_exit: EventWriter<AppExit>,
    mut workspace_saver: WorkspaceSaver,
) {
    if !exit_confirmation_dialog.visible {
        return;
    }
    egui::Window::new("Exit Confirmation")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(contexts.ctx_mut(), |ui| {
            ui.label("You have unsaved changes.");
            ui.label("");
            ui.label("What would you like to do?");
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Save and Exit").clicked() {
                    workspace_saver.save_to_default_file();
                    app_exit.write(AppExit::Success);
                }
                if ui.button("Exit without Saving").clicked() {
                    app_exit.write(AppExit::Success);
                }
                if ui.button("Cancel").clicked() {
                    exit_confirmation_dialog.visible = false;
                }
            });
        });
}
