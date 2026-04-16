# ROADMAP

This document tracks what has been done and what is planned for this desktop-focused fork of RMF Site Editor. It serves as the guide for all development sessions.

Upstream repo: https://github.com/open-rmf/rmf_site

---

## Completed

### v0.1.0 -- UX Polish & Drawing UX

**Visibility Presets** (§5)
- "Nav Graph Only": show only lanes, locations, floors
- "Drawing Mode": show walls, floors, fiducials, measurements
- "Show All": restore all categories to visible
- Added as preset items in the existing View menu

**Cursor Coordinate Display with Georeference** (§5)
- Status bar shows lat/lon when site has a `GeographicComponent`
- Converts cursor world (X,Y) via `world_to_latlon()` and displays as `°` format
- Falls back to site-frame only (unchanged behavior) when no georeference

**Edge Length in Inspector** (§5)
- Walls and measurements show their length in the inspector when selected
- Computed from anchor positions via `AnchorParams`

**Drawing Opacity Slider** (§4)
- Per-drawing opacity slider (0.0–1.0) in the drawing inspector
- Fires `Change<LayerVisibility>` events, undo-compatible
- Maps alpha endpoints: 1.0→Opaque, 0.0→Hidden, between→Alpha(f)

**Ortho Constraint** (§4)
- Hold Shift during edge/wall placement to snap to 0°/45°/90° angles
- `OrthoSnapActive` resource updated each frame, read via `CursorSnapParams`
  SystemParam bundle to stay within Bevy's 16-param limit

**Window Icon** (Known Issues fix)
- Native window icon loaded from `packaging/rmf-site-editor.png` via
  `include_bytes!` + `image::load_from_memory` + `winit::window::Icon`
- Desktop-only startup system

**Dark Mode Panels** (§5)
- Fully unified egui panel colours with the Bevy 3D viewport dark theme
- Custom panel/window fills, widget state colours, accent blue, rounded corners

**Search Goto Camera + Flash Highlight** (§5)
- Clicking a search result pans the camera via PanToElement
- FlashHighlight component pulses the entity's material cyan for 1.5 s
- Auto-removes after duration, restoring original colour

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

### v0.0.7 -- Graph View, Human Lanes & Polish

**User Preferences Persistence**
- Window size and last opened file saved to `~/.config/rmf_site_editor/preferences.json`
- Window restores to last used dimensions on startup
- "Open Recent" button on welcome screen when a previous file exists
- Auto-save every 30s, save on exit

**Human Lanes** (from traffic_editor)
- `LaneType` enum added to format: Robot (default), Human
- Human lanes render narrower (0.35m vs 0.5m) with orange/amber tint
- Lane type selector in inspector panel (ComboBox)
- Persisted in `.site.json`, backward-compatible (defaults to Robot)

**Animated Welcome Screen**
- Blueprint-style animated background on main menu
- Drifting grid lines, floating room outlines, waypoint dots with nav lanes
- All rendered via egui painter at low opacity

**Graph View Mode** (F4 toggle)
- Hides all geometry (floors, walls, doors, models, lifts) to isolate nav graph
- Saves and restores previous visibility state on toggle
- View menu checkbox "Graph View" synced with F4 shortcut
- Color-coded elements by type in graph view:
  - Lanes: robot = blue, human = orange
  - Locations: plain = green, charger = yellow, parking = blue, holding = purple
- Legend overlay with color key (bottom-left corner)
- Status bar shows orange "Graph View" indicator when active
- Normal mode retains nav graph colors (unchanged behavior)

### v0.0.9 -- Traffic & Scenario Preview

**Nav Graph Connectivity Linter**
- Per-graph union-find over lanes; emits diagnostics issue for every
  non-largest connected component, listing the offending lane entities.
- Isolated-location detection: a `Location` associated with a nav graph
  whose anchor is not touched by any lane in that graph is flagged as
  unroutable by the fleet adapter.
- New issue type: "Disconnected nav graph component" / "Isolated location"

**Lane Clearance Check**
- For each lane, min 2D segment-to-segment distance to walls on the same
  level. Flags lanes within 0.35 m of a wall (typical delivery robot
  footprint + safety margin).
- Walls/lanes bucketed per `LevelElevation` via `AncestorIter` to avoid
  cross-level false positives.
- New issue type: "Lane too close to wall"

**DAE Loader Fixes**
- Repaired compile errors introduced by the `dae-parser 0.11` bump:
  turbofish on `parse::<Document>()`, `format!("{e:?}")` for the
  non-`Display` error type, and explicit `Option<(&Box<[f32]>, _, _)>`
  annotations on `norm_floats`/`uv_floats` inference sites.

**Scenario Scrubber** (§3)
- Bottom-panel timeline with play/pause/seek and speed selector (0.5×/1×/2×/4×)
- Rewind button, live t = current/total display
- Always visible when a successful MAPF plan exists

**Lane Usage Heatmap** (§3)
- Per-lane traversal count accumulated across all agent trajectories
- Green→yellow→red color ramp normalized to peak usage
- Gizmo overlay toggled via "Heatmap" checkbox in scrubber panel

**Conflict Preview** (§3)
- Detects two-agent spatial+temporal overlaps on shared lanes
- Pulsing red gizmo line + cross marker at midpoint
- Toggled via "Conflicts" checkbox in scrubber panel

**Style & CI fixes** (housekeeping)
- cargo fmt on all modified files
- clippy: get_single() → single(), #[derive(Default)] on headless_validate
- Replaced doublify pre-commit hook with local cargo fmt --check
- Fixed 0-byte workcell.rs

### v0.0.8 -- P2/P3 Features & Repo Polish

**P3: Panel Declutter**
- Light inspector: Color + Intensity upfront, advanced under collapsing header
- Pose inspector: Yaw inline, rotation mode under collapsing header
- Anchor inspector: Dependencies under collapsing header with count

**P3: Improved Error Messages**
- All save/export error paths now show actionable toast notifications
- Model loading failures show model name and reason
- Drawing load failures show toast
- Warning toast variant (amber) added to notification system

**P3: Export Customizability**
- SDF export options dialog (Ctrl+E): choose models, doors, lifts, lights
- ROS 2 launch file generation toggle
- `SdfExportConfig` in format crate, `to_sdf_with_config()` method

**P2: Saved Views** (upstream #217)
- Save/restore named camera positions per level
- Widget in "Site" tab with Save/Go/Rename/Delete
- Data persists in `.site.json` via `UserCameraPose`

**P2: Nearby Elements** (upstream #195)
- "Inspect" tab widget showing entities near cursor
- Configurable radius, sorted by distance, click to select

**P2: Multi-Select**
- Ctrl+Click to additively select entities
- Visual cue via `support_selected` system
- Delete and Edit menu operate on all selected
- Inspector shows count when multiple selected

**P2: Measurement Tool**
- [M] key toggles tool, click two points to measure distance
- Gizmo line visualization (cyan), distance in toast and status bar
- Escape cancels, results persist until next measurement

**P2: Properties Copy/Paste**
- Ctrl+Shift+C copies Pose/Scale/IsStatic from selected entity
- Ctrl+Shift+V pastes to target entity (via Change events, undo-compatible)
- Edit menu items with enable/disable logic

**P0: GitHub Repo Presentation**
- README rewritten with full feature list, keyboard shortcuts, project structure
- Social preview image (1280x640 SVG+PNG)
- Setup script for GitHub description and topics (docs/setup-github.sh)
- Screenshots directory created (docs/screenshots/)

---

## Planned

### P1 -- High Priority / UX

**Improve Mutex Group UX** (upstream #407)
- Current mutex group editing is confusing
- Better visual feedback for group membership

### P2 -- Medium Priority / Editor Features

**Zones / ROI Sketching Tool** (upstream #183, from traffic_editor)
- Draw zones/regions on the map for area-based constraints
- New entity type: format, rendering, UI, serialization

**Coordinate Systems / WGS84** (from traffic_editor)
- Support for geo-referenced coordinate systems

**Path Inspector Tool** (upstream #358)
- Visual tool to inspect and debug navigation paths

**Billboard Interactions for Locations** (upstream #381)
- Better in-world interaction for location markers

**Sub-element Hover/Select** (upstream #380)
- Hover and select individual sub-elements (e.g., door handles, wall segments)

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

## Robotics Workflow Features (pain-point driven)

A parallel view of the roadmap grouped by the *class of pain* each feature
addresses in an Open-RMF development workflow, rather than by priority
tier. Useful when you already know which part of your workflow hurts.

Legend: ✅ done · 🚧 in progress · ⏳ pending

### §1 — Nav graph validation & feedback

Catches authoring mistakes that would otherwise silently break the fleet
adapter. All validators plug into the existing `ValidateWorkspace` event
and `Diagnostics` panel.

- ✅ **Connectivity linter** — per-graph union-find; disconnected
  components, isolated locations, one-way dead-ends *(v0.0.9)*.
- ✅ **Clearance check** — min 2D distance from each lane to same-level
  walls; flags lanes narrower than robot footprint + safety margin
  *(v0.0.9)*.
- ⏳ **Door/lift reachability matrix** — for each `(level, fleet)` pair,
  which locations are actually reachable. Catches "forgot to tag the lift
  cabin lane".
- ⏳ **Lane inline badges** — on selection, overlay with length, per-fleet
  travel-time estimate, bidirectionality flag.

### §2 — Live Open-RMF round-trip

Closes the edit → test → fix loop so you stop bouncing between editor,
Gazebo, and text editors.

- ⏳ **"Launch in Gazebo" one-click** — button that exports the current
  site and spawns `ros2 launch rmf_demos_gz` with the generated world.
- ⏳ **ROS 2 fleet-state overlay** — desktop-only feature flag; subscribe
  to `/fleet_states` and `/task_summaries` and render robot poses and
  active task paths live on top of the site.

### §3 — Traffic & scenario preview

Makes `Scenario` and `Task` data visually actionable rather than just
config text.

- ✅ **Scenario scrubber** — bottom-panel timeline with play/pause/seek
  that animates scheduled tasks through the nav graph. Reuses
  `rmf_site_animate` *(v0.0.9)*.
- ✅ **Lane usage heatmap** — per-lane usage accumulated across a
  scenario, coloured by intensity to expose congestion hotspots
  *(v0.0.9)*.
- ✅ **Conflict preview** — in multi-robot scenarios, flash intersections
  where two robots' time windows overlap on the same lane *(v0.0.9)*.

### §4 — Drawing / fiducial alignment UX

The most error-prone authoring step; feedback loop today is terrible.

- ⏳ **Fiducial residual display** — per-fiducial residual error in mm
  after alignment. Data already lives in `site/fiducial.rs`.
- ✅ **Drawing opacity + x-ray mode** — per-drawing opacity slider in
  the inspector *(v0.1.0)*.
- ✅ **Rectangular snap + ortho constraint** — shift-hold during edge
  placement clamps to 0/45/90° via `CursorSnapParams` SystemParam bundle
  *(v0.1.0)*.

### §5 — General UX polish

Low-effort items a robotics dev notices in the first hour.

- ✅ **Measurement readouts everywhere** — selecting a wall or
  measurement shows its length in the inspector *(v0.1.0)*.
- ✅ **Visibility presets** — "Nav Graph Only", "Drawing Mode", "Show
  All" one-click presets in the View menu *(v0.1.0)*.
- ✅ **Minimap** — corner widget showing current level with walls, lanes,
  and camera indicator *(v0.1.0)*.
- ✅ **Cursor coordinate display** — site frame + lat/lon when
  georeferenced, shown in the status bar *(v0.1.0)*.
- ✅ **Dark mode panels** — egui visuals fully unified with the Bevy
  viewport dark theme: panel/window fills, widget states, accent colour,
  rounded corners *(v0.1.0)*.
- ✅ **Search-by-name with goto camera** — clicking a search result pans
  the camera to the entity and flash-highlights it in cyan for 1.5 s
  *(v0.1.0)*.

### §6 — Collaboration / CI

Makes the editor usable in a team workflow instead of a single-author
tool.

- ⏳ **Site file diff viewer** — render two `.site.json` side-by-side
  with added/removed entities highlighted in the viewport. Makes PRs
  reviewable without Gazebo.
- ⏳ **Screenshot-on-save** to `docs/` using the headless render path
  that `--export_sdf` already depends on.
- ⏳ **`--validate` CLI mode** — run all §1 linters without UI and exit
  non-zero on errors. Plug into CI to stop merging broken nav graphs.

### Progress

| Category | Total | Done |
|---|---|---|
| §1 Validation | 4 | 2 |
| §2 Live round-trip | 2 | 0 |
| §3 Scenario preview | 3 | 3 |
| §4 Drawing UX | 3 | 2 |
| §5 UX polish | 6 | 6 |
| §6 Collaboration | 3 | 0 |
| **Total** | **21** | **13** |

---

## Known Issues

- **style CI clippy**: uses `-W clippy::all` (warn only) due to remaining upstream warnings
- **ci_linux / ci_windows disabled**: only run manually via workflow_dispatch
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
- **Preferences**: `UserPreferences` resource → `~/.config/rmf_site_editor/preferences.json`
- **Graph View**: `NavGraphViewMode` resource, `ToggleNavGraphView` event (F4), saves/restores `CategoryVisibility`
- **Lane types**: `LaneType` enum (Robot/Human), `ChangePlugin<LaneType>`, `update_lane_type_visuals` system
- **Color coding**: Graph View uses `graph_view_*_material` in `SiteAssets`, locations colored by `LocationTag`
