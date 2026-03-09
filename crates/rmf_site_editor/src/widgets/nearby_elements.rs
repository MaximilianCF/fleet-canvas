use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use bevy_egui::egui::{self, Ui};
use rmf_site_egui::*;
use rmf_site_format::{Category, NameInSite};
use rmf_site_picking::{Select, Selection};

use crate::widgets::CursorWorldPosition;

const MAX_NEARBY: usize = 15;
const DEFAULT_RADIUS: f32 = 10.0;

#[derive(Resource)]
struct NearbyState {
    radius: f32,
}

impl Default for NearbyState {
    fn default() -> Self {
        Self {
            radius: DEFAULT_RADIUS,
        }
    }
}

#[derive(Default)]
pub struct NearbyElementsPlugin;

impl Plugin for NearbyElementsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NearbyState>()
            .add_plugins(PropertiesTilePlugin::<NearbyElements>::new().tab("Inspect"));
    }
}

#[derive(SystemParam)]
pub struct NearbyElements<'w, 's> {
    entities: Query<
        'w,
        's,
        (
            Entity,
            &'static NameInSite,
            &'static GlobalTransform,
            Option<&'static Category>,
        ),
    >,
    cursor: Res<'w, CursorWorldPosition>,
    selection: Res<'w, Selection>,
    select: EventWriter<'w, Select>,
    state: ResMut<'w, NearbyState>,
}

impl<'w, 's> WidgetSystem<Tile> for NearbyElements<'w, 's> {
    fn show(_: Tile, ui: &mut Ui, state: &mut SystemState<Self>, world: &mut World) {
        let mut params = state.get_mut(world);

        let Some(cursor_pos) = params.cursor.position else {
            ui.label(egui::RichText::new("Move cursor over the scene").weak());
            return;
        };

        ui.horizontal(|ui| {
            ui.label("Nearby (radius:");
            ui.add(
                egui::DragValue::new(&mut params.state.radius)
                    .range(1.0..=100.0)
                    .speed(0.5)
                    .suffix("m"),
            );
            ui.label(")");
        });

        let radius = params.state.radius;
        let cursor_2d = Vec2::new(cursor_pos.x, cursor_pos.y);

        // Collect nearby entities sorted by distance
        let mut nearby: Vec<(Entity, String, String, f32)> = params
            .entities
            .iter()
            .filter_map(|(e, name, gtf, cat)| {
                let pos = gtf.translation();
                let pos_2d = Vec2::new(pos.x, pos.y);
                let dist = cursor_2d.distance(pos_2d);
                if dist <= radius {
                    let cat_label = cat.map(|c| c.label()).unwrap_or("").to_string();
                    Some((e, name.0.clone(), cat_label, dist))
                } else {
                    None
                }
            })
            .collect();

        nearby.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));
        let total = nearby.len();
        nearby.truncate(MAX_NEARBY);

        if nearby.is_empty() {
            ui.label(egui::RichText::new("No named elements nearby").weak());
            return;
        }

        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(250.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for (entity, name, cat_label, dist) in &nearby {
                    let is_selected = params.selection.0.is_some_and(|s| s == *entity);
                    let label = format!("{} [{}] ({:.1}m)", name, cat_label, dist);
                    if ui.selectable_label(is_selected, &label).clicked() {
                        params.select.write(Select::new(Some(*entity)));
                    }
                }
            });

        if total > MAX_NEARBY {
            ui.label(
                egui::RichText::new(format!("... and {} more", total - MAX_NEARBY))
                    .small()
                    .weak(),
            );
        }
    }
}
