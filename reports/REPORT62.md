# Title

Add Ctrl-held grid snapping for canvas move operations

# Context

The canvas already supported moving selected items with the select tool, but movement was purely freeform. The requested behavior for this task was temporary snapping only while `Ctrl` is held during a move, not a permanently enabled snapping mode.

The important constraint was that this repo already had unrelated uncommitted edits in `TODO.md`, `src/features/canvas/workspace.ts`, `src/app/shell.ts`, `src/app/types.ts`, `src-tauri/src/domain/canvas.rs`, and `src/styles.css`. That meant the implementation needed to stay tightly scoped to the move path and avoid mixing in adjacent canvas work.

# Goals

- Add move-time grid snapping for selected movable items.
- Activate snapping only while `Ctrl` is held during the drag.
- Return immediately to freeform movement when `Ctrl` is released during the same drag.
- Apply snapping consistently across the movable item types already supported by the select tool.
- Pass `npm run build` and `cargo check --manifest-path src-tauri/Cargo.toml`.

# Approach

I replaced the incremental move path with a start-anchored move session for selection drags.

This was the smallest correct shape for temporary snapping because incremental mutation cannot cleanly toggle between snapped and unsnapped movement in the middle of one drag. A start-anchored session can recompute the moved positions from the original drag state every frame.

I used the same 50-unit spacing the canvas already uses for its visible background grid. Snapping is applied to the dragged selection's top-left corner.

# Implementation

- Task hash:
  - `add-control-grid-snapping-for-move`
- Matching task file:
  - `todos/add-control-grid-snapping-for-move`
- Key files:
  - `TODO.md`
    - moved the task into `# Current TODOs` before implementation and into `# Done TODOs` after verification
  - `src/features/canvas/workspace.ts`
    - added `MoveSession` to capture:
      - drag origin point
      - original selection bounds
      - original geometry/positions for each selected item
    - added `MOVE_SNAP_GRID_SIZE = 50`
    - replaced incremental `moveSelectedItems(deltaX, deltaY)` behavior with `applyMoveSession(...)`
    - recomputed moved positions from the original drag state on every pointer move
    - when `event.ctrlKey` is true during move, snapped the dragged selection’s top-left corner to the 50-unit grid
    - updated pointer down/up/cancel and snapshot-reset paths to use `moveSession` instead of the previous incremental move anchor state

The snapping applies across the existing movable selection types:
- strokes
- shapes
- text items
- pasted images
- imported PDF page placements

# Results

Observed results:
- Move operations now recompute from the drag start instead of accumulating per-frame deltas.
- Holding `Ctrl` during a move snaps the moved selection to the canvas grid.
- Releasing `Ctrl` during the same move returns to freeform movement immediately because positions are recomputed from the original drag state.
- The snap grid size is 50 world units, matching the canvas background grid step already used in `drawGrid()`.

Verification:
- `npm run build`
- `cargo check --manifest-path src-tauri/Cargo.toml`

Both commands passed.

# Decisions

- Fact:
  - The canvas grid renderer already uses a 50-unit step.
- Assessment:
  - Reusing 50 for move snapping keeps the feature interpretable and avoids introducing a second independent grid size.

- Fact:
  - Temporary modifier-based snapping must be reversible during one drag.
- Assessment:
  - A move session anchored to the original drag state is the narrowest reliable implementation; incremental mutation would accumulate drift when `Ctrl` is toggled mid-drag.

# Limitations

- I verified this at build/compile level, not by manually dragging in the running Tauri app.
- Snapping is based on the dragged selection’s top-left corner, not on more advanced alignment heuristics.
- This task does not add resize snapping or edge guides.
- The repo still contains unrelated uncommitted edits outside this task.

# Next steps

1. Implement `todos/add-edge-alignment-guides-for-move` next.
   - Why: it is now the next canvas movement refinement in `TODO.md`.
   - Depends on: keeping the guide logic isolated from the now-updated move session path.
2. Manually runtime-test Ctrl-snapped movement.
   - Why: confirms that snapping feels correct for mixed item types and that toggling `Ctrl` mid-drag behaves as intended.
   - Depends on: launching the canvas workspace locally.

# Reproducibility

1. Inspect:
   - `src/features/canvas/workspace.ts`
   - `todos/add-control-grid-snapping-for-move`
   - `TODO.md`
2. Verify:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Manual runtime check:
   - launch the app
   - select one or more movable canvas items
   - drag normally and confirm freeform movement
   - hold `Ctrl` during the drag and confirm the selection snaps to the 50-unit grid
   - release `Ctrl` during the same drag and confirm movement returns to freeform
