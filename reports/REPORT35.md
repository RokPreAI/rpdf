# Title

Canvas stroke resize report

# Context

The selected worker slice was `add-stroke-resize` from [todos/add-stroke-resize](/home/rok/sync/ideas/rpdf2/todos/add-stroke-resize).

This was the right next action because:

- it was the current priority-2 task in [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md)
- the user explicitly asked for stroke resizing after the earlier shape and multi-selection resize work
- the existing resize session architecture was already in place for shapes, images, and imported PDF pages, so stroke support was a clean extension instead of a redesign

# Goals

- Primary success criteria:
  - allow selected strokes to participate in resize handles
  - resize stroke geometry from drag-start state instead of compounding transforms
  - keep resized strokes valid for rendering, persistence, and export

- Secondary success criteria:
  - keep pressure-derived stroke appearance sensible under resize
  - avoid breaking the existing non-stroke resize paths

# Approach

- Chosen approach:
  - treat strokes as resizeable targets in the existing grouped resize path
  - snapshot each stroke's points and base width at resize start
  - remap the original stroke points into the next resize bounds the same way other geometry is remapped
  - scale stroke width by the average x/y resize factor so resized strokes do not stay visually frozen

- Rejected options:
  - mutating the already-resized stroke points on every pointer move would risk the same compounding instability seen in earlier resize bugs
  - leaving stroke width completely fixed during geometry resize would make large resizes look visually inconsistent

# Implementation

- Task hash:
  - `add-stroke-resize`

- Architecture / flow:
  - Extended `isResizableTarget()` so strokes participate in resize eligibility.
  - Extended `ResizeSession` to store drag-start stroke snapshots.
  - Updated `createResizeSession()` to capture original stroke points and base width.
  - Added a small scale-factor helper and updated `applyResize()` to remap stroke points from the original bounds into the next bounds while scaling `baseWidth`.

- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts)
    - enabled stroke participation in resize sessions
    - added original-stroke snapshot capture
    - added stroke point remap and width scaling during resize
  - [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md)
    - marked `add-stroke-resize` done
    - promoted `add-editable-config-file` to current
  - [todos/add-stroke-resize](/home/rok/sync/ideas/rpdf2/todos/add-stroke-resize)
    - marked done
  - [todos/add-editable-config-file](/home/rok/sync/ideas/rpdf2/todos/add-editable-config-file)
    - marked current

# Results

- Outputs:
  - Selected strokes now expose resize handles when the selection is resize-eligible.
  - Dragging a resize handle rescales stroke points from drag-start geometry instead of distorting incrementally.
  - Stroke width now scales with the overall resize, while per-point pressure remains intact.
  - Mixed resizeable selections can now include strokes alongside the existing resizeable item types.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - Stroke width scales by the average of the x and y resize factors.
  - Assessment:
  - This is a practical first-pass rule that keeps stroke appearance coherent under non-uniform resize without inventing a more complex pressure model.

- Fact:
  - Stroke points are always remapped from the original resize-session snapshot.
  - Assessment:
  - This preserves stability and avoids compounding geometry drift across pointer moves.

# Limitations

- Live manual interaction testing was not performed in this turn, so the feel of stroke width scaling still needs hands-on validation.
- The current first pass uses whole-selection bounds remapping rather than a more advanced stroke-specific skeleton or path-length-aware transform.
- If future testing shows non-uniform width scaling needs a different rule, that tuning remains open.

# Next steps

1. Implement `add-editable-config-file`.
2. If manual testing shows stroke width scaling feels too strong or too weak, refine the width-scale rule without changing the drag-start geometry model.
3. After config-file support, continue with `remove-mode-switch-restore-copy`.

# Reproducibility

1. Open Canvas Mode.
2. Draw one or more freehand strokes.
3. Switch to `Select` and select a stroke.
4. Drag a resize handle and confirm the stroke geometry and visible width change predictably.
5. Repeat with a mixed resizeable selection that includes a stroke and another supported item.
6. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
