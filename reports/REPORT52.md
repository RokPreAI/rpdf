# Title

Canvas double-space fit-view shortcut report

# Context

- Problem:
  - The canvas already had wheel zoom, `Ctrl`+pen drag zoom, and `Space` hold pan, but it lacked a quick way to reframe the camera around all current content.
  - The user wanted a double-press `Space` shortcut that behaves like a fit-view action and makes all strokes, shapes, text, images, and imported PDF pages visible at once.
- Constraints:
  - This needed to stay a narrow camera-control slice and not break the existing single-press or hold-`Space` pan behavior.
  - The repo still has unrelated local changes in `src/app/shell.ts`, `src/styles.css`, `TODO.md`, `.gitignore`, generated `dist/` files, and a stray root `REPORT1.md`, so the commit needed to stay scoped to the canvas workspace plus the task/report artifacts.

# Goals

- Primary success criteria:
  - double pressing `Space` fits all current canvas content into the viewport
  - the fit logic includes strokes, shapes, text items, pasted images, and imported PDF pages
  - existing hold-`Space` pan still works
- Secondary success criteria:
  - empty-canvas fit is safe and does not produce invalid camera state
  - the shortcut is discoverable in the existing toolbar hint

# Approach

- Chosen approach:
  - add a small double-press detection window on the existing `Space` keyboard path
  - compute aggregate world bounds across every drawable or placed canvas item type
  - derive a centered camera translation and zoom from those bounds using viewport-aware padding
- Why this was the right next slice:
  - it extends the current camera system directly instead of introducing a parallel navigation flow
  - it is fully local to Canvas Mode and easy to verify with existing build checks
- Rejected option:
  - binding fit-view to a new dedicated button or a different key would have been heavier and less aligned with the requested workflow than extending `Space`

# Implementation

- Task hash:
  - `add-fit-view-space-shortcut`
- Matching task file:
  - [todos/add-fit-view-space-shortcut](/home/rok/sync/ideas/rpdf2/todos/add-fit-view-space-shortcut:1)
- Architecture / flow:
  - The canvas toolbar hint now advertises `Double Space: fit view`.
  - `allCanvasContentBounds()` collects a single world-space bounds box across strokes, shapes, text, raster images, and imported PDF page placements.
  - `fitCameraToBounds()` computes a safe scale and translation from those bounds using viewport-relative padding and clamps the result to the existing camera zoom limits.
  - The `Space` keyboard handler now detects a non-repeat second press inside a short timing window and runs fit-view without removing the existing `isSpaceDown` pan behavior.
- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:190)
    - added fit-view timing and padding constants
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:223)
    - updated the shortcut hint with `Double Space: fit view`
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:1254)
    - added aggregate content-bounds collection and camera-fit helpers
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:3645)
    - added double-press `Space` detection inside the keyboard handler

# Results

- Outputs:
  - Double pressing `Space` now reframes the canvas camera around all current content with padding instead of leaving content off-screen.
  - The fit logic covers strokes, shapes, text, pasted images, and imported PDF page placements.
  - Empty-canvas fit fails closed and leaves the camera unchanged.
  - Existing hold-`Space` pan behavior remains in place because the same `isSpaceDown` path is still used.
- Verification:
  - Ran `npm run build`
    - result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - result: success

# Decisions

- Fact:
  - The repo already had reusable world-space bounds helpers for individual canvas item kinds.
- Assessment:
  - Building one aggregate content-bounds helper was the simplest way to implement fit-view without special-casing the camera per content type.
- Fact:
  - `Space` was already the temporary-pan key, and the user explicitly wanted fit-view on a double press of that same key.
- Assessment:
  - Timing-based detection on the existing keyboard path is the smallest change that preserves the requested gesture and avoids new UI.

# Limitations

- I did not manually test the live Tauri app in this turn, so the remaining hands-on confirmation is whether the double-press window feels right and whether hold-`Space` pan still feels natural during real interaction.
- `TODO.md` was not updated in this commit because it still contains unrelated local backlog edits.

# Next steps

1. Manually test double-`Space` on canvases containing mixed content.
   - Confirm that fit-view includes strokes, shapes, text, images, and imported PDF pages, and that hold-`Space` pan still feels unchanged.
2. If the gesture feels too sensitive or too strict, tune `SPACE_DOUBLE_PRESS_WINDOW_MS`.
   - That should remain a tiny follow-up instead of a broader camera rewrite.
3. The next visible pending backlog item after this slice is [todos/evaluate-elevenlabs-tts-fallback-and-timing](/home/rok/sync/ideas/rpdf2/todos/evaluate-elevenlabs-tts-fallback-and-timing:1), unless you want to queue more canvas interaction work first.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Tauri/Rust backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch the app, create mixed canvas content, then double press `Space`.
   - Verify the full content set becomes visible and that holding `Space` still enables panning.
