# Title

Settings panel and iconized controls report

# Context

After the reading pipeline cycle, the remaining low-complexity task in the requested `0-5` band was `add-config-and-toolbar-icons`. The app already had enough interactive surface area that text-only controls and hard-coded defaults were starting to make the UI feel heavier than it needed to be.

This was the right next slice because:

- it was the smaller of the two remaining `0-5` tasks
- it stayed local to the existing shell and mode workspaces
- it improved usability without reopening architecture or persistence design
- it created a simple preferences path that later tasks can reuse

# Goals

- Primary success criteria:
  - add a compact persisted settings surface
  - make current actions visually identifiable with icons
  - keep the UI small rather than turning it into a large preferences system

- Secondary success criteria:
  - let the current workspaces react to changed settings immediately
  - keep the preferences path local and offline
  - store likely future PDF defaults without forcing the recolor task into this cycle

# Approach

- Chosen approach:
  - Add a small shell-level settings panel backed by `localStorage`.
  - Persist a narrow preferences set:
    - default stroke width
    - default shape kind
    - default pen color
    - speech rate
    - future PDF recolor defaults
  - Broadcast preference changes with a window event so mounted workspaces can apply them without adding a large shared state layer.
  - Replace text-only control labels with compact icon + text pairings across the shell and the most active workspace buttons.

- Rejected options:
  - A full preferences subsystem with backend file storage would have widened scope beyond the task.
  - Icon-only controls would have hurt clarity; the first pass keeps labels and adds icons rather than removing text.
  - Deferring settings until after every remaining feature would leave tool defaults scattered and hard-coded.

# Implementation

- `src/app/shell.ts`
  - added a compact settings panel behind a `Settings` action
  - added persisted preferences under `rpdf.preferences.v1`
  - added live preference broadcasting with `rpdf:preferences-changed`
  - added iconized mode and save/load/settings controls

- `src/features/canvas/workspace.ts`
  - toolbar buttons now use real symbolic tool icons instead of letter placeholders
  - canvas workspace reads initial default stroke width, shape kind, and pen color from preferences
  - canvas workspace now reacts to preference changes while mounted

- `src/features/pdf/workspace.ts`
  - PDF action controls now have iconized labels
  - local speech playback now honors persisted speech-rate preferences
  - PDF workspace reacts to speech-rate updates while mounted

- `src/styles.css`
  - added settings-panel styling
  - added shared icon/button layout helpers
  - styled compact settings fields and color rows

# Results

- Outputs:
  - The app now has a compact settings surface for current defaults.
  - Core user preferences now persist locally and reload on startup.
  - Current tool and mode controls are visually easier to scan because they no longer rely on text only.
  - `TODO.md` now marks `add-config-and-toolbar-icons` done and leaves `add-pdf-page-import-and-recolor` as the only remaining unfinished task in the requested band.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - Preferences are stored locally in the frontend instead of a backend config file.
  - Assessment:
  - This is the smallest reliable persistence path for the current app stage and satisfies the task’s “simple configuration surface” goal.

- Fact:
  - Controls now use icon + text rather than icon-only.
  - Assessment:
  - This improves scanability without sacrificing clarity or accessibility.

- Fact:
  - Recolor defaults are stored now, but full recolor behavior remains in the separate PDF import/recolor task.
  - Assessment:
  - This keeps the UX/config slice bounded while still preparing for the remaining feature work.

# Limitations

- The settings panel is intentionally small and does not yet cover every tool behavior.
- Preferences are stored in app-local browser storage rather than a dedicated user config file.
- Full PDF recolor behavior is still not implemented; only the defaults are persisted.

# Next steps

1. Complete `add-pdf-page-import-and-recolor`, which is now the only remaining unfinished task in the requested `0-5` band.
2. Reuse the stored recolor defaults when implementing that page-import flow.
3. Consider moving preferences to a backend file later only if the local path becomes limiting.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Rust backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Inspect the shell settings implementation:
   - `src/app/shell.ts`
4. Inspect the canvas icon/default wiring:
   - `src/features/canvas/workspace.ts`
5. Inspect the PDF speech-rate preference wiring:
   - `src/features/pdf/workspace.ts`
