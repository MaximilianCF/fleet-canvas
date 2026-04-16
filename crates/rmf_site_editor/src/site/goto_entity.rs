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

//! Flash-highlight component: when inserted on an entity, briefly pulses
//! the entity's material color to draw the user's eye, then removes itself.

use bevy::prelude::*;

/// Insert on an entity to pulse its material between the base color and
/// bright cyan for `duration` seconds, then auto-remove.
#[derive(Component)]
pub struct FlashHighlight {
    pub duration: f32,
    pub elapsed: f32,
    pub original_color: Option<Color>,
}

impl FlashHighlight {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            original_color: None,
        }
    }
}

const FLASH_COLOR: Color = Color::srgb(0.0, 1.0, 1.0);

/// Advance flash timers, tint materials, and clean up expired flashes.
pub fn update_flash_highlights(
    mut commands: Commands,
    time: Res<Time>,
    mut flashes: Query<(
        Entity,
        &mut FlashHighlight,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut flash, mat_handle) in flashes.iter_mut() {
        let Some(mat) = materials.get_mut(&mat_handle.0) else {
            continue;
        };

        // On first frame, capture the original color.
        if flash.original_color.is_none() {
            flash.original_color = Some(mat.base_color);
        }

        flash.elapsed += time.delta_secs();

        if flash.elapsed >= flash.duration {
            // Restore and remove.
            if let Some(orig) = flash.original_color {
                mat.base_color = orig;
            }
            commands.entity(entity).remove::<FlashHighlight>();
            continue;
        }

        // Fast blink (~2.5 Hz).
        let pulse = (flash.elapsed * std::f32::consts::TAU / 0.4).sin() * 0.5 + 0.5;
        let orig = flash.original_color.unwrap_or(Color::WHITE);

        let mix = |a: f32, b: f32, t: f32| a + (b - a) * t;
        let oc = orig.to_srgba();
        let fc = FLASH_COLOR.to_srgba();
        mat.base_color = Color::srgb(
            mix(oc.red, fc.red, pulse),
            mix(oc.green, fc.green, pulse),
            mix(oc.blue, fc.blue, pulse),
        );
    }
}
