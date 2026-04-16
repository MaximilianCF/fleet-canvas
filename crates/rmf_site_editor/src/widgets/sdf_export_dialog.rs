use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{AppState, WorkspaceSaver};

/// Resource controlling what to include in SDF export.
#[derive(Resource, Clone, Debug)]
pub struct SdfExportOptions {
    pub include_models: bool,
    pub include_doors: bool,
    pub include_lifts: bool,
    pub include_lights: bool,
    pub include_ros2_launch: bool,
}

impl Default for SdfExportOptions {
    fn default() -> Self {
        Self {
            include_models: true,
            include_doors: true,
            include_lifts: true,
            include_lights: true,
            include_ros2_launch: true,
        }
    }
}

/// Controls visibility of the export options dialog.
#[derive(Resource, Default)]
pub struct SdfExportDialogState {
    pub show: bool,
}

fn render_sdf_export_dialog(
    mut contexts: EguiContexts,
    mut dialog: ResMut<SdfExportDialogState>,
    mut options: ResMut<SdfExportOptions>,
    mut workspace_saver: WorkspaceSaver,
) {
    if !dialog.show {
        return;
    }

    let mut open = dialog.show;
    egui::Window::new("SDF Export Options")
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(contexts.ctx_mut(), |ui| {
            ui.add_space(4.0);
            ui.heading("Include in export:");
            ui.add_space(4.0);

            ui.checkbox(&mut options.include_models, "Models (non-static)");
            ui.checkbox(&mut options.include_doors, "Doors");
            ui.checkbox(&mut options.include_lifts, "Lifts");
            ui.checkbox(&mut options.include_lights, "Lights");

            ui.add_space(4.0);
            ui.separator();
            ui.checkbox(
                &mut options.include_ros2_launch,
                "Generate ROS 2 launch file",
            );

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Export...").clicked() {
                    dialog.show = false;
                    workspace_saver.export_sdf_to_dialog();
                }
                if ui.button("Cancel").clicked() {
                    dialog.show = false;
                }
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
            ui.label("Or launch a previously exported world:");
            #[cfg(not(target_arch = "wasm32"))]
            if ui
                .button("🚀 Launch in Gazebo...")
                .on_hover_text("Pick a launch.py from a previous SDF export")
                .clicked()
            {
                dialog.show = false;
                std::thread::spawn(|| {
                    let file = rfd::FileDialog::new()
                        .add_filter("ROS 2 launch", &["py"])
                        .set_title("Select the exported launch.py")
                        .pick_file();
                    if let Some(path) = file {
                        info!("Launching Gazebo with: {}", path.display());
                        match std::process::Command::new("ros2")
                            .arg("launch")
                            .arg(&path)
                            .spawn()
                        {
                            Ok(_) => info!("Gazebo process spawned"),
                            Err(e) => error!("Failed to launch Gazebo: {e}. Is ros2 on your PATH?"),
                        }
                    }
                });
            }
        });

    if !open {
        dialog.show = false;
    }
}

pub struct SdfExportDialogPlugin;

impl Plugin for SdfExportDialogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SdfExportOptions>()
            .init_resource::<SdfExportDialogState>()
            .add_systems(
                Update,
                render_sdf_export_dialog.run_if(AppState::in_displaying_mode()),
            );
    }
}
