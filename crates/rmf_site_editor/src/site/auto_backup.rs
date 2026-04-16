/*
 * Copyright (C) 2024 Open Source Robotics Foundation
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
use bevy::time::Time;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::{AppState, workspace::CurrentWorkspace};

use super::save::generate_site;

/// How often to auto-backup (in seconds).
const BACKUP_INTERVAL_SECS: f32 = 120.0;

/// Maximum number of backup files to keep.
const MAX_BACKUPS: usize = 5;

#[derive(Resource)]
pub struct AutoBackupConfig {
    pub enabled: bool,
    pub interval_secs: f32,
    pub timer: f32,
    pub backup_dir: PathBuf,
}

impl Default for AutoBackupConfig {
    fn default() -> Self {
        let backup_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("rmf_site_editor")
            .join("backups");
        Self {
            enabled: true,
            interval_secs: BACKUP_INTERVAL_SECS,
            timer: 0.0,
            backup_dir,
        }
    }
}

pub struct AutoBackupPlugin;

impl Plugin for AutoBackupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutoBackupConfig>().add_systems(
            Update,
            auto_backup_system.run_if(AppState::in_displaying_mode()),
        );
    }
}

fn auto_backup_system(world: &mut World) {
    let delta = world.resource::<Time>().delta_secs();

    let (enabled, interval, backup_dir) = {
        let mut config = world.resource_mut::<AutoBackupConfig>();
        config.timer += delta;
        if !config.enabled || config.timer < config.interval_secs {
            return;
        }
        config.timer = 0.0;
        (
            config.enabled,
            config.interval_secs,
            config.backup_dir.clone(),
        )
    };

    let _ = (enabled, interval);

    let Some(workspace_root) = world.resource::<CurrentWorkspace>().root else {
        return;
    };

    let site = match generate_site(world, workspace_root) {
        Ok(site) => site,
        Err(err) => {
            warn!("Auto-backup: failed to generate site: {err}");
            return;
        }
    };

    // Ensure backup directory exists
    if let Err(err) = std::fs::create_dir_all(&backup_dir) {
        warn!("Auto-backup: failed to create directory: {err}");
        return;
    }

    // Generate backup filename with timestamp
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let backup_file = backup_dir.join(format!(
        "{}_backup_{}.site.json",
        site.properties.name.0, timestamp
    ));

    let f = match std::fs::File::create(&backup_file) {
        Ok(f) => f,
        Err(err) => {
            warn!("Auto-backup: failed to create file: {err}");
            return;
        }
    };

    match site.to_writer_json(f) {
        Ok(()) => {
            info!("Auto-backup saved to {}", backup_file.display());
        }
        Err(err) => {
            warn!("Auto-backup: failed to write: {err}");
            return;
        }
    }

    // Clean up old backups, keeping only the most recent MAX_BACKUPS
    cleanup_old_backups(&backup_dir, &site.properties.name.0);
}

fn cleanup_old_backups(backup_dir: &PathBuf, site_name: &str) {
    let prefix = format!("{}_backup_", site_name);
    let mut backups: Vec<PathBuf> = match std::fs::read_dir(backup_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(&prefix) && n.ends_with(".site.json"))
            })
            .collect(),
        Err(_) => return,
    };

    if backups.len() <= MAX_BACKUPS {
        return;
    }

    backups.sort();
    let to_remove = backups.len() - MAX_BACKUPS;
    for backup in backups.iter().take(to_remove) {
        let _ = std::fs::remove_file(backup);
    }
}
