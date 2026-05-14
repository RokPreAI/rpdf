# Title

Stroke selection editing report

# Context

The selected worker slice was `add-stroke-selection-editing` from [todos/add-stroke-selection-editing](/home/rok/sync/ideas/rpdf2/todos/add-stroke-selection-editing).

This was the right next action because:

- it was the current priority-2 selection task
- marquee and multi-select were already in place, so stroke interaction quality was the next concrete gap
- strokes were technically selectable, but still behaved like second-class items compared with box-like selections

# Goals

- Primary success criteria:
  - selected strokes should have clear visible feedback
  - stroke selections should stay stable during normal selection interactions
  - selected strokes should participate more naturally in later multi-select and style-edit flows

- Secondary success criteria:
  - avoid widening into full stroke-resize semantics
  - improve editability without disturbing existing shape/image/pdf resize behavior

# Approach

- Chosen approach:
  - add stroke-specific selection highlighting along the actual stroke path, not just the generic bounds box
  - allow existing selected strokes to be moved by dragging from the active selection bounds, instead of forcing a precise hit on the thin line
  - keep resize semantics unchanged and explicit: strokes remain non-resizeable in this pass

- Rejected options:
  - inventing geometric stroke-resize behavior here would have widened the task and made it harder to verify
  - limiting the fix to purely visual changes would have left the main interaction problem unresolved

# Implementation

- Task hash:
  - `add-stroke-selection-editing`

- Architecture / flow:
  - Added a cyan path highlight for selected strokes, including single-point strokes, so selection feedback follows the stroke geometry itself.
  - Added bounds-body drag behavior for the current selection. If a selected stroke or mixed selection is already active, dragging from inside the selection bounds now moves the selection even when the pointer is not exactly on the stroke line.
  - Kept stroke selection compatible with the existing multi-select and marquee flows by reusing the shared selection-bounds model rather than creating a separate stroke-only interaction path.

- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts)
    - added stroke-specific selection overlay rendering
    - added selection-bounds hit support for moving already-selected strokes
  - [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md)
    - marked `add-stroke-selection-editing` done and promoted `add-selection-style-editing` to current
  - [todos/add-stroke-selection-editing](/home/rok/sync/ideas/rpdf2/todos/add-stroke-selection-editing)
    - marked done
  - [todos/add-selection-style-editing](/home/rok/sync/ideas/rpdf2/todos/add-selection-style-editing)
    - marked current

# Results

- Outputs:
  - Selected strokes now have explicit path-based visual feedback in addition to the generic selection box.
  - A selected stroke can be moved more predictably because the active selection bounds can be grabbed directly.
  - Stroke selections remain compatible with current multi-select and marquee interactions.
  - Stroke resize is still intentionally unsupported in this pass, and that limit remains explicit in the code path.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - The stroke editability improvement uses selection-bounds dragging rather than stroke-specific handles.
  - Assessment:
  - This is the smallest useful improvement that makes strokes feel less brittle without introducing new geometry semantics.

- Fact:
  - Stroke selection highlight is path-based and uses a separate accent color.
  - Assessment:
  - This makes the selected state readable even when the stroke’s bounds box is larger or visually less informative than the path itself.

# Limitations

- Manual runtime validation across dense overlapping strokes was not performed in this turn.
- Strokes are still not resizeable.
- Batch color and stroke-width editing is still a separate follow-up task.

# Next steps

1. Implement `add-selection-style-editing` so selected strokes and shapes can share batch color and width changes.
2. Implement `add-multi-selection-resize` for grouped resize on box-like items after selection-style editing is in place.
3. Revisit keyboard shortcuts once the selection editing surface is more complete.

# Reproducibility

1. Open Canvas Mode.
2. Draw one or more freehand strokes.
3. Switch to `Select` and select a stroke.
4. Confirm the stroke path gets a dedicated highlight.
5. Drag from inside the active selection bounds and confirm the selected stroke moves predictably.
6. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
