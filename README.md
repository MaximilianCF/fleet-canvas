# Fleet Canvas

Visual editor for Open-RMF robot fleet deployment sites.

Built in Rust with [Bevy 0.16](https://bevyengine.org/) and
[egui](https://github.com/emilk/egui) — native Linux desktop,
designed for robotics engineers working with Open-RMF fleets.

![License](https://img.shields.io/badge/license-Apache--2.0-blue)
![Bevy](https://img.shields.io/badge/bevy-0.16-green)
![Platform](https://img.shields.io/badge/platform-Linux-lightgrey)

<!-- Take a screenshot of the editor with a site loaded and save as docs/screenshots/editor-overview.png -->
<!-- ![Editor Screenshot](docs/screenshots/editor-overview.png) -->

## Features

**Site Editing**
- Multi-level building: floors, walls, doors, lifts
- Nav graph editor with robot/human lane types and color coding (F4)
- Model placement from local files or [Fuel](https://app.gazebosim.org/fuel) asset browser
- Anchor-based geometry with snap-to-grid (6 presets, Alt+G overlay)
- Measurement tool for distance checking [M]
- Drawing opacity slider for floor plan tracing
- Ortho constraint during wall draw (Shift)
- Properties copy/paste between entities (Ctrl+Shift+C/V)

**Validation & Diagnostics**
- Nav graph connectivity linter (disconnected components, dead-ends)
- Lane clearance check (robot footprint safety margin vs walls)
- Door/lift reachability matrix per (level, fleet) pair
- Lane inline badges: length, direction, travel-time estimate
- Duplicate location name detection
- All validators integrate with the Diagnostics panel

**Traffic & Scenario Preview**
- MAPF plan scrubber: play/pause/seek with speed selector (0.5×–4×)
- Lane usage heatmap (green→red intensity by traversal count)
- Conflict preview: pulsing red overlay on contested lanes

**Editor UX**
- Dark-themed egui panels unified with 3D viewport
- Tabbed properties panel (Inspect / Site / Nav / Tasks)
- Entity search with #ID jump syntax + camera goto + flash highlight
- Minimap corner widget with camera frustum overlay
- Cursor coordinates in site frame + UTM/WGS84 when georeferenced
- Visibility presets: Nav Graph Only / Drawing Mode / Show All
- Saved camera views per level
- Undo/redo with full history
- Auto-backup every 2 minutes to ~/.cache/
- Window icon + .desktop integration for Linux app menu

**Export & Integration**
- SDF export with configurable options (models, doors, lifts, lights)
- Light export: point, spot, and directional with full properties
- ROS 2 launch file auto-generated alongside SDF
- Nav graph export for fleet management
- Launch in Gazebo button (picks previously exported launch.py)
- Headless batch export via CLI (`--export-sdf`, `--export-nav`)
- Headless validation via CLI (`--validate`) for CI pipelines
- Site diff viewer for comparing two `.site.json` files

## Download

Pre-built packages for Linux are available on the
[Releases](https://github.com/MaximilianCF/fleet-canvas/releases) page:

| Format | Description |
|--------|-------------|
| `.deb` | Debian, Ubuntu and derivatives — `sudo dpkg -i` |
| `.AppImage` | Portable — `chmod +x` and run on any Linux distro |

## Building from source

### Dependencies (Ubuntu/Debian)

```bash
sudo apt install libgtk-3-dev libasound2-dev libudev-dev
```

Install Rust via [rustup](https://www.rust-lang.org/tools/install):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Run

```bash
cargo run                                    # Debug build
cargo run --features bevy/dynamic_linking    # Faster incremental compiles
cargo run --release                          # Release build (recommended)
```

### Build packages

```bash
# .deb package
cargo install cargo-deb
cargo deb -p rmf_site_editor

# AppImage (requires linuxdeploy, auto-downloaded if missing)
cargo build --release --bin rmf_site_editor
bash packaging/build-appimage.sh
```

## Keyboard shortcuts

| Key | Action |
|-----|--------|
| **File** | |
| Ctrl+N | New workspace |
| Ctrl+O | Open file |
| Ctrl+S | Save |
| Ctrl+Shift+S | Save As |
| Ctrl+E | Export SDF (opens options dialog) |
| **Edit** | |
| Ctrl+Z | Undo |
| Ctrl+Shift+Z / Ctrl+Y | Redo |
| Delete / Backspace | Delete selected (supports multi-select) |
| Ctrl+Shift+C | Copy properties |
| Ctrl+Shift+V | Paste properties |
| **View & Tools** | |
| F2 | Orthographic projection |
| F3 | Perspective projection |
| F4 | Toggle Graph View |
| G | Toggle snap-to-grid |
| Shift+G | Cycle grid size (0.1, 0.25, 0.5, 1.0, 2.0, 5.0m) |
| Alt+G | Toggle snap grid overlay |
| M | Toggle measurement tool |
| Shift (hold) | Ortho constraint during wall/edge draw (0/45/90°) |
| D | Toggle debug mode |
| **Selection** | |
| Ctrl+Click | Add/remove from multi-selection |
| Escape | Cancel current tool / deselect |

## Project structure

Cargo workspace with 8 crates:

| Crate | Role |
|-------|------|
| `rmf_site_editor` | Main application — Bevy ECS plugins, UI, interaction, export |
| `rmf_site_format` | Data model and serialization (`.site.json`, SDF export) |
| `rmf_site_egui` | Menu bar and UI widget framework |
| `rmf_site_camera` | 3D camera controls (orbit, pan, zoom) |
| `rmf_site_picking` | Mouse picking, selection, and hover system |
| `rmf_site_mesh` | Procedural mesh generation (walls, floors, doors) |
| `rmf_site_animate` | Visual cue animations |
| `rmf_site_editor_web` | WASM entry point (standby) |

## ROS 2 integration

Exported SDF worlds work with Gazebo and the [Open-RMF](https://github.com/open-rmf) ecosystem. The generated `launch.py` can be used directly:

```bash
ros2 launch <export_dir>/launch.py
```

For full fleet management integration, see [rmf_site_ros2](https://github.com/open-rmf/rmf_site_ros2).

## Credits

Fleet Canvas is a desktop-focused fork of
[open-rmf/rmf_site](https://github.com/open-rmf/rmf_site) by the
Open Source Robotics Foundation, used under the Apache-2.0 license.

## License

Apache-2.0. See [LICENSE](LICENSE).
