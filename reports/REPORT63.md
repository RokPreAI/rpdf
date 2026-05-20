# Title

Add edge alignment guides for canvas move operations

# Context

The previous task added Ctrl-held grid snapping by converting selection movement to a start-anchored move session. The next indexed task was to add move-time edge alignment assistance so dragged items can line up against nearby existing content.

The repository still contained unrelated uncommitted edits in tracked files, so this task needed to stay local to canvas move/render logic plus the required TODO/report bookkeeping.

# Goals

- Add alignment assistance while moving selected items.
- Support obvious edge cases first, especially left-edge and top-edge alignment.
- Keep the first pass limited to move-time edge alignment, not full smart layout.
- Provide visible guidance so the user can understand why snapping occurred.
- Pass:
  - `npm run build`
  - `cargo check --manifest-path src-tauri/Cargo.toml`

# Approach

I kept the existing start-anchored move session from the previous task and extended it with a narrow alignment pass.

During a drag, after the raw move delta is computed, the code now:
1. computes the moved selection bounds,
2. compares that bounds box against all non-selected canvas items,
3. looks for nearby edge matches on the x and y axes,
4. applies the closest x and y corrections within a small threshold, and
5. stores visible guide lines for the current drag frame.

This keeps the behavior reversible during one drag and avoids mutating movement incrementally.

# Implementation

- Task hash:
  - `add-edge-alignment-guides-for-move`
- Matching task file:
  - `todos/add-edge-alignment-guides-for-move`
- Files changed:
  - `src/features/canvas/workspace.ts`
  - `TODO.md`
  - `reports/REPORT63.md`

## `src/features/canvas/workspace.ts`

Added:
- `AlignmentGuide` type for temporary move-time guide rendering.
- `MOVE_ALIGNMENT_THRESHOLD_PX = 10` for proximity testing in screen space.
- `activeAlignmentGuides` state cleared outside active drags.
- `offsetBounds(...)` helper.
- `allCanvasTargets()` helper to iterate candidate non-selected items.
- `drawAlignmentGuides()` to render visible horizontal/vertical guide lines.
- `resolveMoveAlignment(bounds)` to:
  - ignore currently selected items,
  - inspect nearby edge matches,
  - choose the closest vertical and horizontal alignment candidates,
  - return both the move correction and the guide line geometry.

Changed:
- `applyMoveSession(...)`
  - still starts from the drag origin,
  - still applies optional Ctrl grid snapping first,
  - now also applies nearest edge-alignment snapping against other item bounds,
  - now records visible guide lines for the current move frame.
- `redraw()`
  - now draws alignment guides before selection overlays.
- pointer/session cleanup paths
  - now clear `activeAlignmentGuides` when drag state ends or is replaced.

Supported first-pass edge matches:
- left to left
- left to right
- right to left
- right to right
- top to top
- top to bottom
- bottom to top
- bottom to bottom

This applies to the moved selection bounds against all non-selected item bounds, regardless of whether the moved content is strokes, shapes, text, images, or imported PDF pages.

## `TODO.md`

- Marked `add-edge-alignment-guides-for-move` done.

# Why this was the right next task

- It was the only remaining indexed TODO.
- It had a concrete task file with acceptance criteria.
- `REPORT62.md` explicitly called it out as the next follow-up.
- The move-session rewrite from the previous task made this addition local and implementable in one pass.

# Verification

Ran:
- `npm run build`
- `cargo check --manifest-path src-tauri/Cargo.toml`

Both passed.

# Results

The canvas move path now provides visible alignment assistance:
- dragging near another item’s left/right edge can snap x alignment,
- dragging near another item’s top/bottom edge can snap y alignment,
- matching guide lines appear while the alignment is active,
- guides disappear when the drag ends or is canceled.

# Limitations

- This was verified at build/check level, not by manual runtime dragging in the app.
- The first pass is edge-only; it does not add center alignment, spacing/distribution, or resize guides.
- If several nearby candidates exist, the code chooses the closest horizontal and closest vertical match independently.
- The repo still contains unrelated uncommitted edits outside this task.

# Best next follow-up task

There is no remaining indexed task in `TODO.md` after this change.

Best follow-up:
1. add a new focused task for manual/runtime validation of move snapping behavior
   - Why: both recent move-related tasks were verified by build/check only, not by interactive canvas use
   - Scope: confirm Ctrl grid snapping and edge-guide snapping feel correct across mixed item types and mixed multi-selection cases
2. if runtime behavior exposes ambiguity, split the next refinement narrowly
   - examples: center alignment, suppressing edge matches that feel too aggressive, or prioritizing same-edge matches over opposite-edge matches

# Reproducibility

1. Inspect:
   - `todos/add-edge-alignment-guides-for-move`
   - `src/features/canvas/workspace.ts`
   - `TODO.md`
2. Verify:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Manual runtime check:
   - launch the app
   - place two or more items on the canvas
   - drag one item near another item’s top or left edge
   - confirm snapping occurs near the edge match
   - confirm a visible horizontal or vertical guide line appears during the snap
