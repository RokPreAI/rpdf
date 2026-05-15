# Title

PDF recolor control layout and state report

# Context

The selected worker slice was `fix-recolor-controls-layout-and-state` from `TODO.md`.

This was the right next action because:

- it was still an open current task in the backlog
- the latest report explicitly called it out as the next tight PDF/UX cleanup slice
- the user had already reported the exact failure mode: the recolor controls were stacked vertically and their visible indicators did not reflect the currently selected colors

Constraints for this slice:

- keep the change local to the PDF recolor controls
- do not widen into broader PDF layout, annotation, or export work
- preserve the existing recolor algorithm and import behavior

# Goals

- Primary success criteria:
  - make the recolor controls render as a compact horizontal or wrapped-horizontal row
  - make the visible recolor indicators track the actual selected foreground and background colors
  - keep that state stable through document refreshes, page switches, and imported PDF session restores

- Secondary success criteria:
  - remove the PDF workspace's dependency on the old generic settings-panel styles
  - keep the implementation easy to verify with the standard build checks

# Approach

- Chosen approach:
  - replace the generic `settings-*` recolor markup with PDF-specific controls
  - add explicit foreground/background swatches and hex-value readouts that are updated from the live recolor state
  - keep `syncRecolorControls()` as the single source of truth for repainting the control state

- Rejected options:
  - only changing CSS layout would not solve the missing selected-color feedback
  - relying on native `input[type="color"]` rendering alone would still make the active state inconsistent across platforms and themes
  - widening this slice into a full PDF sidebar redesign would have broken the worker-task scope

# Implementation

- Architecture / flow:
  - The PDF recolor card now has a local toggle row and a dedicated recolor control row.
  - Each color control shows:
    - the native color input
    - a visible swatch chip
    - the active hex value
  - `syncRecolorControls()` now updates all three state surfaces and dims the color row when recolor is disabled.

- Key files or components:
  - `src/features/pdf/workspace.ts`
    - replaced the old generic recolor markup with `pdf-recolor-*` elements
    - added references for recolor swatches and value readouts
    - extended `syncRecolorControls()` so the UI mirrors the live recolor state on every render path
  - `src/styles.css`
    - removed the now-unused generic `settings-*` styles
    - added compact PDF-specific recolor layout and indicator styles

# Results

- Outputs:
  - Recolor controls are now arranged in a horizontal wrapping row instead of a vertical stacked block.
  - Foreground and background indicators visibly update with the active recolor values.
  - Recolor state remains synced when the workspace rerenders through page/document/session changes because the update still flows through `syncRecolorControls()`.

- Metrics or observations:
  - The slice stayed entirely in frontend TypeScript and CSS.
  - The recolor rendering algorithm itself was untouched.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success
  - Manual live-window verification of the recolor control appearance was not performed in this turn.

# Decisions

- Fact:
  - The recolor indicators are now explicit swatches and hex labels instead of implicit browser-native control rendering.
  - Assessment:
  - This makes the active state much easier to read and avoids depending on platform-specific color-input visuals.

- Fact:
  - The PDF recolor card now uses PDF-scoped classes instead of the old shared settings classes.
  - Assessment:
  - This keeps the PDF workspace from inheriting shell-era layout assumptions that no longer fit the current UI.

# Limitations

- This slice does not change the recolor algorithm itself.
- It does not manually verify the live Tauri window at narrow widths; only build checks were run here.
- If the user wants the recolor controls even denser, that would be a separate UI-compaction pass.

# Next steps

1. Implement `add-recent-pdf-quick-open-list` as the next bounded PDF workflow improvement.
2. Implement `add-element-resize` now that selection behavior is more reliable.
3. Revisit `harden-responsive-layout-under-window-resize` with manual runtime verification, since the user explicitly marked that area as needing human intervention.

# Reproducibility

1. Inspect:
   - `src/features/pdf/workspace.ts`
   - `src/styles.css`
   - `TODO.md`
2. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch:
   - `npm run tauri dev`
4. In PDF Mode, enable recolor and change both colors. Confirm that the controls stay in a horizontal row, the swatches update immediately, and the same values are still shown after page refreshes or reopening a saved PDF session.
