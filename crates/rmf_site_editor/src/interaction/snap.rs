use bevy::prelude::*;

/// Controls snap-to-grid behavior for anchor and object placement/dragging.
#[derive(Resource)]
pub struct SnapToGrid {
    /// Whether snapping is currently enabled.
    pub enabled: bool,
    /// Grid cell size in meters.
    pub grid_size: f32,
}

impl Default for SnapToGrid {
    fn default() -> Self {
        Self {
            enabled: false,
            grid_size: 0.5,
        }
    }
}

impl SnapToGrid {
    /// Snap a value to the nearest grid increment, or return it unchanged if
    /// snapping is disabled.
    pub fn snap_value(&self, v: f32) -> f32 {
        if self.enabled {
            (v / self.grid_size).round() * self.grid_size
        } else {
            v
        }
    }

    /// Snap the X and Y components of a Vec3, leaving Z unchanged.
    pub fn snap_xy(&self, v: Vec3) -> Vec3 {
        if self.enabled {
            Vec3::new(self.snap_value(v.x), self.snap_value(v.y), v.z)
        } else {
            v
        }
    }

    /// Available grid size presets (in meters).
    pub const PRESETS: &[f32] = &[0.1, 0.25, 0.5, 1.0, 2.0, 5.0];
}

/// Set each frame by [`update_ortho_snap`]. True when Shift is held.
#[derive(Resource, Default)]
pub struct OrthoSnapActive(pub bool);

/// Reads the keyboard each frame and updates [`OrthoSnapActive`].
pub fn update_ortho_snap(keyboard: Res<ButtonInput<KeyCode>>, mut ortho: ResMut<OrthoSnapActive>) {
    ortho.0 = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
}
