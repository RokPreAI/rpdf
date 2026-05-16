# Title

Canvas delete-selection shortcut report

# Context

- Problem:
  - The canvas already supported selecting single and multiple items, but there was no keyboard deletion path for the active selection.
  - Removing content required the eraser or other more manual workflows, even for obviously selected items.
- Constraints:
  - This needed to stay a bounded canvas keyboard-input slice.
  - The repo still has unrelated local changes in `src/app/shell.ts`, `src/styles.css`, `TODO.md`, `.gitignore`, and generated `dist/` files, so the commit needed to stay scoped to the selection-delete path only.

# Goals

- Primary success criteria:
  - pressing `Delete` removes the current selected item or selected set
  - deletion works across strokes, shapes, text, images, and imported PDF page placements
  - selection clears coherently after deletion
- Secondary success criteria:
  - keep the change local to the existing canvas keyboard path
  - avoid breaking text-entry flows by respecting editable targets

# Approach

- Chosen approach:
  - add a small `removeSelectedItems()` helper that removes selected items by stable ids instead of array indexes
  - call that helper from the existing `onKeyDown()` path when `Delete` is pressed
- Rejected option:
  - deleting directly by selection indexes would be brittle because the selected set can mix multiple item arrays and array indexes shift during removal

# Implementation

- Task hash:
  - `add-delete-selection-shortcut`
- Matching task file:
  - [todos/add-delete-selection-shortcut](/home/rok/sync/ideas/rpdf2/todos/add-delete-selection-shortcut:1)
- Architecture / flow:
  - `removeSelectedItems()` now collects the selected ids by item kind, filters each backing array, writes the filtered arrays back in place, and clears the selection.
  - `onKeyDown()` now listens for `Delete` and uses that helper to remove the current selection before redrawing.
  - The existing editable-target guard remains intact, so pressing `Delete` while focused inside a text editor or other input does not delete unrelated canvas content.
- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:2338)
    - added `removeSelectedItems()` using stable selected ids
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:3534)
    - added `Delete` handling in the canvas keyboard path

# Results

- Outputs:
  - Pressing `Delete` now removes the selected strokes, shapes, text items, images, and imported PDF page placements.
  - Multi-selection deletion works through the same path because removal is based on the whole selected id set.
  - The selection clears cleanly after deletion.
- Verification:
  - Ran `npm run build`
    - result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - result: success

# Decisions

- Fact:
  - Selection already had stable id-based serialization and restore helpers.
- Assessment:
  - Reusing ids for deletion keeps the removal path robust across mixed selection sets and array mutation.
- Fact:
  - Text editing already uses the standard editable-target guard in the keyboard handler.
- Assessment:
  - Keeping `Delete` behind that guard avoids accidentally deleting selected canvas items while the user is editing text content.

# Limitations

- I did not manually test the live Tauri app in this turn, so the remaining hands-on confirmation is whether deletion feel is correct across single selection, multi-selection, and text-edit focus boundaries.
- This slice does not add undo history for selection deletion; it only adds the deletion trigger.
- `TODO.md` was not updated in this commit because it already contains unrelated local backlog edits.

# Next steps

1. Manually test `Delete` on single and multi-selection in Canvas Mode.
   - Confirm it works for vectors, text, pasted images, and imported PDF page placements.
2. Take the next canvas keyboard task, [todos/add-select-all-shortcut](/home/rok/sync/ideas/rpdf2/todos/add-select-all-shortcut:1), if you want the rest of the standard selection keyboard flow next.
3. If users expect `Backspace` to behave the same way on this platform, add that as a small follow-up task after confirming desired behavior.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Tauri/Rust backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch the app, switch to Canvas Mode, select one or more items, then press `Delete` and verify the selected set is removed while input-focused text editing remains unaffected.
