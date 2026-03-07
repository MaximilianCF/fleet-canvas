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

impl Plugin for ViewMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ViewMenuItems>()
            .add_systems(Update, handle_view_menu_events);
    }
}
