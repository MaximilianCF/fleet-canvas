use std::collections::HashSet;

use bevy::prelude::*;
use rmf_site_picking::{Select, Selected, Selection};

/// Resource tracking entities selected via Ctrl+Click (additive selection).
/// The primary `Selection` resource continues to track the "active" entity
/// for inspector purposes.
#[derive(Resource, Default, Debug)]
pub struct MultiSelection {
    pub entities: HashSet<Entity>,
}

impl MultiSelection {
    pub fn toggle(&mut self, entity: Entity) {
        if !self.entities.remove(&entity) {
            self.entities.insert(entity);
        }
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn count(&self) -> usize {
        self.entities.len()
    }

    /// Returns all selected entities (multi-selection + primary selection).
    pub fn all_with_primary(&self, primary: &Selection) -> Vec<Entity> {
        let mut all: Vec<Entity> = self.entities.iter().copied().collect();
        if let Some(primary) = primary.0 {
            if !self.entities.contains(&primary) {
                all.push(primary);
            }
        }
        all
    }
}

/// System that intercepts Select events when Ctrl is held to do additive selection.
pub fn multi_select_on_ctrl_click(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut select_events: EventReader<Select>,
    mut multi: ResMut<MultiSelection>,
    mut selected_q: Query<&mut Selected>,
    selection: Res<Selection>,
) {
    let ctrl = keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

    for event in select_events.read() {
        if !ctrl {
            // Normal click: clear multi-selection visual cues
            for entity in multi.entities.drain() {
                if let Ok(mut sel) = selected_q.get_mut(entity) {
                    sel.support_selected.remove(&Entity::PLACEHOLDER);
                }
            }
            continue;
        }

        // Ctrl+Click: toggle entity in multi-selection
        if let Some(candidate) = event.0 {
            let entity = candidate.candidate;
            let was_in = multi.entities.contains(&entity);
            multi.toggle(entity);

            if let Ok(mut sel) = selected_q.get_mut(entity) {
                if was_in {
                    sel.support_selected.remove(&Entity::PLACEHOLDER);
                } else {
                    // Use PLACEHOLDER as the "multi-select" marker
                    sel.support_selected.insert(Entity::PLACEHOLDER);
                }
            }

            // Also keep the primary selection highlighted if it exists
            if let Some(primary) = selection.0 {
                if primary != entity {
                    multi.entities.insert(primary);
                    if let Ok(mut sel) = selected_q.get_mut(primary) {
                        sel.support_selected.insert(Entity::PLACEHOLDER);
                    }
                }
            }
        }
    }
}

/// Clean up multi-selection when entities are despawned.
pub fn cleanup_despawned_multi_selection(
    mut multi: ResMut<MultiSelection>,
    existing: Query<Entity>,
) {
    multi.entities.retain(|e| existing.get(*e).is_ok());
}

pub struct MultiSelectPlugin;

impl Plugin for MultiSelectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MultiSelection>().add_systems(
            Update,
            (
                multi_select_on_ctrl_click.after(rmf_site_picking::SelectionServiceStages::Select),
                cleanup_despawned_multi_selection,
            ),
        );
    }
}
