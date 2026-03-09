use bevy::prelude::*;

use crate::widgets::{CursorWorldPosition, Notifications};

/// State of the measurement tool.
#[derive(Resource, Default)]
pub struct MeasureTool {
    pub active: bool,
    pub point_a: Option<Vec3>,
    pub last_result: Option<MeasureResult>,
}

pub struct MeasureResult {
    pub from: Vec3,
    pub to: Vec3,
    pub distance: f32,
}

impl MeasureTool {
    pub fn toggle(&mut self) {
        self.active = !self.active;
        if !self.active {
            self.point_a = None;
        }
    }

    pub fn reset(&mut self) {
        self.point_a = None;
    }
}

/// Draw the measurement line and handle clicks.
fn measure_tool_system(
    mut tool: ResMut<MeasureTool>,
    cursor_pos: Res<CursorWorldPosition>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut gizmos: Gizmos,
    mut notifications: ResMut<Notifications>,
) {
    if !tool.active {
        return;
    }

    let Some(cursor) = cursor_pos.position else {
        return;
    };

    let cursor_2d = Vec3::new(cursor.x, cursor.y, 0.0);

    if let Some(point_a) = tool.point_a {
        // Draw line from point A to cursor
        let color = Color::srgba(0.0, 1.0, 1.0, 0.9);
        gizmos.line(point_a, cursor_2d, color);

        // Draw small crosses at endpoints
        let cross = 0.15;
        gizmos.line(
            point_a + Vec3::new(-cross, 0.0, 0.0),
            point_a + Vec3::new(cross, 0.0, 0.0),
            color,
        );
        gizmos.line(
            point_a + Vec3::new(0.0, -cross, 0.0),
            point_a + Vec3::new(0.0, cross, 0.0),
            color,
        );

        let dist = (cursor_2d - point_a).length();

        // Show distance near cursor via a small midpoint indicator
        let mid = (point_a + cursor_2d) / 2.0;
        let perp = Vec3::new(-(cursor_2d.y - point_a.y), cursor_2d.x - point_a.x, 0.0)
            .normalize_or_zero()
            * 0.3;
        gizmos.line(mid, mid + perp, color);

        if mouse.just_pressed(MouseButton::Left) {
            // Second click: finalize measurement
            tool.last_result = Some(MeasureResult {
                from: point_a,
                to: cursor_2d,
                distance: dist,
            });
            notifications.success(format!("Distance: {:.3}m", dist));
            tool.point_a = None;
        }
    } else if mouse.just_pressed(MouseButton::Left) {
        // First click: set point A
        tool.point_a = Some(cursor_2d);
    }

    // Draw last result if it exists (persistent line)
    if let Some(ref result) = tool.last_result {
        let color = Color::srgba(0.0, 0.8, 0.8, 0.5);
        gizmos.line(result.from, result.to, color);
    }
}

/// Render the measure tool indicator in the status bar area.
/// This is handled by checking MeasureTool.active in the status bar.

pub struct MeasureToolPlugin;

impl Plugin for MeasureToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeasureTool>()
            .add_systems(Update, measure_tool_system);
    }
}
