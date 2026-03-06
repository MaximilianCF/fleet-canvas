/*
 * Copyright (C) 2025 Open Source Robotics Foundation
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

use bevy::{ecs::component::Mutable, prelude::*};
use std::collections::VecDeque;
use std::fmt::Debug;

/// Maximum number of operations stored in the undo history.
const MAX_HISTORY_SIZE: usize = 256;

/// A single undoable operation that can be reversed.
trait UndoOperation: Send + Sync + 'static {
    fn undo(&self, world: &mut World);
    fn redo(&self, world: &mut World);
    fn description(&self) -> &str;
}

/// A typed undo operation that stores the previous and new values of a component.
struct ComponentChange<T: Component<Mutability = Mutable> + Clone + Debug> {
    entity: Entity,
    old_value: T,
    new_value: T,
}

impl<T: Component<Mutability = Mutable> + Clone + Debug> UndoOperation for ComponentChange<T> {
    fn undo(&self, world: &mut World) {
        if let Some(mut component) = world.get_mut::<T>(self.entity) {
            *component = self.old_value.clone();
        }
    }

    fn redo(&self, world: &mut World) {
        if let Some(mut component) = world.get_mut::<T>(self.entity) {
            *component = self.new_value.clone();
        }
    }

    fn description(&self) -> &str {
        std::any::type_name::<T>()
    }
}

/// The undo/redo history stack.
#[derive(Resource)]
pub struct UndoHistory {
    undo_stack: VecDeque<Box<dyn UndoOperation>>,
    redo_stack: Vec<Box<dyn UndoOperation>>,
    /// Temporarily disables recording when we are performing an undo/redo
    /// operation, to avoid the undo itself being recorded as a new operation.
    recording_enabled: bool,
}

impl Default for UndoHistory {
    fn default() -> Self {
        Self {
            undo_stack: VecDeque::with_capacity(MAX_HISTORY_SIZE),
            redo_stack: Vec::new(),
            recording_enabled: true,
        }
    }
}

impl UndoHistory {
    pub fn push(&mut self, operation: Box<dyn UndoOperation>) {
        if !self.recording_enabled {
            return;
        }
        if self.undo_stack.len() >= MAX_HISTORY_SIZE {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(operation);
        // A new action clears the redo stack
        self.redo_stack.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.back().map(|op| op.description())
    }

    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.last().map(|op| op.description())
    }
}

/// Event to request an undo operation.
#[derive(Event)]
pub struct UndoRequest;

/// Event to request a redo operation.
#[derive(Event)]
pub struct RedoRequest;

/// Records a component change into the undo history. This should be called
/// by the change_plugin before applying the change, to capture the old value.
pub fn record_change<T: Component<Mutability = Mutable> + Clone + Debug>(
    entity: Entity,
    old_value: T,
    new_value: T,
    history: &mut UndoHistory,
) {
    history.push(Box::new(ComponentChange::<T> {
        entity,
        old_value,
        new_value,
    }));
}

fn handle_undo(world: &mut World) {
    let mut events = world.resource_mut::<Events<UndoRequest>>();
    let should_undo = events.drain().next().is_some();

    if !should_undo {
        return;
    }

    let operation = {
        let mut history = world.resource_mut::<UndoHistory>();
        history.undo_stack.pop_back()
    };

    if let Some(operation) = operation {
        {
            let mut history = world.resource_mut::<UndoHistory>();
            history.recording_enabled = false;
        }

        operation.undo(world);

        let mut history = world.resource_mut::<UndoHistory>();
        history.recording_enabled = true;
        history.redo_stack.push(operation);
    }
}

fn handle_redo(world: &mut World) {
    let mut events = world.resource_mut::<Events<RedoRequest>>();
    let should_redo = events.drain().next().is_some();

    if !should_redo {
        return;
    }

    let operation = {
        let mut history = world.resource_mut::<UndoHistory>();
        history.redo_stack.pop()
    };

    if let Some(operation) = operation {
        {
            let mut history = world.resource_mut::<UndoHistory>();
            history.recording_enabled = false;
        }

        operation.redo(world);

        let mut history = world.resource_mut::<UndoHistory>();
        history.recording_enabled = true;
        history.undo_stack.push_back(operation);
    }
}

pub struct UndoPlugin;

impl Plugin for UndoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UndoHistory>()
            .add_event::<UndoRequest>()
            .add_event::<RedoRequest>()
            .add_systems(PreUpdate, (handle_undo, handle_redo));
    }
}
