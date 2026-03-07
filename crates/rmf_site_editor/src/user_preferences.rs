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

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// User preferences persisted between sessions.
#[derive(Serialize, Deserialize, Debug, Clone, Resource)]
pub struct UserPreferences {
    /// Last opened file path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_file: Option<PathBuf>,
    /// Window width in pixels.
    #[serde(default = "default_window_width")]
    pub window_width: f32,
    /// Window height in pixels.
    #[serde(default = "default_window_height")]
    pub window_height: f32,
}

fn default_window_width() -> f32 {
    1600.0
}
fn default_window_height() -> f32 {
    900.0
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            last_file: None,
            window_width: default_window_width(),
            window_height: default_window_height(),
        }
    }
}

impl UserPreferences {
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("rmf_site_editor").join("preferences.json"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };
        match std::fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let Some(path) = Self::config_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    warn!("Failed to save preferences: {e}");
                }
            }
            Err(e) => warn!("Failed to serialize preferences: {e}"),
        }
    }
}

/// Track window size changes and save preferences on exit.
fn track_window_size(
    windows: Query<&Window, (With<bevy::window::PrimaryWindow>, Changed<Window>)>,
    mut prefs: ResMut<UserPreferences>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    prefs.window_width = window.resolution.width();
    prefs.window_height = window.resolution.height();
}

/// Save the last opened file path whenever a file is loaded.
fn track_last_file(
    files: Query<&crate::site::DefaultFile, Changed<crate::site::DefaultFile>>,
    mut prefs: ResMut<UserPreferences>,
) {
    for file in files.iter() {
        prefs.last_file = Some(file.0.clone());
    }
}

/// Save preferences on app exit.
fn save_preferences_on_exit(
    mut exit_events: EventReader<bevy::app::AppExit>,
    prefs: Res<UserPreferences>,
) {
    for _ in exit_events.read() {
        prefs.save();
    }
}

/// Periodically auto-save preferences (every 30 seconds).
fn auto_save_preferences(time: Res<Time>, prefs: Res<UserPreferences>, mut timer: Local<f32>) {
    *timer += time.delta_secs();
    if *timer > 30.0 {
        *timer = 0.0;
        if prefs.is_changed() {
            prefs.save();
        }
    }
}

pub struct UserPreferencesPlugin;

impl Plugin for UserPreferencesPlugin {
    fn build(&self, app: &mut App) {
        let prefs = UserPreferences::load();
        app.insert_resource(prefs).add_systems(
            Update,
            (
                track_window_size,
                track_last_file,
                save_preferences_on_exit,
                auto_save_preferences,
            ),
        );
    }
}
