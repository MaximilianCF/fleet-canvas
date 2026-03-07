use bevy::prelude::*;

use super::SnapToGrid;

/// Marker component for the snap grid overlay entity.
#[derive(Component)]
struct SnapGridMarker;

/// Resource controlling snap grid overlay visibility.
#[derive(Resource)]
pub struct SnapGridConfig {
    pub visible: bool,
    /// Number of grid lines in each direction from the origin.
    pub half_extent: i32,
}

impl Default for SnapGridConfig {
    fn default() -> Self {
        Self {
            visible: false,
            half_extent: 50,
        }
    }
}

fn spawn_snap_grid(mut commands: Commands) {
    commands.spawn((
        SnapGridMarker,
        Transform::from_xyz(0.0, 0.0, 0.001),
        Visibility::Hidden,
    ));
}

fn update_snap_grid(
    snap: Res<SnapToGrid>,
    config: Res<SnapGridConfig>,
    mut grid_query: Query<&mut Visibility, With<SnapGridMarker>>,
    mut gizmos: Gizmos,
) {
    let Ok(mut vis) = grid_query.single_mut() else {
        return;
    };

    if !config.visible {
        *vis = Visibility::Hidden;
        return;
    }
    *vis = Visibility::Inherited;

    let grid_size = snap.grid_size;
    let half = config.half_extent;
    let extent = half as f32 * grid_size;

    let minor_color = Color::srgba(0.4, 0.4, 0.4, 0.15);
    let major_color = Color::srgba(0.5, 0.5, 0.5, 0.3);
    let axis_x_color = Color::srgba(0.8, 0.2, 0.2, 0.5);
    let axis_y_color = Color::srgba(0.2, 0.8, 0.2, 0.5);

    let major_interval = if grid_size < 0.5 { 10 } else { 5 };

    for i in -half..=half {
        let offset = i as f32 * grid_size;

        let color_x = if i == 0 {
            axis_y_color
        } else if i % major_interval == 0 {
            major_color
        } else {
            minor_color
        };
        gizmos.line(
            Vec3::new(offset, -extent, 0.001),
            Vec3::new(offset, extent, 0.001),
            color_x,
        );

        let color_y = if i == 0 {
            axis_x_color
        } else if i % major_interval == 0 {
            major_color
        } else {
            minor_color
        };
        gizmos.line(
            Vec3::new(-extent, offset, 0.001),
            Vec3::new(extent, offset, 0.001),
            color_y,
        );
    }
}

pub struct SnapGridPlugin;

impl Plugin for SnapGridPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SnapGridConfig>()
            .add_systems(Startup, spawn_snap_grid)
            .add_systems(Update, update_snap_grid);
    }
}
