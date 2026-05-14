# Title

Multi-selection resize report

# Context

The selected worker slice was `add-multi-selection-resize` from [todos/add-multi-selection-resize](/home/rok/sync/ideas/rpdf2/todos/add-multi-selection-resize).

This was the right next action because:

- it was the current priority-2 canvas task in [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md)
- multi-select, marquee selection, and single-item resize were already in place
- grouped resize was the next obvious gap in the selection editing flow

# Goals

- Primary success criteria:
  - allow multiple selected resizeable items to show one shared resize frame
  - allow shared-handle dragging to resize all supported selected items together
  - keep resize math stable and bounded

- Secondary success criteria:
  - preserve the existing single-selection resize behavior
  - avoid inventing stroke-resize semantics in this pass

# Approach

- Chosen approach:
  - generalize the resize session from a single target to a captured selection snapshot
  - compute resizing from the shared original selection bounds to the next shared bounds
  - remap each selected item from the original shared bounds into the next shared bounds using geometry captured at drag start

- Rejected options:
  - resizing only the outer frame while leaving internal items fixed would have been visually misleading
  - attempting to include strokes in grouped resize would have widened the task into unresolved stroke-scaling semantics

# Implementation

- Task hash:
  - `add-multi-selection-resize`

- Architecture / flow:
  - Added selection-level resize targeting so handles are derived from the current resizeable selection bounds rather than only the primary selected item.
  - Reworked the resize session to snapshot every selected resizeable target at drag start, including original shape points or original box geometry.
  - Updated resize application so shapes, images, and imported PDF pages are all remapped from the shared original selection bounds to the dragged next bounds.
  - Kept grouped resize explicitly disabled when the selection includes unsupported item types such as strokes, so the interaction fails closed instead of half-working.

- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts)
    - generalized resize sessions from single-item to grouped selection snapshots
    - added shared resize-bounds helpers for the current selection
    - moved handle hit-testing and handle drawing to the grouped selection frame
  - [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md)
    - marked `add-multi-selection-resize` done
    - promoted `add-input-polling-rate-setting` to current
  - [todos/add-multi-selection-resize](/home/rok/sync/ideas/rpdf2/todos/add-multi-selection-resize)
    - marked done
  - [todos/add-input-polling-rate-setting](/home/rok/sync/ideas/rpdf2/todos/add-input-polling-rate-setting)
    - marked current

# Results

- Outputs:
  - Multi-selected shapes, pasted images, and imported PDF pages now show one shared resize box with usable corner handles.
  - Dragging a shared handle resizes all supported selected items together from the captured drag-start geometry.
  - Single-item resize still goes through the same path and remains available.
  - Selections containing strokes no longer show misleading grouped resize handles, because grouped resize is intentionally limited to fully supported selections.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - Grouped resize only activates when every selected item is resizeable.
  - Assessment:
  - This is the safest bounded behavior because mixed stroke selections still lack settled resize semantics.

- Fact:
  - Grouped resize uses original drag-start geometry for all items.
  - Assessment:
  - This avoids compounded resize drift and keeps the behavior aligned with the earlier single-item resize fix.

# Limitations

- Manual runtime validation for every mixed supported combination was not performed in this turn.
- Strokes remain non-resizeable, including inside multi-selection.
- This pass only supports corner-handle grouped resize; there are still no edge handles or rotation controls.

# Next steps

1. Implement `add-input-polling-rate-setting` so stroke capture quality can be tuned when manual testing still shows jagged lines.
2. Harden `harden-svg-export-and-save-path` after more multi-selection export testing.
3. Revisit keyboard shortcuts once the remaining selection/drawing controls are stable.

# Reproducibility

1. Open Canvas Mode.
2. Add two or more resizeable items such as shapes, pasted images, or imported PDF pages.
3. Switch to `Select` and multi-select those items.
4. Confirm a shared resize frame and shared corner handles appear.
5. Drag a corner handle and confirm all selected supported items resize together.
6. Add a stroke into the selection and confirm grouped resize handles do not appear for the mixed unsupported selection.
7. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
