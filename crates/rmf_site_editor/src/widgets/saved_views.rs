use bevy::{
    ecs::{
        hierarchy::ChildOf,
        system::{SystemParam, SystemState},
    },
    prelude::*,
};
use bevy_egui::egui::{self, Ui};
use rmf_site_camera::{active_camera_maybe, resources::CameraConfig, ActiveCameraQuery};
use rmf_site_egui::*;
use rmf_site_format::{NameInSite, Pose, UserCameraPoseMarker};

use crate::site::CurrentLevel;

#[derive(Resource, Default)]
struct SavedViewsState {
    new_view_name: String,
    renaming: Option<Entity>,
    rename_buf: String,
}

#[derive(Default)]
pub struct SavedViewsPlugin;

impl Plugin for SavedViewsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SavedViewsState>()
            .add_event::<SaveCurrentView>()
            .add_plugins(PropertiesTilePlugin::<SavedViews>::new().tab("Site"))
            .add_systems(Update, handle_save_current_view);
    }
}

#[derive(SystemParam)]
pub struct SavedViews<'w, 's> {
    views: Query<'w, 's, (Entity, &'static NameInSite, &'static Pose), With<UserCameraPoseMarker>>,
    children_q: Query<'w, 's, &'static Children>,
    current_level: Res<'w, CurrentLevel>,
    active_cam: ActiveCameraQuery<'w, 's>,
    state: ResMut<'w, SavedViewsState>,
}

enum ViewAction {
    GoTo(Pose),
    Delete(Entity),
    StartRename(Entity, String),
    FinishRename(Entity, String),
    Save(String),
}

impl<'w, 's> WidgetSystem<Tile> for SavedViews<'w, 's> {
    fn show(_: Tile, ui: &mut Ui, state: &mut SystemState<Self>, world: &mut World) {
        let mut params = state.get_mut(world);

        let level = params.current_level.0;
        let Some(level) = level else {
            ui.label("No level loaded");
            return;
        };

        // Collect views for current level
        let level_views: Vec<(Entity, String, Pose)> =
            if let Ok(children) = params.children_q.get(level) {
                children
                    .iter()
                    .filter_map(|c| params.views.get(c).ok())
                    .map(|(e, name, pose)| (e, name.0.clone(), *pose))
                    .collect()
            } else {
                Vec::new()
            };

        let renaming = params.state.renaming;
        let mut actions: Vec<ViewAction> = Vec::new();

        ui.heading("Saved Views");
        ui.add_space(4.0);

        // Save current view row
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut params.state.new_view_name);
            let name_empty = params.state.new_view_name.trim().is_empty();
            if ui
                .add_enabled(!name_empty, egui::Button::new("Save"))
                .on_hover_text("Save current camera position")
                .clicked()
            {
                actions.push(ViewAction::Save(
                    params.state.new_view_name.trim().to_string(),
                ));
                params.state.new_view_name.clear();
            }
        });

        ui.add_space(4.0);

        // View list
        if level_views.is_empty() {
            ui.label(egui::RichText::new("No saved views for this level").weak());
        } else {
            for (entity, name, pose) in &level_views {
                ui.horizontal(|ui| {
                    if renaming == Some(*entity) {
                        let mut buf = params.state.rename_buf.clone();
                        let response = ui.text_edit_singleline(&mut buf);
                        params.state.rename_buf = buf.clone();
                        if response.lost_focus() || ui.small_button("OK").clicked() {
                            actions.push(ViewAction::FinishRename(*entity, buf));
                        }
                    } else {
                        if ui
                            .small_button("Go")
                            .on_hover_text("Move camera to this view")
                            .clicked()
                        {
                            actions.push(ViewAction::GoTo(*pose));
                        }
                        ui.label(&**name);
                        if ui.small_button("Rename").clicked() {
                            actions.push(ViewAction::StartRename(*entity, name.clone()));
                        }
                        if ui
                            .small_button("Del")
                            .on_hover_text("Delete this view")
                            .clicked()
                        {
                            actions.push(ViewAction::Delete(*entity));
                        }
                    }
                });
            }
        }

        // Get camera entity before dropping params
        let active_camera = active_camera_maybe(&params.active_cam).ok();

        drop(params);

        // Apply actions
        for action in actions {
            match action {
                ViewAction::GoTo(pose) => {
                    if let Some(cam) = active_camera {
                        if let Some(mut tf) = world.get_mut::<Transform>(cam) {
                            *tf = pose.transform();
                        }
                        let mut translation = pose.transform().translation;
                        translation.x += 10.0;
                        translation.y += 10.0;
                        translation.z = 0.0;
                        world.resource_mut::<CameraConfig>().orbit_center = Some(translation);
                    }
                }
                ViewAction::Delete(entity) => {
                    world.commands().entity(entity).despawn();
                }
                ViewAction::StartRename(entity, name) => {
                    let mut s = world.resource_mut::<SavedViewsState>();
                    s.renaming = Some(entity);
                    s.rename_buf = name;
                }
                ViewAction::FinishRename(entity, new_name) => {
                    if let Some(mut name) = world.get_mut::<NameInSite>(entity) {
                        name.0 = new_name;
                    }
                    world.resource_mut::<SavedViewsState>().renaming = None;
                }
                ViewAction::Save(name) => {
                    world.send_event(SaveCurrentView { name });
                }
            }
        }
    }
}

/// Event to save the current camera pose as a named view.
#[derive(Event)]
pub struct SaveCurrentView {
    pub name: String,
}

fn handle_save_current_view(
    mut events: EventReader<SaveCurrentView>,
    active_cam: ActiveCameraQuery,
    transforms: Query<&Transform>,
    current_level: Res<CurrentLevel>,
    mut commands: Commands,
    mut notifications: ResMut<crate::widgets::Notifications>,
) {
    for event in events.read() {
        let Some(level) = current_level.0 else {
            notifications.error("Cannot save view: no level loaded");
            continue;
        };
        let Ok(cam) = active_camera_maybe(&active_cam) else {
            continue;
        };
        let Ok(tf) = transforms.get(cam) else {
            continue;
        };

        let pose = Pose::from(*tf);
        commands.spawn((
            pose,
            NameInSite(event.name.clone()),
            UserCameraPoseMarker,
            ChildOf(level),
        ));
        notifications.success(format!("View \"{}\" saved", event.name));
    }
}
