/*
 * Copyright (C) 2023 Open Source Robotics Foundation
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

use crate::interaction::{CategoryVisibility, SetCategoryVisibility, SnapGridConfig};
use crate::site::{
    CollisionMeshMarker, DoorMarker, FiducialMarker, FloorMarker, LaneMarker, LiftCabin,
    LiftCabinDoorMarker, LocationTags, MeasurementMarker, VisualMeshMarker, WallMarker,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use rmf_site_camera::resources::ProjectionMode;
use rmf_site_egui::*;

/// When active, hides all geometry and shows only nav graph elements (lanes + locations).
#[derive(Resource, Default)]
pub struct NavGraphViewMode {
    pub active: bool,
    /// Saved visibility states to restore when leaving graph view.
    saved: Option<SavedVisibility>,
}

#[derive(Clone)]
struct SavedVisibility {
    doors: bool,
    floors: bool,
    lifts: bool,
    fiducials: bool,
    measurements: bool,
    collisions: bool,
    visuals: bool,
    walls: bool,
}

#[derive(Event)]
pub struct ToggleNavGraphView;

#[derive(SystemParam)]
struct VisibilityEvents<'w> {
    doors: EventWriter<'w, SetCategoryVisibility<DoorMarker>>,
    floors: EventWriter<'w, SetCategoryVisibility<FloorMarker>>,
    lanes: EventWriter<'w, SetCategoryVisibility<LaneMarker>>,
    lift_cabins: EventWriter<'w, SetCategoryVisibility<LiftCabin<Entity>>>,
    lift_cabin_doors: EventWriter<'w, SetCategoryVisibility<LiftCabinDoorMarker>>,
    locations: EventWriter<'w, SetCategoryVisibility<LocationTags>>,
    fiducials: EventWriter<'w, SetCategoryVisibility<FiducialMarker>>,
    measurements: EventWriter<'w, SetCategoryVisibility<MeasurementMarker>>,
    walls: EventWriter<'w, SetCategoryVisibility<WallMarker>>,
    visuals: EventWriter<'w, SetCategoryVisibility<VisualMeshMarker>>,
    collisions: EventWriter<'w, SetCategoryVisibility<CollisionMeshMarker>>,
}

#[derive(Default)]
pub struct ViewMenuPlugin;

#[derive(Resource)]
pub struct ViewMenuItems {
    orthographic: Entity,
    perspective: Entity,
    reference_grid: Entity,
    pub graph_view: Entity,
    doors: Entity,
    floors: Entity,
    lanes: Entity,
    lifts: Entity,
    locations: Entity,
    fiducials: Entity,
    measurements: Entity,
    collisions: Entity,
    visuals: Entity,
    walls: Entity,
}

impl FromWorld for ViewMenuItems {
    fn from_world(world: &mut World) -> Self {
        let view_header = world.resource::<ViewMenu>().get();

        let orthographic = world
            .spawn(MenuItem::Text(
                TextMenuItem::new("Orthographic").shortcut("F2"),
            ))
            .insert(ChildOf(view_header))
            .id();
        let perspective = world
            .spawn(MenuItem::Text(
                TextMenuItem::new("Perspective").shortcut("F3"),
            ))
            .insert(ChildOf(view_header))
            .id();
        let grid_visible = world.resource::<SnapGridConfig>().visible;
        let reference_grid = world
            .spawn(MenuItem::CheckBox("Snap Grid".to_string(), grid_visible))
            .insert(ChildOf(view_header))
            .id();
        let graph_view = world
            .spawn(MenuItem::CheckBox("Graph View".to_string(), false))
            .insert(ChildOf(view_header))
            .id();
        world
            .spawn(MenuItem::Separator)
            .insert(ChildOf(view_header));

        let default_visibility = world.resource::<CategoryVisibility<DoorMarker>>();
        let doors = world
            .spawn(MenuItem::CheckBox(
                "Doors".to_string(),
                default_visibility.0,
            ))
            .insert(ChildOf(view_header))
            .id();
        let default_visibility = world.resource::<CategoryVisibility<FloorMarker>>();
        let floors = world
            .spawn(MenuItem::CheckBox(
                "Floors".to_string(),
                default_visibility.0,
            ))
            .insert(ChildOf(view_header))
            .id();
        let default_visibility = world.resource::<CategoryVisibility<LaneMarker>>();
        let lanes = world
            .spawn(MenuItem::CheckBox(
                "Lanes".to_string(),
                default_visibility.0,
            ))
            .insert(ChildOf(view_header))
            .id();
        let default_visibility = world.resource::<CategoryVisibility<LiftCabin<Entity>>>();
        let lifts = world
            .spawn(MenuItem::CheckBox(
                "Lifts".to_string(),
                default_visibility.0,
            ))
            .insert(ChildOf(view_header))
            .id();
        let default_visibility = world.resource::<CategoryVisibility<LocationTags>>();
        let locations = world
            .spawn(MenuItem::CheckBox(
                "Locations".to_string(),
                default_visibility.0,
            ))
            .insert(ChildOf(view_header))
            .id();
        let default_visibility = world.resource::<CategoryVisibility<FiducialMarker>>();
        let fiducials = world
            .spawn(MenuItem::CheckBox(
                "Fiducials".to_string(),
                default_visibility.0,
            ))
            .insert(ChildOf(view_header))
            .id();
        let default_visibility = world.resource::<CategoryVisibility<MeasurementMarker>>();
        let measurements = world
            .spawn(MenuItem::CheckBox(
                "Measurements".to_string(),
                default_visibility.0,
            ))
            .insert(ChildOf(view_header))
            .id();
        let default_visibility = world.resource::<CategoryVisibility<CollisionMeshMarker>>();
        let collisions = world
            .spawn(MenuItem::CheckBox(
                "Collision meshes".to_string(),
                default_visibility.0,
            ))
            .insert(ChildOf(view_header))
            .id();
        let default_visibility = world.resource::<CategoryVisibility<VisualMeshMarker>>();
        let visuals = world
            .spawn(MenuItem::CheckBox(
                "Visual meshes".to_string(),
                default_visibility.0,
            ))
            .insert(ChildOf(view_header))
            .id();
        let default_visibility = world.resource::<CategoryVisibility<WallMarker>>();
        let walls = world
            .spawn(MenuItem::CheckBox(
                "Walls".to_string(),
                default_visibility.0,
            ))
            .insert(ChildOf(view_header))
            .id();

        ViewMenuItems {
            orthographic,
            perspective,
            reference_grid,
            graph_view,
            doors,
            floors,
            lanes,
            lifts,
            locations,
            fiducials,
            measurements,
            collisions,
            visuals,
            walls,
        }
    }
}

fn handle_view_menu_events(
    mut menu_events: EventReader<MenuEvent>,
    view_menu: Res<ViewMenuItems>,
    mut menu_items: Query<&mut MenuItem>,
    mut events: VisibilityEvents,
    mut projection_mode: ResMut<ProjectionMode>,
    mut grid_config: ResMut<SnapGridConfig>,
    mut toggle_graph_view: EventWriter<ToggleNavGraphView>,
) {
    let mut toggle = |entity| {
        let mut menu = menu_items.get_mut(entity).unwrap();
        let value = menu.checkbox_value_mut().unwrap();
        *value = !*value;
        *value
    };
    for event in menu_events.read() {
        if event.clicked() && event.source() == view_menu.orthographic {
            *projection_mode = ProjectionMode::Orthographic;
        } else if event.clicked() && event.source() == view_menu.perspective {
            *projection_mode = ProjectionMode::Perspective;
        } else if event.clicked() && event.source() == view_menu.reference_grid {
            grid_config.visible = toggle(event.source());
        } else if event.clicked() && event.source() == view_menu.graph_view {
            toggle(event.source());
            toggle_graph_view.write(ToggleNavGraphView);
        } else if event.clicked() && event.source() == view_menu.doors {
            events.doors.write(toggle(event.source()).into());
        } else if event.clicked() && event.source() == view_menu.floors {
            events.floors.write(toggle(event.source()).into());
        } else if event.clicked() && event.source() == view_menu.lanes {
            events.lanes.write(toggle(event.source()).into());
        } else if event.clicked() && event.source() == view_menu.lifts {
            let value = toggle(event.source());
            events.lift_cabins.write(value.into());
            events.lift_cabin_doors.write(value.into());
        } else if event.clicked() && event.source() == view_menu.locations {
            events.locations.write(toggle(event.source()).into());
        } else if event.clicked() && event.source() == view_menu.fiducials {
            events.fiducials.write(toggle(event.source()).into());
        } else if event.clicked() && event.source() == view_menu.measurements {
            events.measurements.write(toggle(event.source()).into());
        } else if event.clicked() && event.source() == view_menu.collisions {
            events.collisions.write(toggle(event.source()).into());
        } else if event.clicked() && event.source() == view_menu.visuals {
            events.visuals.write(toggle(event.source()).into());
        } else if event.clicked() && event.source() == view_menu.walls {
            events.walls.write(toggle(event.source()).into());
        }
    }
}

#[derive(SystemParam)]
struct GraphViewVisibilityState<'w> {
    cat_doors: Res<'w, CategoryVisibility<DoorMarker>>,
    cat_floors: Res<'w, CategoryVisibility<FloorMarker>>,
    cat_lifts: Res<'w, CategoryVisibility<LiftCabin<Entity>>>,
    cat_fiducials: Res<'w, CategoryVisibility<FiducialMarker>>,
    cat_measurements: Res<'w, CategoryVisibility<MeasurementMarker>>,
    cat_collisions: Res<'w, CategoryVisibility<CollisionMeshMarker>>,
    cat_visuals: Res<'w, CategoryVisibility<VisualMeshMarker>>,
    cat_walls: Res<'w, CategoryVisibility<WallMarker>>,
}

#[derive(SystemParam)]
struct GraphViewVisibilityWriters<'w> {
    doors: EventWriter<'w, SetCategoryVisibility<DoorMarker>>,
    floors: EventWriter<'w, SetCategoryVisibility<FloorMarker>>,
    lifts: EventWriter<'w, SetCategoryVisibility<LiftCabin<Entity>>>,
    lift_doors: EventWriter<'w, SetCategoryVisibility<LiftCabinDoorMarker>>,
    fiducials: EventWriter<'w, SetCategoryVisibility<FiducialMarker>>,
    measurements: EventWriter<'w, SetCategoryVisibility<MeasurementMarker>>,
    collisions: EventWriter<'w, SetCategoryVisibility<CollisionMeshMarker>>,
    visuals: EventWriter<'w, SetCategoryVisibility<VisualMeshMarker>>,
    walls: EventWriter<'w, SetCategoryVisibility<WallMarker>>,
    lanes: EventWriter<'w, SetCategoryVisibility<LaneMarker>>,
    locations: EventWriter<'w, SetCategoryVisibility<LocationTags>>,
}

fn handle_toggle_nav_graph_view(
    mut toggle_events: EventReader<ToggleNavGraphView>,
    mut graph_view: ResMut<NavGraphViewMode>,
    state: GraphViewVisibilityState,
    mut writers: GraphViewVisibilityWriters,
) {
    for _ in toggle_events.read() {
        graph_view.active = !graph_view.active;

        if graph_view.active {
            // Save current visibility and hide geometry
            graph_view.saved = Some(SavedVisibility {
                doors: state.cat_doors.0,
                floors: state.cat_floors.0,
                lifts: state.cat_lifts.0,
                fiducials: state.cat_fiducials.0,
                measurements: state.cat_measurements.0,
                collisions: state.cat_collisions.0,
                visuals: state.cat_visuals.0,
                walls: state.cat_walls.0,
            });

            writers.doors.write(false.into());
            writers.floors.write(false.into());
            writers.lifts.write(false.into());
            writers.lift_doors.write(false.into());
            writers.fiducials.write(false.into());
            writers.measurements.write(false.into());
            writers.collisions.write(false.into());
            writers.visuals.write(false.into());
            writers.walls.write(false.into());

            // Ensure nav elements are visible
            writers.lanes.write(true.into());
            writers.locations.write(true.into());
        } else if let Some(saved) = graph_view.saved.take() {
            // Restore previous visibility
            writers.doors.write(saved.doors.into());
            writers.floors.write(saved.floors.into());
            writers.lifts.write(saved.lifts.into());
            writers.lift_doors.write(saved.lifts.into());
            writers.fiducials.write(saved.fiducials.into());
            writers.measurements.write(saved.measurements.into());
            writers.collisions.write(saved.collisions.into());
            writers.visuals.write(saved.visuals.into());
            writers.walls.write(saved.walls.into());
        }
    }
}

/// Draws a legend overlay when graph view is active, showing color codes for each element type.
fn render_graph_view_legend(
    graph_view: Res<NavGraphViewMode>,
    mut contexts: bevy_egui::EguiContexts,
) {
    if !graph_view.active {
        return;
    }

    let ctx = contexts.ctx_mut();
    bevy_egui::egui::Window::new("Graph Legend")
        .anchor(
            bevy_egui::egui::Align2::LEFT_TOP,
            bevy_egui::egui::vec2(10.0, 70.0),
        )
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .frame(bevy_egui::egui::Frame {
            fill: bevy_egui::egui::Color32::from_rgba_premultiplied(30, 30, 40, 200),
            inner_margin: bevy_egui::egui::Margin::same(8),
            corner_radius: bevy_egui::egui::CornerRadius::same(4),
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.label(
                bevy_egui::egui::RichText::new("Graph View")
                    .strong()
                    .size(12.0)
                    .color(bevy_egui::egui::Color32::from_rgb(255, 180, 80)),
            );
            ui.add_space(4.0);

            let entries = [
                (
                    "Lane (Robot)",
                    bevy_egui::egui::Color32::from_rgb(80, 140, 220),
                ),
                (
                    "Lane (Human)",
                    bevy_egui::egui::Color32::from_rgb(230, 160, 50),
                ),
                (
                    "Location",
                    bevy_egui::egui::Color32::from_rgb(100, 200, 100),
                ),
                ("Charger", bevy_egui::egui::Color32::from_rgb(255, 220, 50)),
                (
                    "Parking Spot",
                    bevy_egui::egui::Color32::from_rgb(80, 180, 255),
                ),
                (
                    "Holding Point",
                    bevy_egui::egui::Color32::from_rgb(200, 120, 255),
                ),
            ];

            for (label, color) in entries {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(
                        bevy_egui::egui::vec2(10.0, 10.0),
                        bevy_egui::egui::Sense::hover(),
                    );
                    ui.painter().circle_filled(rect.center(), 4.0, color);
                    ui.label(
                        bevy_egui::egui::RichText::new(label)
                            .size(11.0)
                            .color(bevy_egui::egui::Color32::from_rgb(200, 200, 210)),
                    );
                });
            }

            ui.add_space(2.0);
            ui.label(
                bevy_egui::egui::RichText::new("[F4] toggle")
                    .size(10.0)
                    .color(bevy_egui::egui::Color32::from_rgb(120, 120, 130)),
            );
        });
}

impl Plugin for ViewMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ViewMenuItems>()
            .init_resource::<NavGraphViewMode>()
            .add_event::<ToggleNavGraphView>()
            .add_systems(
                Update,
                (
                    handle_view_menu_events,
                    handle_toggle_nav_graph_view,
                    render_graph_view_legend,
                ),
            );
    }
}
