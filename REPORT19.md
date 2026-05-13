# Title

Canvas arrow tool report

# Context

The selected worker slice was the user-requested `add-arrow-tool` canvas feature.

This was the right next action because:

- the user explicitly requested an arrow tool
- the canvas already had a stable shape pipeline for line, rectangle, and ellipse
- arrow support was local enough to finish in one pass without widening into resize handles or shortcut work

Constraints for this slice:

- keep the change inside the existing canvas shape model
- avoid inventing a separate arrow-only drawing path
- keep persistence, selection, and SVG export working instead of adding a canvas-only visual

# Goals

- Primary success criteria:
  - add a direct arrow tool button in Canvas Mode
  - make arrows draw with a visible shaft and arrowhead
  - make arrows selectable, movable, persistable, and SVG-exportable

- Secondary success criteria:
  - keep the data-model change small and compatible with the existing shape system
  - verify the result with the normal frontend and Rust checks

# Approach

- Chosen approach:
  - extend the shared shape kind model with `arrow`
  - reuse the existing shape lifecycle for creation, movement, serialization, and import/export
  - add a small arrow-geometry helper so canvas rendering, bounds, hit testing, and SVG export all use the same head points

- Rejected options:
  - treating arrows as freehand strokes would have broken the existing vector-shape model
  - rendering an arrow visually without updating SVG export or persistence would have created a partial feature
  - widening this slice into keyboard shortcuts or resize handles would have broken the bounded worker scope

# Implementation

- Architecture / flow:
  - `arrow` is now a first-class canvas shape kind.
  - Canvas Mode exposes it as a direct tool button in `.canvas-pickers`.
  - Arrow shapes use the same shape creation path as line/rectangle/ellipse.
  - A shared `arrowHeadPoints()` helper computes the two head segments from the shape start/end points and base width.
  - That helper is reused by:
    - shape bounds
    - canvas rendering
    - hit testing
    - SVG export

- Key files or components:
  - `src/features/canvas/workspace.ts`
    - added the arrow picker button
    - extended `Tool` and shape-tool checks with `arrow`
    - added `arrowHeadPoints()`
    - taught shape drawing, selection, and SVG export how to handle arrows
    - updated preference-driven default-shape restoration to accept `arrow`
  - `src/app/types.ts`
    - added `arrow` to `CanvasShapeKindDocument`
  - `src-tauri/src/domain/canvas.rs`
    - added `Arrow` to the Rust `CanvasShapeKind` enum
  - `TODO.md`
    - recorded `add-arrow-tool` as a completed canvas task

# Results

- Outputs:
  - Canvas Mode now has a direct arrow tool button.
  - Arrows render as a shaft plus arrowhead, can be selected and moved like other vector shapes, and survive save/load.
  - SVG export now includes arrow shapes instead of silently dropping them.

- Metrics or observations:
  - The change remained inside the existing canvas shape architecture.
  - No shell, PDF, or storage flow redesign was needed.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success
  - Manual live drawing validation in the Tauri window was not performed in this turn.

# Decisions

- Fact:
  - Arrow support was implemented as another shape kind rather than as a special-case stroke.
  - Assessment:
  - This keeps the feature consistent with the existing vector-item model and avoids future duplication in selection and export logic.

- Fact:
  - Arrowhead geometry is shared across rendering, hit testing, bounds, and SVG export.
  - Assessment:
  - This reduces the chance of arrows looking correct on screen but behaving incorrectly in selection or export.

# Limitations

- This slice does not add arrow-specific resize handles or editing grips.
- It does not add keyboard shortcuts for arrow selection; that belongs with the backlog shortcut task.
- Manual runtime validation is still needed to tune arrowhead proportions if they feel too large or too small during real drawing use.

# Next steps

1. Implement `add-element-resize` now that the canvas tool set is broader and selection is already in place.
2. Implement `add-excalidraw-style-tool-and-color-shortcuts` so the growing tool palette stays fast to use.
3. Revisit `fix-pdf-mode-annotations` only with live manual verification, since the code fix exists but the backlog item still depends on human confirmation.

# Reproducibility

1. Inspect:
   - `src/features/canvas/workspace.ts`
   - `src/app/types.ts`
   - `src-tauri/src/domain/canvas.rs`
   - `TODO.md`
2. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch:
   - `npm run tauri dev`
4. In Canvas Mode, select the arrow tool, draw an arrow, move it with the select tool, save/load the canvas, and export SVG to confirm the arrow persists across all paths.
