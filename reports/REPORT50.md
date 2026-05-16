# Title

Canvas select-all shortcut report

# Context

- Problem:
  - Canvas Mode already supported single selection, additive selection, and marquee selection, but it lacked the standard `Ctrl+A` / `Cmd+A` select-all gesture.
  - That left the selection keyboard workflow incomplete, especially right after adding the `Delete` shortcut for selected items.
- Constraints:
  - This needed to stay a narrow keyboard-selection slice.
  - The repo still has unrelated local changes in `src/app/shell.ts`, `src/styles.css`, `TODO.md`, `.gitignore`, and generated `dist/` files, so the commit needed to stay scoped to the select-all behavior only.

# Goals

- Primary success criteria:
  - pressing `Ctrl+A` / `Cmd+A` in Canvas Mode selects every selectable canvas item
  - browser/webview default page-text selection does not interfere
  - the resulting selection works with existing move, resize, style-edit, and delete flows
- Secondary success criteria:
  - keep the change local to the existing keyboard handler and selection model

# Approach

- Chosen approach:
  - add a small `selectAllItems()` helper that builds a complete `SelectionTarget[]` across strokes, shapes, text items, images, and imported PDF page placements
  - call that helper from the existing `onKeyDown()` path on `Ctrl+A` / `Cmd+A`
  - prevent the default browser/webview action before updating the selection
- Rejected option:
  - trying to route select-all through marquee or other gesture simulation would have been heavier and less reliable than constructing the selection directly

# Implementation

- Task hash:
  - `add-select-all-shortcut`
- Matching task file:
  - [todos/add-select-all-shortcut](/home/rok/sync/ideas/rpdf2/todos/add-select-all-shortcut:1)
- Architecture / flow:
  - `selectAllItems()` now constructs a full `SelectionTarget[]` for every selectable backing array in the canvas workspace.
  - `onKeyDown()` now intercepts `Ctrl+A` / `Cmd+A`, prevents the browser default action, applies that full selection, and redraws.
  - The existing editable-target guard remains in place, so pressing `Ctrl+A` inside a text editor or other input still follows normal input behavior instead of hijacking the app selection.
- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:2387)
    - added `selectAllItems()`
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:3578)
    - added `Ctrl+A` / `Cmd+A` handling in the canvas keyboard path

# Results

- Outputs:
  - `Ctrl+A` / `Cmd+A` now selects all strokes, shapes, text items, pasted images, and imported PDF page placements in Canvas Mode.
  - Browser/webview default select-all behavior is prevented when the canvas owns the keyboard event.
  - The resulting selection uses the same selection model as manual and marquee selection, so downstream move, resize, style-edit, export, and delete flows continue to work on that set.
- Verification:
  - Ran `npm run build`
    - result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - result: success

# Decisions

- Fact:
  - The selection model already supports mixed-kind multi-selection through `SelectionTarget[]`.
- Assessment:
  - Constructing the full target set directly is the simplest and most robust way to implement select-all.
- Fact:
  - The keyboard handler already guards editable targets before app-level shortcuts run.
- Assessment:
  - Keeping select-all behind that same guard avoids breaking standard text selection inside inline editors.

# Limitations

- I did not manually test the live Tauri app in this turn, so the remaining hands-on confirmation is whether select-all feel is correct with mixed content and whether the resulting full selection behaves well on very dense canvases.
- This slice does not update `TODO.md` because that file still contains unrelated local backlog edits.

# Next steps

1. Manually test `Ctrl+A` / `Cmd+A` in Canvas Mode.
   - Confirm it selects mixed content and works with delete, move, style edit, and resize expectations.
2. If you want to continue the keyboard workflow cleanup, the next bounded candidate is [todos/optimize-project-save-file-size](/home/rok/sync/ideas/rpdf2/todos/optimize-project-save-file-size:1) or [todos/add-fit-view-space-shortcut](/home/rok/sync/ideas/rpdf2/todos/add-fit-view-space-shortcut:1), depending on whether you want storage or interaction work next.
3. If full-canvas selection becomes too heavy on large canvases, add a follow-up task to measure selection/render cost under large mixed-content loads.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Tauri/Rust backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch the app, switch to Canvas Mode, create mixed content, then press `Ctrl+A` / `Cmd+A` and verify all selectable items are selected without browser text-selection interference.
