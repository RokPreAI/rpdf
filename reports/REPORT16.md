# Title

Canvas select-tool reliability report

# Context

The selected worker slice was `fix-select-tool`. The current canvas selection baseline existed, but the backlog still marked it as unreliable and the latest report named it as the next bounded interaction fix.

This was the right next action because:

- it was the top remaining priority-1 task in `TODO.md`
- `REPORT15.md` called it out as the next canvas interaction slice
- it was a local frontend fix that could be completed without mixing in resize handles, shortcuts, or export redesign

Constraints for this slice:

- keep the work focused on existing selectable item types
- do not widen into the later `add-element-resize` task
- preserve current save/load behavior while improving mode-switch continuity for selection state

# Goals

- Primary success criteria:
  - make strokes easier to select by testing against drawn segments instead of only stored points
  - make shapes selectable by their visible body instead of only their border
  - respect vector stacking order when multiple vector items overlap
  - preserve the current selection through canvas mode switches

- Secondary success criteria:
  - keep selection recovery compatible with existing saved canvas files
  - avoid backend behavior changes beyond schema-safe optional fields
  - verify the result with the normal frontend and Rust build checks

# Approach

- Chosen approach:
  - improve canvas hit testing in place instead of replacing the selection model
  - add a transient canvas-selection snapshot so the shell can restore the selected item after a mode switch
  - add stable optional stroke IDs to the serialized canvas model so selected strokes can survive export/import cycles used by workspace switching

- Rejected options:
  - redesigning selection around multi-select or resize handles would have widened scope too far
  - keeping stroke selection index-only would fail once a canvas workspace was exported and reimported during mode switching
  - leaving hit testing point-only would keep long sparse strokes frustrating to select

# Implementation

- Architecture / flow:
  - Canvas snapshots now optionally carry a `selection` field alongside the existing document payload.
  - Strokes now export an optional stable `id`, and the Rust-side canvas schema accepts that field with a default for older files.
  - On import, the canvas workspace resolves the stored selection back to the current in-memory item arrays by `id`.
  - Hit testing now walks vector items in descending draw order so selection follows what is visually on top.
  - Stroke selection now checks distance to stroke segments.
  - Rectangle and ellipse selection now use their visible interior as selectable area instead of only edge proximity.

- Key files or components:
  - `src/features/canvas/workspace.ts`
    - added selection snapshot helpers
    - improved stroke and shape hit testing
    - changed vector hit testing to respect draw order
    - restored the selected item during `importDocument()`
  - `src/app/types.ts`
    - added `CanvasSelectionDocument`
    - added optional stroke `id`
    - added optional `selection` on canvas workspace snapshots
  - `src-tauri/src/domain/canvas.rs`
    - added optional `id` on `CanvasStroke` with `serde(default)` for compatibility

# Results

- Outputs:
  - Strokes can now be selected by clicking near the visible line path rather than only near raw sampled points.
  - Rectangles and ellipses can now be selected from their interior area.
  - Overlapping vector items now select in visual top-order instead of always preferring shapes before strokes.
  - The current canvas selection now survives shell-level mode switching.

- Metrics or observations:
  - The fix stayed localized to canvas workspace logic plus one schema-safe optional field on strokes.
  - Older saved canvas files remain loadable because the new stroke `id` is optional on both TypeScript and Rust sides.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success
  - Manual runtime selection testing across all item types was not performed in this turn.

# Decisions

- Fact:
  - Canvas selection snapshots are stored outside the core canvas document content.
  - Assessment:
  - This keeps selection as transient workspace state while still allowing mode-switch restoration.

- Fact:
  - Stroke IDs were added as optional serialized fields.
  - Assessment:
  - This was the smallest safe way to restore selected strokes after export/import-driven workspace switches.

- Fact:
  - Shape hit testing now includes interior area.
  - Assessment:
  - This matches the visible object better and makes the select tool predictable enough to support later resize work.

# Limitations

- This slice does not add resize handles or any new editing affordances beyond more reliable selection and movement.
- Manual runtime verification is still needed for pasted images, imported PDF pages, and overlapping mixed-content cases.
- File-based load paths do not restore selection unless the snapshot came from the in-memory workspace state that now carries the optional `selection` field.

# Next steps

1. Decide whether `fix-pdf-mode-annotations` can now be marked done after manual runtime confirmation.
2. Implement `fix-recolor-controls-layout-and-state` as the next tight PDF/UX cleanup slice.
3. Implement `add-element-resize` now that selection is more stable and can serve as the base interaction layer.

# Reproducibility

1. Inspect the selection changes:
   - `src/features/canvas/workspace.ts`
   - `src/app/types.ts`
   - `src-tauri/src/domain/canvas.rs`
2. Build the frontend:
   - `npm run build`
3. Check the Rust side:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
4. Launch the app:
   - `npm run tauri dev`
5. In Canvas Mode, verify selection for strokes, shapes, images, and imported PDF pages, then switch modes and confirm the selected item is still selected when returning.
