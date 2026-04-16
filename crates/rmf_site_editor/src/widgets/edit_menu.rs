use crate::{
    AppState,
    interaction::{CopyProperties, MultiSelection, PasteProperties, PropertiesClipboard},
    site::Delete,
    undo::{RedoRequest, UndoHistory, UndoRequest},
};
use bevy::{ecs::hierarchy::ChildOf, prelude::*};
use rmf_site_egui::*;
use rmf_site_picking::Selection;

#[derive(Resource)]
pub struct EditMenuItems {
    undo: Entity,
    redo: Entity,
    _separator1: Entity,
    delete: Entity,
    _separator2: Entity,
    copy_props: Entity,
    paste_props: Entity,
}

impl FromWorld for EditMenuItems {
    fn from_world(world: &mut World) -> Self {
        let edit_menu = world.resource::<EditMenu>().get();

        let undo = world
            .spawn((
                MenuItem::Text(TextMenuItem::new("Undo").shortcut("Ctrl-Z")),
                ChildOf(edit_menu),
                MenuDisabled,
            ))
            .id();
        let redo = world
            .spawn((
                MenuItem::Text(TextMenuItem::new("Redo").shortcut("Ctrl-Shift-Z")),
                ChildOf(edit_menu),
                MenuDisabled,
            ))
            .id();
        let separator1 = world.spawn((MenuItem::Separator, ChildOf(edit_menu))).id();
        let delete = world
            .spawn((
                MenuItem::Text(TextMenuItem::new("Delete").shortcut("Del")),
                ChildOf(edit_menu),
                MenuDisabled,
            ))
            .id();
        let separator2 = world.spawn((MenuItem::Separator, ChildOf(edit_menu))).id();
        let copy_props = world
            .spawn((
                MenuItem::Text(TextMenuItem::new("Copy Properties").shortcut("Ctrl-Shift-C")),
                ChildOf(edit_menu),
                MenuDisabled,
            ))
            .id();
        let paste_props = world
            .spawn((
                MenuItem::Text(TextMenuItem::new("Paste Properties").shortcut("Ctrl-Shift-V")),
                ChildOf(edit_menu),
                MenuDisabled,
            ))
            .id();

        Self {
            undo,
            redo,
            _separator1: separator1,
            delete,
            _separator2: separator2,
            copy_props,
            paste_props,
        }
    }
}

fn update_edit_menu_state(
    edit_menu: Res<EditMenuItems>,
    undo_history: Res<UndoHistory>,
    selection: Res<Selection>,
    clipboard: Res<PropertiesClipboard>,
    mut commands: Commands,
) {
    if undo_history.can_undo() {
        commands.entity(edit_menu.undo).remove::<MenuDisabled>();
    } else {
        commands.entity(edit_menu.undo).insert(MenuDisabled);
    }

    if undo_history.can_redo() {
        commands.entity(edit_menu.redo).remove::<MenuDisabled>();
    } else {
        commands.entity(edit_menu.redo).insert(MenuDisabled);
    }

    if selection.0.is_some() {
        commands.entity(edit_menu.delete).remove::<MenuDisabled>();
        commands
            .entity(edit_menu.copy_props)
            .remove::<MenuDisabled>();
    } else {
        commands.entity(edit_menu.delete).insert(MenuDisabled);
        commands.entity(edit_menu.copy_props).insert(MenuDisabled);
    }

    if selection.0.is_some() && !clipboard.is_empty() {
        commands
            .entity(edit_menu.paste_props)
            .remove::<MenuDisabled>();
    } else {
        commands.entity(edit_menu.paste_props).insert(MenuDisabled);
    }
}

fn handle_edit_menu_events(
    mut menu_events: EventReader<MenuEvent>,
    edit_menu: Res<EditMenuItems>,
    selection: Res<Selection>,
    mut multi: ResMut<MultiSelection>,
    mut undo_request: EventWriter<UndoRequest>,
    mut redo_request: EventWriter<RedoRequest>,
    mut delete: EventWriter<Delete>,
    mut copy_events: EventWriter<CopyProperties>,
    mut paste_events: EventWriter<PasteProperties>,
) {
    for event in menu_events.read() {
        if !event.clicked() {
            continue;
        }
        let source = event.source();
        if source == edit_menu.undo {
            undo_request.write(UndoRequest);
        } else if source == edit_menu.redo {
            redo_request.write(RedoRequest);
        } else if source == edit_menu.delete {
            let all = multi.all_with_primary(&selection);
            for entity in &all {
                delete.write(Delete::new(*entity));
            }
            if !all.is_empty() {
                multi.clear();
            }
        } else if source == edit_menu.copy_props {
            copy_events.write(CopyProperties);
        } else if source == edit_menu.paste_props {
            paste_events.write(PasteProperties);
        }
    }
}

#[derive(Default)]
pub struct EditMenuPlugin {}

impl Plugin for EditMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditMenuItems>().add_systems(
            Update,
            (update_edit_menu_state, handle_edit_menu_events)
                .run_if(AppState::in_displaying_mode()),
        );
    }
}
