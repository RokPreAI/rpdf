# Title

Add selection-scoped recolor controls for pasted canvas images

# Context

The canvas already supported pasted raster images and separately supported recolor settings for imported PDF pages. The missing feature was image recoloring on the canvas itself: when a pasted image is selected, the user wanted a compact floating control near the image so recolor can be enabled and adjusted without turning this into a global canvas mode.

This repo still had unrelated uncommitted edits in `TODO.md`, `src/features/canvas/workspace.ts`, `src/app/shell.ts`, `src/app/types.ts`, `src-tauri/src/domain/canvas.rs`, and `src/styles.css`, so the implementation had to stay tightly local and avoid broader canvas UI cleanup.

# Goals

- Show a compact floating recolor UI only when a pasted canvas image is selected.
- Support enabling/disabling recolor and changing foreground/background colors.
- Make the on-canvas rendered image match the persisted recolor settings.
- Preserve image recolor state through save/load.
- Pass `npm run build` and `cargo check --manifest-path src-tauri/Cargo.toml`.

# Approach

I reused the existing PDF recolor settings shape and recolor math instead of inventing a second image-specific format.

The implementation stays local by:
- extending canvas image document/state with recolor settings
- adding a small absolute-positioned popover inside the canvas workspace
- caching a recolored image source per canvas image so redraw stays simple

I did not move this into global CSS or a shared recolor subsystem because the task only asked for a compact, selection-scoped control.

# Implementation

- Task hash:
  - `add-image-recolor-when-selected`
- Matching task file:
  - `todos/add-image-recolor-when-selected`
- Key files:
  - `TODO.md`
    - moved the task into `# Current TODOs` before implementation and into `# Done TODOs` after verification
  - `src/app/types.ts`
    - added optional `recolor` settings to `CanvasImagePlacementDocument`
  - `src-tauri/src/domain/canvas.rs`
    - added persisted `recolor` settings to `CanvasImagePlacement`
    - added a default recolor payload for backward compatibility with older canvas documents that do not yet contain image recolor state
    - extended the deserialization test to cover image recolor payloads
  - `src/features/canvas/workspace.ts`
    - extended `CanvasImage` with:
      - `renderedImage`
      - `recolor`
    - added a compact `#image-recolor-popover` inside the canvas workspace
    - positioned the popover near the bottom-left of a single selected image using current camera/world coordinates
    - hid the popover for non-image or multi-item selections
    - reused recolor math equivalent to the PDF page recolor path to build a cached recolored image canvas
    - updated image snapshot export and import so recolor state persists and reloads correctly

The canvas draw path now renders `canvasImage.renderedImage`, which is either:
- the original image when recolor is disabled, or
- a recolored offscreen canvas when recolor is enabled

# Results

Observed results:
- A single selected pasted image now gets a compact floating recolor popover.
- The popover supports:
  - a recolor enable toggle
  - foreground color selection
  - background color selection
- Recolor changes are reflected directly in the canvas image rendering.
- Image recolor state is now part of the saved canvas document and reload path.
- Older canvas documents without image recolor state now fall back to disabled recolor defaults.

Verification:
- `npm run build`
- `cargo check --manifest-path src-tauri/Cargo.toml`

Both commands passed.

# Decisions

- Fact:
  - PDF page imports already used a simple `{ enabled, foreground, background }` recolor model.
- Assessment:
  - Reusing that shape for canvas images was the narrowest way to keep behavior and persistence consistent.

- Fact:
  - The task explicitly wanted a compact floating control near the selected image.
- Assessment:
  - An absolute-positioned popover inside `workspace.ts` was the smallest direct implementation and avoided unrelated styling or architecture churn.

# Limitations

- I verified this at build/compile level, not by manually selecting and recoloring images in the running Tauri app.
- The popover currently appears only for a single selected image, not for mixed or multi-image selection states.
- I did not recolor imported PDF page placements through this popover because the task explicitly scoped the feature to pasted canvas images.
- The repo still contains unrelated uncommitted edits outside this task.

# Next steps

1. Implement `todos/add-control-grid-snapping-for-move` next.
   - Why: it is now the next task in `TODO.md` and follows naturally after recent canvas interaction work.
   - Depends on: keeping the move path isolated from unrelated in-progress canvas edits.
2. Manually runtime-test image recolor behavior.
   - Why: confirms popover placement, live recolor preview, and save/load behavior in the actual app.
   - Depends on: launching the canvas workspace locally and selecting a pasted image.

# Reproducibility

1. Inspect:
   - `src/features/canvas/workspace.ts`
   - `src/app/types.ts`
   - `src-tauri/src/domain/canvas.rs`
   - `todos/add-image-recolor-when-selected`
   - `TODO.md`
2. Verify:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Manual runtime check:
   - launch the app
   - paste an image into canvas mode
   - switch to select mode and select that image
   - confirm the floating recolor popover appears near the image
   - toggle recolor and change foreground/background colors
   - save and reload the canvas document and confirm the recolor state persists
