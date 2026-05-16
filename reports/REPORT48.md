# Title

Canvas eraser image-removal support report

# Context

- Problem:
  - The eraser tool already removed strokes, shapes, text items, and imported PDF page placements, but it did not remove normal pasted images.
  - That made raster-backed canvas content inconsistent: some image-like items were eraseable and others were not.
- Constraints:
  - This needed to stay a bounded eraser-hit-testing slice.
  - The repo still has unrelated local edits in `src/app/shell.ts`, `src/styles.css`, `TODO.md`, `.gitignore`, and generated `dist/` files, so the commit needed to stay scoped to the eraser change only.

# Goals

- Primary success criteria:
  - make the eraser remove pasted image items
  - keep imported PDF page removal working
  - avoid changing broader selection or deletion behavior
- Secondary success criteria:
  - keep erase semantics consistent with the existing bounding-box style used for PDF page removal

# Approach

- Chosen approach:
  - extend the existing `eraseAtPoint()` path with an image loop parallel to the already-existing PDF page loop
  - use the same simple padded bounding-box hit logic already used for imported PDF page placements
- Rejected option:
  - routing image deletion through selection workflows would have widened the scope and violated the purpose of the eraser tool

# Implementation

- Task hash:
  - `extend-eraser-to-images`
- Matching task file:
  - [todos/extend-eraser-to-images](/home/rok/sync/ideas/rpdf2/todos/extend-eraser-to-images:1)
- Architecture / flow:
  - `eraseAtPoint()` now checks `images[]` first, using the same `eraserRadius` padding strategy already used for PDF page placements.
  - When the pointer falls within the padded image bounds, that image is removed from the canvas item list.
  - The rest of the eraser pipeline remains unchanged: PDF pages, shapes, text, and strokes still use their existing removal logic.
- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:1717)
    - added image hit-testing and removal inside `eraseAtPoint()`

# Results

- Outputs:
  - Pasted image items are now eraseable with the eraser tool.
  - Imported PDF page placements remain eraseable through the same path.
  - The change stays local to eraser behavior and does not alter selection, resize, or export logic.
- Verification:
  - Ran `npm run build`
    - result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - result: success

# Decisions

- Fact:
  - The eraser already used a padded bounds check for imported PDF page placements.
- Assessment:
  - Reusing that same approach for images is the least surprising behavior and keeps raster-backed item removal consistent.

# Limitations

- I did not manually test the live app in this turn, so the remaining hands-on confirmation is whether the eraser radius feels right on large and small pasted images.
- This slice does not add keyboard deletion or select-all workflows; those remain separate queued tasks.
- `TODO.md` was not updated in this commit because it already contains unrelated local backlog edits.

# Next steps

1. Manually test erasing a pasted image and an imported PDF page in Canvas Mode.
   - This confirms the live hit feel of the padded bounds logic.
2. Take the next selection keyboard task, [todos/add-delete-selection-shortcut](/home/rok/sync/ideas/rpdf2/todos/add-delete-selection-shortcut:1), if you want another bounded canvas-input slice next.
3. If the eraser hit box feels too generous on images, add a small follow-up task to tune raster erase hit-testing separately from PDF page erase behavior.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Tauri/Rust backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch the app, switch to Canvas Mode, paste an image or import a PDF page, then verify the eraser removes those items when dragged across them.
