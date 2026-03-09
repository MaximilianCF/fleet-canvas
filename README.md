# RMF Site Editor

A visual editor for designing and managing robot fleet management (RMF) sites. Built in Rust with [Bevy](https://bevyengine.org/) and [egui](https://github.com/emilk/egui).

Design multi-level buildings with navigation graphs, lanes, locations, doors, lifts, and models — then export to Gazebo SDF with ROS 2 launch files.

<!-- Take a screenshot of the editor with a site loaded and save as docs/screenshots/editor-overview.png -->
<!-- ![Editor Screenshot](docs/screenshots/editor-overview.png) -->

## Features

**Site Editing**
- Multi-level building design with floors, walls, doors, lifts
- Navigation graph editor with robot and human lane types
- Model placement from local files or [Fuel](https://app.gazebosim.org/fuel) asset browser
- Anchor-based geometry with snap-to-grid (configurable presets)
- Measurement tool for distance checking

**Export & Integration**
- SDF export with configurable options (models, doors, lifts, lights)
- Light export: point, spot, and directional lights with full properties
- ROS 2 launch file auto-generated alongside SDF
- Nav graph export for fleet management
- Headless batch export via CLI (`--export_sdf`, `--export_nav`)

**Editor UX**
- Tabbed properties panel (Inspect, Site, Nav, Tasks)
- Entity search bar with `#ID` jump-to syntax
- Saved camera views per level
- Nearby elements panel (shows entities near cursor)
- Multi-select with Ctrl+Click and batch operations
- Properties copy/paste between entities (Ctrl+Shift+C/V)
- Graph View mode to isolate navigation elements (F4)
- Undo/redo with full history
- Auto-backup every 2 minutes
- Toast notifications for all operations
- Window remembers size and last opened file

**Desktop-First**
- Native Linux application (.deb and AppImage packages)
- Animated welcome screen with recent file quick-open
- Status bar: cursor coordinates, snap state, projection mode, tool indicators

## Download

Pre-built packages for Linux are available on the [Releases](https://github.com/MaximilianCF/rmf_site/releases) page:

| Format | Description |
|--------|-------------|
| `.deb` | For Debian, Ubuntu, and derivatives — install with `sudo dpkg -i` |
| `.AppImage` | Portable binary — `chmod +x` and run on any Linux distro |

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

## License

Apache-2.0. See [LICENSE](LICENSE).

Based on [open-rmf/rmf_site](https://github.com/open-rmf/rmf_site) by Open Source Robotics Foundation contributors.
