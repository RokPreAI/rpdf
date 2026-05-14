# Title

Canvas geometry-aware selection report

# Context

The selected worker slice was `improve-selection-hit-geometry` from [todos/improve-selection-hit-geometry](/home/rok/sync/ideas/rpdf2/todos/improve-selection-hit-geometry).

This was the right next action because:

- it was the current priority-1 task in [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md)
- the user explicitly reported that selection still felt like bounding-box picking
- this was a narrow canvas interaction fix that directly improves daily usability without widening into marquee policy or stroke-resize work

# Goals

- Primary success criteria:
  - make vector click selection follow visible geometry more closely
  - stop hollow shapes from selecting like large filled rectangles
  - reduce accidental picks on thin vector items

- Secondary success criteria:
  - keep image and imported PDF page selection behavior stable
  - avoid widening the slice into marquee-containment changes

# Approach

- Chosen approach:
  - keep the existing selection pipeline intact
  - tighten the vector hit tolerance to a smaller geometry-aware value
  - replace rectangle and ellipse hit testing with outline-based checks instead of broad interior-box checks

- Rejected options:
  - rewriting the whole selection system would have widened the task too much
  - changing marquee behavior here would have mixed click-hit geometry with selection-box policy, which already has its own follow-up task

# Implementation

- Task hash:
  - `improve-selection-hit-geometry`

- Architecture / flow:
  - Added a shared `selectionToleranceForWidth()` helper for vector click-picking.
  - Added `pointNearRectangleOutline()` so rectangles are hit-tested against their visible border band instead of their entire interior box.
  - Added `pointNearEllipseOutline()` so ellipses are hit-tested against the visible ellipse ring rather than a filled oval area.
  - Updated the shape hit-test path to use those outline-specific helpers.
  - Kept image and imported PDF page selection behavior unchanged.

- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts)
    - added geometry-aware outline hit helpers for rectangle and ellipse selection
    - tightened vector selection tolerance for click selection
  - [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md)
    - marked `improve-selection-hit-geometry` done
    - promoted `require-full-containment-selection` to current
  - [todos/improve-selection-hit-geometry](/home/rok/sync/ideas/rpdf2/todos/improve-selection-hit-geometry)
    - marked done
  - [todos/require-full-containment-selection](/home/rok/sync/ideas/rpdf2/todos/require-full-containment-selection)
    - marked current

# Results

- Outputs:
  - Rectangle and ellipse selection now tracks the visible outline much more closely instead of treating the whole hollow interior as easy hit area.
  - Thin vector items use a smaller click tolerance, which reduces accidental picks when the pointer is merely nearby.
  - Line, arrow, stroke, image, and imported PDF page selection paths remain compatible with the existing selection flow.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - The selection fix was limited to click-hit geometry, not selection-box containment.
  - Assessment:
  - This keeps the change understandable and preserves a clean next slice for the user-requested full-containment marquee rule.

- Fact:
  - Images and imported PDF pages still use practical box selection.
  - Assessment:
  - That matches their raster/page nature and avoids overcomplicating non-vector picking in this slice.

# Limitations

- Live manual interaction testing was not performed in this turn, so the feel of the new tolerance still needs user confirmation in the running app.
- Marquee selection still uses the existing intersection-based policy; the user-requested full-containment behavior is intentionally left for the next task.
- Stroke resizing remains out of scope for this slice.

# Next steps

1. Implement `require-full-containment-selection`.
2. If manual testing still finds line or stroke picking too loose or too strict, tune the selection tolerance further with item-type-specific values.
3. After containment policy is stable, implement `add-stroke-resize`.

# Reproducibility

1. Open Canvas Mode.
2. Draw a thin line, an arrow, a rectangle, and an ellipse.
3. Switch to `Select`.
4. Click near each vector item and confirm nearby empty space no longer selects as easily.
5. Click inside the hollow interior of a rectangle or ellipse and confirm selection is tied more to the outline than the whole box.
6. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
