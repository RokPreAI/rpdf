# Title

Fix zoom-insensitive hand-tool drag panning on the canvas

# Context

The canvas workspace already supported a dedicated hand tool, temporary spacebar panning, and middle/right-button drag panning. The reported problem was that hand-tool dragging felt wrong because panning was driven by raw pointer movement deltas instead of an explicit grabbed world-point anchor. The intended behavior for this task was direct drag panning: when the user grabs a point on the canvas, that same world point should stay under the cursor while dragging, regardless of zoom level.

This repo still had unrelated uncommitted edits in `TODO.md`, `src/features/canvas/workspace.ts`, `src/app/shell.ts`, `src/app/types.ts`, `src-tauri/src/domain/canvas.rs`, and `src/styles.css`, so the implementation had to stay tightly local to the pan path.

# Goals

- Make hand-tool panning keep the grabbed world point under the cursor.
- Preserve the existing hand-tool entry paths:
  - explicit pan tool
  - hold-space drag pan
  - middle-button drag pan
  - right-button drag pan
- Keep the change limited to panning feel and camera math.
- Pass `npm run build` and `cargo check --manifest-path src-tauri/Cargo.toml`.

# Approach

I replaced the boolean `isPanning` state with a pointer-specific `PanSession` that stores the world point grabbed at pan start.

That lets pointer move handling compute camera translation from first principles:
- read the current pointer screen position inside the canvas
- keep the originally grabbed world point fixed
- solve `camera.x` and `camera.y` so that world point still maps to the pointer screen position

I did not touch selection, snapping, or any other movement behavior because the task only asked for panning feel.

# Implementation

- Task hash:
  - `fix-hand-tool-drag-pan-scaling`
- Matching task file:
  - `todos/fix-hand-tool-drag-pan-scaling`
- Key files:
  - `TODO.md`
    - moved the task into `# Current TODOs` before implementation and into `# Done TODOs` after verification
  - `src/features/canvas/workspace.ts`
    - added `PanSession` with:
      - `pointerId`
      - `anchorWorldPoint`
    - replaced `isPanning` state with `panSession`
    - updated cursor handling so active pan capture shows `grabbing`
    - on pan start, store the world point under the pointer in `panSession`
    - on pointer move during pan, compute:
      - `camera.x = pointerScreenX - anchorWorldPoint.x * camera.scale`
      - `camera.y = pointerScreenY - anchorWorldPoint.y * camera.scale`
    - clear `panSession` on pointer up, pointer cancel, and document import/reset paths

The practical effect is that panning is now anchored to the grabbed scene position instead of incrementally applying `event.movementX` and `event.movementY`.

# Results

Observed results:
- The pan path now uses an explicit world-anchor model instead of accumulated browser movement deltas.
- The code keeps one specific world point attached to the pointer while the pan drag is active.
- The existing pan entry paths still go through the same `selectedTool === "pan" || isSpaceDown || event.button === 1 || event.button === 2` gate.

Verification:
- `npm run build`
- `cargo check --manifest-path src-tauri/Cargo.toml`

Both commands passed.

# Decisions

- Fact:
  - The old implementation updated camera translation with `event.movementX` and `event.movementY`.
- Assessment:
  - Recomputing camera translation from the grabbed world point is a more explicit and reliable way to satisfy the “same world point stays under the cursor” requirement.

- Fact:
  - The task was about panning feel only.
- Assessment:
  - Replacing only the pan state and move math was the narrowest change that met the acceptance criteria without entangling selection or resize logic.

# Limitations

- I verified this at build/compile level, not by manually dragging in the running Tauri app.
- The repo still contains unrelated uncommitted edits outside this task.
- I did not change any touchpad or wheel navigation behavior because that is outside the scope of drag panning.

# Next steps

1. Implement `todos/add-image-recolor-when-selected` next.
   - Why: it is now the first remaining priority-2 canvas task in `TODO.md`.
   - Depends on: careful isolation from the current in-progress canvas text-resize edits.
2. Manually runtime-test pan behavior at multiple zoom levels.
   - Why: confirms the subjective drag feel and verifies hold-space, middle-button, and explicit hand-tool workflows in the real app.
   - Depends on: launching the canvas workspace locally.

# Reproducibility

1. Inspect:
   - `src/features/canvas/workspace.ts`
   - `todos/fix-hand-tool-drag-pan-scaling`
   - `TODO.md`
2. Verify:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Manual runtime check:
   - launch the app
   - zoom in and drag with the pan tool
   - repeat with hold-space drag
   - repeat with middle/right-button drag
   - confirm the grabbed canvas point stays under the cursor during the drag
