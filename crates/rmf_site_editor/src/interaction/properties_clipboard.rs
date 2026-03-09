use bevy::prelude::*;
use rmf_site_format::{IsStatic, NameInSite, Pose, Scale};
use rmf_site_picking::Selection;

use crate::site::Change;
use crate::widgets::Notifications;

/// Clipboard for entity properties. Stores copies of common components.
#[derive(Resource, Default)]
pub struct PropertiesClipboard {
    pub source: Option<Entity>,
    pub pose: Option<Pose>,
    pub scale: Option<Scale>,
    pub is_static: Option<IsStatic>,
    // Name is intentionally excluded — copying names would create duplicates.
}

impl PropertiesClipboard {
    pub fn is_empty(&self) -> bool {
        self.pose.is_none() && self.scale.is_none() && self.is_static.is_none()
    }

    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if self.pose.is_some() {
            parts.push("pose");
        }
        if self.scale.is_some() {
            parts.push("scale");
        }
        if self.is_static.is_some() {
            parts.push("static");
        }
        parts.join(", ")
    }
}

/// Event to copy properties from the selected entity.
#[derive(Event)]
pub struct CopyProperties;

/// Event to paste properties onto the selected entity.
#[derive(Event)]
pub struct PasteProperties;

fn handle_copy(
    mut events: EventReader<CopyProperties>,
    selection: Res<Selection>,
    poses: Query<&Pose>,
    scales: Query<&Scale>,
    statics: Query<&IsStatic>,
    names: Query<&NameInSite>,
    mut clipboard: ResMut<PropertiesClipboard>,
    mut notifications: ResMut<Notifications>,
) {
    for _event in events.read() {
        let Some(entity) = selection.0 else {
            notifications.error("No entity selected to copy");
            continue;
        };

        clipboard.source = Some(entity);
        clipboard.pose = poses.get(entity).ok().cloned();
        clipboard.scale = scales.get(entity).ok().cloned();
        clipboard.is_static = statics.get(entity).ok().cloned();

        let name = names
            .get(entity)
            .map(|n| format!("\"{}\"", n.0))
            .unwrap_or_else(|_| "entity".to_string());

        if clipboard.is_empty() {
            notifications.warning(format!("No copyable properties on {name}"));
        } else {
            notifications.success(format!("Copied {} from {name}", clipboard.summary()));
        }
    }
}

fn handle_paste(
    mut events: EventReader<PasteProperties>,
    selection: Res<Selection>,
    clipboard: Res<PropertiesClipboard>,
    names: Query<&NameInSite>,
    mut commands: Commands,
    mut notifications: ResMut<Notifications>,
) {
    for _event in events.read() {
        let Some(entity) = selection.0 else {
            notifications.error("No entity selected to paste onto");
            continue;
        };

        if clipboard.is_empty() {
            notifications.warning("Clipboard is empty — copy properties first (Ctrl+Shift+C)");
            continue;
        }

        if clipboard.source == Some(entity) {
            notifications.warning("Cannot paste onto the same entity");
            continue;
        }

        let mut pasted = Vec::new();

        if let Some(pose) = &clipboard.pose {
            commands.trigger(Change::new(pose.clone(), entity));
            pasted.push("pose");
        }
        if let Some(scale) = &clipboard.scale {
            commands.trigger(Change::new(scale.clone(), entity));
            pasted.push("scale");
        }
        if let Some(is_static) = &clipboard.is_static {
            commands.trigger(Change::new(*is_static, entity));
            pasted.push("static");
        }

        let name = names
            .get(entity)
            .map(|n| format!("\"{}\"", n.0))
            .unwrap_or_else(|_| "entity".to_string());
        notifications.success(format!("Pasted {} to {name}", pasted.join(", ")));
    }
}

pub struct PropertiesClipboardPlugin;

impl Plugin for PropertiesClipboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PropertiesClipboard>()
            .add_event::<CopyProperties>()
            .add_event::<PasteProperties>()
            .add_systems(Update, (handle_copy, handle_paste));
    }
}
