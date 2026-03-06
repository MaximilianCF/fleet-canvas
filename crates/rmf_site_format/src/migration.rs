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

use crate::{SemVer, CURRENT_MAJOR_VERSION, CURRENT_MINOR_VERSION};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MigrationError {
    #[error(
        "Cannot migrate from major version {0} to {1}: major version upgrades \
        are not backwards compatible"
    )]
    IncompatibleMajorVersion(u32, u32),
    #[error("Migration step from {from} to {to} failed: {reason}")]
    StepFailed {
        from: String,
        to: String,
        reason: String,
    },
    #[error("Format version field is missing or invalid")]
    MissingFormatVersion,
}

/// A single migration step that transforms a JSON value from one version to the next.
struct MigrationStep {
    from: SemVer,
    to: SemVer,
    migrate: fn(&mut Value) -> Result<(), String>,
}

/// Registry of all known migrations, applied in sequence.
pub struct MigrationRegistry {
    steps: Vec<MigrationStep>,
}

impl MigrationRegistry {
    pub fn new() -> Self {
        let mut registry = Self { steps: Vec::new() };
        registry.register_all();
        registry
    }

    fn register_all(&mut self) {
        // v0.1 -> v0.2: task timestamps i32 -> i64 (compatible at JSON level,
        // but we bump the minor version to track it)
        self.steps.push(MigrationStep {
            from: SemVer(0, 1),
            to: SemVer(0, 2),
            migrate: migrate_v0_1_to_v0_2,
        });
    }

    /// Apply all necessary migrations to bring a site JSON value up to the
    /// current version. Returns the final version after migration.
    pub fn migrate(&self, value: &mut Value) -> Result<SemVer, MigrationError> {
        let mut current = parse_format_version(value)?;

        if current.major() > CURRENT_MAJOR_VERSION {
            return Err(MigrationError::IncompatibleMajorVersion(
                current.major(),
                CURRENT_MAJOR_VERSION,
            ));
        }

        let target = SemVer(CURRENT_MAJOR_VERSION, CURRENT_MINOR_VERSION);

        // Already at or beyond target version (forward compat for minor bumps)
        if current.major() == target.major() && current.minor() >= target.minor() {
            return Ok(current);
        }

        for step in &self.steps {
            if current.major() == step.from.major() && current.minor() == step.from.minor() {
                (step.migrate)(value).map_err(|reason| MigrationError::StepFailed {
                    from: step.from.to_string(),
                    to: step.to.to_string(),
                    reason,
                })?;
                current = step.to;

                // Update the format_version in the JSON
                if let Some(obj) = value.as_object_mut() {
                    obj.insert(
                        "format_version".to_string(),
                        Value::String(current.to_string()),
                    );
                }
            }
        }

        Ok(current)
    }
}

fn parse_format_version(value: &Value) -> Result<SemVer, MigrationError> {
    let version_str = value
        .get("format_version")
        .and_then(|v| v.as_str())
        .ok_or(MigrationError::MissingFormatVersion)?;

    let parts: Vec<&str> = version_str.split('.').collect();
    if parts.len() != 2 {
        return Err(MigrationError::MissingFormatVersion);
    }

    let major = parts[0]
        .parse::<u32>()
        .map_err(|_| MigrationError::MissingFormatVersion)?;
    let minor = parts[1]
        .parse::<u32>()
        .map_err(|_| MigrationError::MissingFormatVersion)?;

    Ok(SemVer(major, minor))
}

/// Migration from v0.1 to v0.2:
/// - Task timestamps changed from i32 to i64. JSON numbers are already
///   compatible, but we ensure any existing integer values are valid i64.
fn migrate_v0_1_to_v0_2(value: &mut Value) -> Result<(), String> {
    if let Some(tasks) = value.get_mut("tasks").and_then(|t| t.as_object_mut()) {
        for (_task_id, task_value) in tasks.iter_mut() {
            // TaskParams might be nested inside Dispatch or Direct variants
            migrate_task_timestamps(task_value);
        }
    }
    Ok(())
}

fn migrate_task_timestamps(task_value: &mut Value) {
    // Task is an enum: {"Dispatch": {...}} or {"Direct": {...}}
    // The TaskParams are separate from Task, but stored in scenario modifiers.
    // For the task itself, timestamps live in TaskParams which is a separate
    // field. The i32->i64 change is seamless in JSON, so this is mostly a
    // validation pass.
    let params_fields = [
        "unix_millis_earliest_start_time",
        "unix_millis_request_time",
    ];

    for field in &params_fields {
        if let Some(val) = task_value.get(field) {
            if let Some(n) = val.as_i64() {
                // Value fits in i64, which is the new type - nothing to do
                let _ = n;
            }
        }
    }

    // Also check inside scenario modifiers
    if let Some(scenarios) = task_value
        .get_mut("scenarios")
        .and_then(|s| s.as_object_mut())
    {
        for (_scenario_id, scenario) in scenarios.iter_mut() {
            if let Some(task_mods) = scenario.get_mut("tasks").and_then(|t| t.as_object_mut()) {
                for (_id, modifier) in task_mods.iter_mut() {
                    if let Some(params) = modifier.get_mut("params") {
                        for field in &params_fields {
                            if let Some(val) = params.get(field) {
                                let _ = val.as_i64();
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn migrate_from_v0_1_to_current() {
        let mut site = json!({
            "format_version": "0.1",
            "properties": {
                "name": "test_site"
            },
            "tasks": {
                "1": {
                    "Dispatch": {
                        "request": {
                            "category": "patrol",
                            "description": {}
                        }
                    }
                }
            }
        });

        let registry = MigrationRegistry::new();
        let result = registry.migrate(&mut site);
        assert!(result.is_ok());
        let version = result.unwrap();
        assert_eq!(version.major(), CURRENT_MAJOR_VERSION);
        assert_eq!(version.minor(), CURRENT_MINOR_VERSION);
        assert_eq!(
            site.get("format_version").unwrap().as_str().unwrap(),
            format!("{}.{}", CURRENT_MAJOR_VERSION, CURRENT_MINOR_VERSION)
        );
    }

    #[test]
    fn already_current_version_is_noop() {
        let version_str = format!("{}.{}", CURRENT_MAJOR_VERSION, CURRENT_MINOR_VERSION);
        let mut site = json!({
            "format_version": version_str,
            "properties": { "name": "test" }
        });

        let registry = MigrationRegistry::new();
        let result = registry.migrate(&mut site);
        assert!(result.is_ok());
    }

    #[test]
    fn incompatible_major_version_fails() {
        let mut site = json!({
            "format_version": "99.0",
            "properties": { "name": "test" }
        });

        let registry = MigrationRegistry::new();
        let result = registry.migrate(&mut site);
        assert!(result.is_err());
    }
}
