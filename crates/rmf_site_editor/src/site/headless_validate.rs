/*
 * Copyright (C) 2026 Open Source Robotics Foundation
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

//! Headless validation mode. When `--validate` is passed on the CLI, the
//! editor loads the requested site, runs all diagnostic validators, prints
//! a summary to stdout, and exits with a non-zero code if any issues exist.
//! Intended for CI pipelines to block merges with broken nav graphs.

use bevy::prelude::*;

use crate::{Autoload, Issue, ValidateWorkspace, WorkspaceLoader};
use rmf_site_format::NameOfSite;

use crate::site::ModelLoadingState;
use crossflow::Promise;

#[derive(Resource, Default)]
pub struct HeadlessValidateState {
    iterations: u32,
    models_loaded: bool,
    validate_sent: bool,
    loading: Option<Promise<()>>,
}

pub fn headless_validate(
    mut exit: EventWriter<bevy::app::AppExit>,
    missing_models: Query<(), With<ModelLoadingState>>,
    mut state: ResMut<HeadlessValidateState>,
    sites: Query<Entity, With<NameOfSite>>,
    autoload: Option<ResMut<Autoload>>,
    mut workspace_loader: WorkspaceLoader,
    mut validate_events: EventWriter<ValidateWorkspace>,
    issues: Query<&Issue>,
) {
    let Some(mut autoload) = autoload else {
        error!("Cannot validate: no file specified via Autoload");
        exit.write(bevy::app::AppExit::error());
        return;
    };

    if let Some(filename) = autoload.filename.take() {
        state.loading = Some(workspace_loader.load_from_path(filename));
    }

    if state
        .loading
        .as_mut()
        .is_some_and(|p| p.peek().is_pending())
    {
        return;
    }

    state.iterations += 1;

    if state.iterations < 5 {
        return;
    }

    if sites.is_empty() {
        error!("No site is loaded — cannot validate");
        exit.write(bevy::app::AppExit::error());
        return;
    }

    if !state.models_loaded {
        if missing_models.is_empty() {
            state.models_loaded = true;
            state.iterations = 0;
        }
        return;
    }

    if !state.validate_sent {
        for site in sites.iter() {
            info!("Sending ValidateWorkspace for {:?}", site);
            validate_events.write(ValidateWorkspace(site));
        }
        state.validate_sent = true;
        state.iterations = 0;
        return;
    }

    if state.iterations < 5 {
        return;
    }

    // Collect and print issues.
    let issue_list: Vec<&Issue> = issues.iter().collect();
    if issue_list.is_empty() {
        println!("Validation passed: no issues found.");
        exit.write(bevy::app::AppExit::Success);
    } else {
        println!("Validation found {} issue(s):", issue_list.len());
        for (i, issue) in issue_list.iter().enumerate() {
            println!("  [{}] {}", i + 1, issue.brief);
            println!("       Hint: {}", issue.hint);
        }
        exit.write(bevy::app::AppExit::error());
    }
}
