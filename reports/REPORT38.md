# Title

Canvas text tool implementation report

# Context

- Problem:
  - Canvas Mode did not support typed text at all, even though the product direction and backlog expected users to be able to write directly on the canvas.
- Constraints:
  - This had to stay a bounded first pass.
  - The task required real canvas persistence and export support, not just a temporary overlay.
  - The repo still contains unrelated staged deletions and generated `dist/` churn, so this worker slice had to stay path-limited.

# Goals

- Primary success criteria:
  - add a text tool in Canvas Mode
  - let the user type and place visible text on the canvas
  - persist text through save/load and mode-switch snapshots
  - include text in selection, movement, and SVG export
- Secondary success criteria:
  - keep the first pass simple
  - avoid turning this into a full rich-text or text-editing subsystem

# Approach

- Chosen approach:
  - add `text` as a first-class canvas document item alongside strokes and shapes
  - use a lightweight positioned `textarea` as the entry surface, then commit its contents into the canvas model
  - extend the existing selection, move, serialization, and SVG-export paths to understand text items
- Rejected options:
  - a prompt-only text flow would have satisfied typing narrowly but would have been a poor fit for spatial placement and would not have felt like part of the canvas
  - an always-live DOM text layer would have widened the slice into a separate rendering system

# Implementation

- Task hash:
  - `add-text-tool`
- Matching task file:
  - [todos/add-text-tool](/home/rok/sync/ideas/rpdf2/todos/add-text-tool:1)
- Architecture / flow:
  - Added `CanvasTextDocument` / `CanvasText` to the frontend and Rust canvas document models.
  - Added a `Text` tool button to the Canvas toolbar.
  - Clicking the canvas with the text tool opens a positioned inline `textarea`.
  - Blurring the editor or pressing `Ctrl+Enter` / `Cmd+Enter` commits the text into the canvas model; `Escape` cancels it.
  - Text items now participate in draw ordering, hit testing, marquee selection, movement, erasing, snapshot export/import, and SVG export.
- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:49)
    - added the `CanvasText` item type
    - added the inline text editor flow
    - added text drawing, bounds, selection, movement, erasing, save/load, and SVG export support
  - [src/app/types.ts](/home/rok/sync/ideas/rpdf2/src/app/types.ts:119)
    - added `CanvasTextDocument`
    - extended `CanvasDocument` and selection-target types
  - [src-tauri/src/domain/canvas.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/domain/canvas.rs:18)
    - added `texts` to the Rust-side document model
    - added `CanvasText`
  - [src/styles.css](/home/rok/sync/ideas/rpdf2/src/styles.css:300)
    - added styling for the inline canvas text editor
  - [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md:1)
    - marked `add-text-tool` done
  - [todos/add-text-tool](/home/rok/sync/ideas/rpdf2/todos/add-text-tool:1)
    - marked done

# Results

- Outputs:
  - Canvas Mode now has a `Text` tool.
  - Users can click to open an inline editor, type text, and commit it into the canvas.
  - Text items survive save/load and mode switches.
  - Text items can be selected, marquee-selected, moved, erased, and exported to SVG.
- Metrics or observations:
  - This stayed inside the existing canvas rendering model rather than adding a second persistent DOM-layer representation.
  - The first pass intentionally does not add text resizing, rotation, or in-place editing of existing text items.
- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - Text was implemented as a real document item instead of a transient overlay.
  - Assessment:
  - This was necessary to satisfy persistence, selection, and SVG export honestly.
- Fact:
  - The editor is a temporary positioned `textarea`, but the stored/rendered result is still canvas-native text.
  - Assessment:
  - This keeps typing practical without adding a full DOM text layout system.

# Limitations

- Existing text items are not yet re-editable after placement; the first pass supports creation, selection, and movement only.
- Text items do not currently participate in resize handles or selection-style editing.
- I did not manually test the live desktop UX in this turn, so the remaining manual confirmation is the actual feel of placement, blur/commit, and SVG output in the running app.

# Next steps

1. Manually test the new text tool in the desktop app, especially multiline entry, save/load, and SVG export.
2. If the user wants richer text behavior later, add a follow-up task for editing existing text items and text-style controls instead of widening this first pass retroactively.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Rust/Tauri backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch the app, switch to Canvas Mode, choose the `Text` tool, click the canvas, type text, and commit with `Ctrl+Enter` or blur.
4. Save and reload the project, then confirm the text persists and can still be selected and moved.
5. Export SVG and confirm text items are included in the output.
