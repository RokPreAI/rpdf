# Title

Canvas full-containment marquee report

# Context

The selected worker slice was `require-full-containment-selection` from [todos/require-full-containment-selection](/home/rok/sync/ideas/rpdf2/todos/require-full-containment-selection).

This was the right next action because:

- it was the current priority-1 task in [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md)
- the user explicitly asked for marquee selection to require full coverage instead of partial overlap
- the previous selection slice already improved click-hit geometry, so marquee policy was the next cleanly isolated selection follow-up

# Goals

- Primary success criteria:
  - change marquee selection from overlap-based inclusion to full containment
  - keep additive marquee behavior compatible with the new policy
  - make the rule predictable for strokes, shapes, images, and imported PDF pages

- Secondary success criteria:
  - keep the implementation narrow and local to marquee membership
  - avoid mixing this slice with click-hit tuning or stroke resize behavior

# Approach

- Chosen approach:
  - keep the existing marquee drag flow unchanged
  - replace the old bounds-intersection membership check with an explicit bounds-containment check
  - allow a small tolerance margin so near-edge containment is not overly brittle at different zoom levels

- Rejected options:
  - rewriting marquee around point-by-point geometry checks would have widened the task and made the first pass harder to reason about
  - changing click-hit selection here would have mixed two separate user-reported issues again

# Implementation

- Task hash:
  - `require-full-containment-selection`

- Architecture / flow:
  - Added `boundsContainBounds()` as the marquee membership rule.
  - Updated `collectTargetsInBounds()` to require the marquee box to fully contain each target's bounds instead of merely intersecting them.
  - Added a small zoom-aware containment tolerance to avoid edge-case misses when the selection box lands essentially on an item's outer bounds.

- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts)
    - replaced marquee overlap checks with full-containment checks
    - added a small containment tolerance for stable selection at different zoom levels
  - [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md)
    - marked `require-full-containment-selection` done
    - promoted `add-stroke-resize` to current
  - [todos/require-full-containment-selection](/home/rok/sync/ideas/rpdf2/todos/require-full-containment-selection)
    - marked done
  - [todos/add-stroke-resize](/home/rok/sync/ideas/rpdf2/todos/add-stroke-resize)
    - marked current

# Results

- Outputs:
  - Drag-box selection now requires full containment instead of partial overlap.
  - Partially covered strokes, shapes, images, and imported PDF pages are no longer selected by marquee.
  - Additive marquee selection still uses the same selection-set merge flow, but now with the stricter containment rule.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - The first pass uses target bounds with a small containment tolerance rather than exact point-cloud or path containment.
  - Assessment:
  - This keeps the marquee rule predictable and cheap while still matching the user's request much more closely than overlap-based selection.

- Fact:
  - The task only changed marquee membership logic.
  - Assessment:
  - This preserves a clean next step for stroke resizing without entangling selection policy and resize math in one slice.

# Limitations

- Live manual interaction testing was not performed in this turn, so the precise feel of the containment tolerance still needs hands-on validation.
- Stroke containment still relies on the stroke bounds approximation rather than exact path containment.
- Click selection remains unchanged in this slice.

# Next steps

1. Implement `add-stroke-resize`.
2. If manual testing shows marquee containment still feels too generous for some stroke shapes, refine stroke containment bounds separately without changing the overall policy back.
3. After stroke resize, continue with either `add-editable-config-file` or `remove-mode-switch-restore-copy`.

# Reproducibility

1. Open Canvas Mode.
2. Draw several strokes and shapes with visible spacing.
3. Switch to `Select`.
4. Drag a marquee that only partially overlaps an item and confirm it is not selected.
5. Drag a marquee that fully encloses the item and confirm it is selected.
6. Repeat with additive marquee selection using `Shift`, `Ctrl`, or `Cmd`.
7. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
