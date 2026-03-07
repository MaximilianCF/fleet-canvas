# ROADMAP

This document tracks what has been done and what is planned for this desktop-focused fork of RMF Site Editor. It serves as the guide for all development sessions.

Upstream repo: https://github.com/open-rmf/rmf_site

---

## Completed

### v0.0.3 -- Foundation

**Save/Export UX**
- Window title shows `*` when unsaved changes exist
- Exit confirmation dialog: "Save and Exit" / "Exit without Saving" / "Cancel"
- Toast notification system for save/export feedback (success/error)

**Menu System**
- Edit menu with Undo, Redo, Delete (auto-enable/disable based on state)
- File menu: New (Ctrl+N), Open (Ctrl+O), Save, Save As
- MenuItem::Separator for visual grouping
- Shortcut hints match actual keybindings

**Editing Tools**
- Snap-to-grid: `G` toggle, `Shift+G` cycle presets (0.1m to 5.0m)
- Snapping applied to anchors and model placement
- Status bar: cursor X/Y coordinates, snap state, hint keys

**SDF Export**
- Lights exported to SDF (point, spot, directional) with attenuation, color, shadows
- ROS 2 launch file (launch.py) auto-generated alongside SDF export

**Desktop Packaging**
- cargo-deb integration (.deb builds)
- AppImage build script with linuxdeploy
- CI release workflow: builds .deb + AppImage on tag push
- Linux .desktop file for application menu integration
- App icon (256x256 blueprint-style PNG)

**Code Quality**
- Replaced once_cell with std::sync::LazyLock
- Replaced panic!() with Result in rmf_site_mesh
- add_change_plugins! macro to reduce boilerplate
- WASM compilation fixed (cfg gates on desktop-only code)

### v0.0.4 -- UI Redesign

**Tabbed Properties Panel**
- Right panel reorganized from flat collapsing headers into 4 tabs
- Inspect tab (default): element properties, improved empty state
- Site tab: Levels, Scenarios, Models, Lights
- Nav tab: Navigation graphs, Layers
- Tasks tab: Tasks, Groups, Building preview
- PanelTab component + ActivePanelTab resource in rmf_site_egui

**CI Fixes**
- Added system dependencies to style workflow for clippy
- Disabled ci_linux and ci_windows automatic triggers (manual only)
- Fixed AppImage build script variable export
- Updated README for the fork

### v0.0.5 -- P0 + P1

**Welcome Screen** (P0)
- Full-screen splash with "New Project", "Open File", "Load Demo Map"
- Dark theme, centered layout, keyboard hints
- Version display with "Desktop Edition" tag

**Clippy Fixes** (P0)
- Fixed 25+ clippy errors in rmf_site_camera (needless_return, clone_on_copy, etc.)
- CI clippy changed to -W clippy::all (warn, not error) for remaining upstream issues

**Entity Search Bar** (P1, upstream #342)
- Search field in Inspect tab to find elements by name
- Shows category and SiteID, click to select
- Max 20 results, alphabetically sorted

**Menu Close Fix** (P1, upstream #393)
- Menus now close after clicking an item (ui.close_menu())

**Fix File Open While Site Loaded** (P1, upstream #354)
- Old workspace entities despawned before loading new file
- Selection and CurrentWorkspace resources reset on load

**Automatic Backups** (P1, upstream #256)
- Auto-saves every 2 minutes to ~/.cache/rmf_site_editor/backups/
- Keeps last 5 backups per site, auto-cleanup of old files
- Desktop only (not WASM)

**Duplicate Location Name Diagnostic** (P3, upstream #346)
- Validation warns when two locations share the same name
- Integrated into existing diagnostics panel (Validate button)

**Type-in Site ID** (P3, upstream #322)
- Search bar supports `#123` syntax to jump to entity by SiteID

**Reduce Compile Times** (P3, upstream #291, #292)
- Mold linker config added to .cargo/config.toml (commented, opt-in)

### v0.0.6 -- Editor Features & Polish

**Default Scenario Persistence** (P1, upstream #365)
- `default_scenario` field added to Site JSON format
- Save persists which scenario is the default (by SiteID)
- Load restores DefaultScenario and auto-selects it

**Snap Grid Overlay** (P2, upstream #304)
- Gizmo-based grid that matches the snap-to-grid size
- Minor/major lines + colored axis indicators (red X, green Y)
- Alt+G toggle, View menu checkbox "Snap Grid"
- Grid indicator in status bar

**Improved Error Notifications** (P3, upstream #296)
- Model loading failures shown as toast notifications with model name
- Site loading errors shown as toast
- Nav graph import shows success/error toast
- Console: "Clear" button, fixed panel ID typo

**UI Polish**
- View menu: Orthographic/Perspective items (F2/F3), Snap Grid checkbox
- Status bar: projection mode indicator, grid state, expanded shortcut hints
- Fuel asset browser: tooltips on all buttons, better empty state messages
- Keyboard: F2/F3/Delete/Debug feedback via toast notifications
- Keyboard refactored to stay within Bevy 16-param system limit

---

## Planned

### P0 -- Critical / Next

**GitHub Repo Presentation**
- Repository description and topics (robotics, rmf, bevy, rust, editor)
- Social preview image for link sharing
- Add screenshots to README (editor with a loaded site)

### P1 -- High Priority / UX

**Improve Mutex Group UX** (upstream #407)
- Current mutex group editing is confusing
- Better visual feedback for group membership

### P2 -- Medium Priority / Editor Features

**Path Inspector Tool** (upstream #358)
- Visual tool to inspect and debug navigation paths

**Billboard Interactions for Locations** (upstream #381)
- Better in-world interaction for location markers

**Sub-element Hover/Select** (upstream #380)
- Hover and select individual sub-elements (e.g., door handles, wall segments)

**Zone Sketching Tool** (upstream #183)
- Draw zones/regions on the map for area-based constraints

**Saving Views** (upstream #217)
- Save camera positions and switch between named views

**List Nearby Elements** (upstream #195)
- Context panel showing elements near the cursor or selection

### P3 -- Low Priority / Polish

**Reduce Clutter in Right Panel** (upstream #261, #355)
- Further UI cleanup beyond tabs (progressive disclosure, better defaults)

**Improve Error Messages / Diagnostics** (upstream #296)
- Better feedback when assets fail to load

**Export Customizability** (upstream #246)
- Options for what to include/exclude in SDF export

### P4 -- Future / Research

**SDF Support Improvements** (upstream #210)
- Tracking issue for broader SDF import/export capabilities
- Reduce hard-coding in SDF pipeline (upstream #328)

**Workcell / Dispenser Integration** (upstream #244)
- Support for workcell definitions and dispenser/ingestor placement

**Runtime Nav Graph Constraints** (upstream #222)
- Variable constraints in navigation graphs for dynamic scenarios

**Drawing Warp Fiducials** (upstream #184)
- Local-warp fiducials for floor plan alignment

**Vendor Data Bridges** (upstream #193)
- Import/export vendor-specific data formats

**Remove .expect()/.unwrap()** (upstream #255)
- Systematic replacement across codebase for robustness

---

## Known Issues

- **style CI clippy**: uses `-W clippy::all` (warn only) due to remaining upstream warnings
- **ci_linux / ci_windows disabled**: only run manually via workflow_dispatch
- **No app icon in window titlebar**: Bevy window icon requires separate setup
- **Lanes on wrong levels**: upstream #395 (not yet investigated)
- **Flaky roundtrip test**: upstream #409

## Release Process

1. Commit and push to `main`
2. Create annotated tag: `git tag v0.X.Y`
3. Push tag: `git push origin v0.X.Y`
4. Release workflow builds .deb + AppImage and creates GitHub Release
5. Artifacts available at https://github.com/MaximilianCF/rmf_site/releases

## Architecture Reference

- **Engine**: Bevy 0.16 + egui (via bevy_egui 0.34)
- **Workspace**: 8 crates under `crates/`
- **Key crate**: `rmf_site_editor` (binary + lib)
- **UI system**: PanelWidget > PropertiesPanel > Tiles (WidgetSystem<Tile> trait)
- **Tabbed panel**: PanelTab component, ActivePanelTab resource, PANEL_TAB_ORDER constant
- **Menu system**: ECS-based Menu/MenuItem hierarchy with parent-child relationships
- **Format**: `.site.json` (current), `.building.yaml` (legacy import)
- **Desktop-only gates**: `#[cfg(not(target_arch = "wasm32"))]`
