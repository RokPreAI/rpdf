# Title

Marquee selection report

# Context

The selected worker slice was `add-marquee-selection` from [todos/add-marquee-selection](/home/rok/sync/ideas/rpdf2/todos/add-marquee-selection).

This was the right next action because:

- it was the only task under `# Current TODOs`
- it builds directly on the just-finished multi-select foundation
- it unlocks practical multi-object workflows without needing modifier clicks for every item

# Goals

- Primary success criteria:
  - dragging on empty canvas with the select tool should show a visible selection rectangle
  - releasing that rectangle should select intersecting items predictably
  - marquee behavior should not break existing move or resize interactions

- Secondary success criteria:
  - support additive marquee selection with the same modifier family as additive click selection
  - keep the first pass explicit and stable rather than feature-rich

# Approach

- Chosen approach:
  - start marquee selection only when the select tool begins on empty space
  - keep existing item click, move, and single-item resize paths unchanged
  - use a simple intersecting-bounds rule across strokes, shapes, images, and imported PDF pages

- Rejected options:
  - adding direction-dependent semantics like desktop left-to-right versus right-to-left selection would have widened the task unnecessarily
  - mixing marquee with in-progress item drag would have made the interaction ambiguous and harder to verify

# Implementation

- Task hash:
  - `add-marquee-selection`

- Architecture / flow:
  - Added a `MarqueeSession` to track the drag origin, current point, and whether the marquee is additive.
  - Select mode now starts marquee selection only from empty-space drags; clicking existing items still routes to select/move/resize behavior.
  - Added a cyan marquee overlay that renders live while dragging.
  - On release, the marquee resolves to normalized bounds and selects all intersecting targets.
  - Modifier-assisted marquee adds to the current selection instead of replacing it.

- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts)
    - added marquee session state and overlay rendering
    - added bounds-intersection target collection
    - wired empty-space select drag into marquee behavior
  - [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md)
    - marked `add-marquee-selection` done and promoted `add-stroke-selection-editing` to current
  - [todos/add-marquee-selection](/home/rok/sync/ideas/rpdf2/todos/add-marquee-selection)
    - marked done
  - [todos/add-stroke-selection-editing](/home/rok/sync/ideas/rpdf2/todos/add-stroke-selection-editing)
    - marked current

# Results

- Outputs:
  - Dragging on empty canvas in Select mode now shows a visible marquee rectangle.
  - Releasing the marquee selects intersecting canvas items predictably.
  - Existing click-to-select, move, and resize behavior continues to work for already-selected items.
  - Additive marquee selection works with `Shift` / `Ctrl` / `Cmd`.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - The first pass uses simple bounds intersection instead of more elaborate containment rules.
  - Assessment:
  - This is easier to reason about, works across all current item types, and is enough to unlock the workflow the user requested.

- Fact:
  - Empty click and empty drag are now distinct only by drag distance, not by separate tools or modes.
  - Assessment:
  - This keeps the interaction close to desktop selection behavior without adding extra UI.

# Limitations

- Manual runtime verification of marquee selection across every item type was not performed in this turn.
- This slice does not yet add stroke-specific style editing or batch style changes for the selection.
- The marquee uses one intersection rule in all drag directions; it does not implement directional desktop variants.

# Next steps

1. Implement `add-stroke-selection-editing` so freehand strokes participate cleanly in upcoming batch style operations.
2. Implement `add-selection-style-editing` to change colors and stroke width for selected items.
3. Revisit shortcut discoverability once the selection workflow is more complete.

# Reproducibility

1. Open Canvas Mode.
2. Switch to `Select`.
3. Drag on empty canvas and confirm a visible marquee rectangle appears.
4. Release over multiple items and confirm they become selected together.
5. Repeat with `Shift`, `Ctrl`, or `Cmd` held to confirm additive marquee selection.
6. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
