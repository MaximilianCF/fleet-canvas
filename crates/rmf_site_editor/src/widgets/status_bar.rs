use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use rmf_site_camera::resources::ProjectionMode;

use crate::{
    interaction::{SnapGridConfig, SnapToGrid},
    site::NavGraphViewMode,
    AppState,
};

use super::RenderUiSet;

/// Tracks the cursor's world-space position for display in the status bar.
#[derive(Resource, Default)]
pub struct CursorWorldPosition {
    pub position: Option<Vec3>,
}

fn update_cursor_world_position(
    mut cursor_pos: ResMut<CursorWorldPosition>,
    intersect: crate::interaction::IntersectGroundPlaneParams,
    cursor_moved: EventReader<CursorMoved>,
) {
    if cursor_moved.is_empty() {
        return;
    }
    cursor_pos.position = intersect
        .ground_plane_intersection()
        .map(|tf| tf.translation);
}

fn render_status_bar(
    mut contexts: EguiContexts,
    cursor_pos: Res<CursorWorldPosition>,
    snap: Res<SnapToGrid>,
    projection_mode: Res<ProjectionMode>,
    grid_config: Res<SnapGridConfig>,
    graph_view: Res<NavGraphViewMode>,
    measure_tool: Res<crate::interaction::MeasureTool>,
) {
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(22.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal_centered(|ui| {
                // Cursor coordinates
                if let Some(pos) = cursor_pos.position {
                    ui.label(
                        egui::RichText::new(format!("X: {:.2}  Y: {:.2}", pos.x, pos.y))
                            .monospace()
                            .small(),
                    );
                } else {
                    ui.label(egui::RichText::new("X: ---  Y: ---").monospace().small());
                }

                ui.separator();

                // Projection mode indicator
                let mode_text = match *projection_mode {
                    ProjectionMode::Orthographic => "Ortho",
                    ProjectionMode::Perspective => "Persp",
                };
                ui.label(
                    egui::RichText::new(mode_text)
                        .small()
                        .color(egui::Color32::from_rgb(140, 180, 220)),
                );

                ui.separator();

                // Snap indicator
                let snap_text = if snap.enabled {
                    format!("Snap: {}m", snap.grid_size)
                } else {
                    "Snap: OFF".to_string()
                };
                let snap_color = if snap.enabled {
                    egui::Color32::from_rgb(100, 200, 120)
                } else {
                    egui::Color32::from_rgb(160, 160, 160)
                };
                ui.label(egui::RichText::new(snap_text).small().color(snap_color));

                ui.separator();

                // Grid indicator
                let grid_text = if grid_config.visible {
                    "Grid: ON"
                } else {
                    "Grid: OFF"
                };
                let grid_color = if grid_config.visible {
                    egui::Color32::from_rgb(140, 180, 220)
                } else {
                    egui::Color32::from_rgb(160, 160, 160)
                };
                ui.label(egui::RichText::new(grid_text).small().color(grid_color));

                ui.separator();

                // Graph view indicator
                if graph_view.active {
                    ui.label(
                        egui::RichText::new("Graph View")
                            .small()
                            .color(egui::Color32::from_rgb(255, 180, 80)),
                    );
                    ui.separator();
                }

                // Measure tool indicator
                if measure_tool.active {
                    let measure_text = if measure_tool.point_a.is_some() {
                        "Measure: click 2nd point"
                    } else {
                        "Measure: click 1st point"
                    };
                    ui.label(
                        egui::RichText::new(measure_text)
                            .small()
                            .color(egui::Color32::from_rgb(0, 220, 220)),
                    );
                    if let Some(ref result) = measure_tool.last_result {
                        ui.label(
                            egui::RichText::new(format!("({:.3}m)", result.distance))
                                .small()
                                .color(egui::Color32::from_rgb(0, 200, 200)),
                        );
                    }
                    ui.separator();
                }

                ui.label(
                    egui::RichText::new(
                        "[G] snap  [M] measure  [F2] ortho  [F3] persp  [F4] graph  [Del] delete",
                    )
                    .small()
                    .weak(),
                );
            });
        });
}

pub struct StatusBarPlugin;

impl Plugin for StatusBarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldPosition>()
            .add_systems(
                Update,
                update_cursor_world_position.run_if(AppState::in_displaying_mode()),
            )
            .add_systems(
                Update,
                render_status_bar
                    .in_set(RenderUiSet)
                    .run_if(AppState::in_displaying_mode()),
            );
    }
}
