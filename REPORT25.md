# Title

Multi-select foundation report

# Context

The selected worker slice was `add-multi-select` from [todos/add-multi-select](/home/rok/sync/ideas/rpdf2/todos/add-multi-select).

This was the right next action because:

- it was the only task under `# Current TODOs`
- it is the dependency for marquee selection and batch style editing
- the canvas file already had a partial local migration toward multi-selection, so finishing that path was the cleanest way to restore consistency

# Goals

- Primary success criteria:
  - allow more than one canvas object to be selected at the same time
  - move a selected group together with the select tool
  - preserve selection state through export/import without breaking item identity

- Secondary success criteria:
  - keep resize behavior limited to single-selection only
  - stay compatible with older single-target selection snapshots

# Approach

- Chosen approach:
  - convert the remaining selection flow from `selectedItem` to `selectedItems`
  - use additive selection on `Shift` / `Ctrl` / `Cmd` click
  - keep one-primary-item semantics only for resize handles, while move/export/snapshot use the full selected set

- Rejected options:
  - introducing marquee selection in the same slice would have widened the task beyond the backlog contract
  - changing selection to item references instead of id-based snapshot resolution would have made save/load less stable

# Implementation

- Task hash:
  - `add-multi-select`

- Architecture / flow:
  - The selection document type now supports multiple targets while remaining backward-compatible with older single-target snapshots.
  - Canvas selection state now normalizes, exports, imports, and redraws from a `selectedItems` array instead of a single target.
  - `Shift` / `Ctrl` / `Cmd` click toggles items into or out of the current selection.
  - Dragging any selected item in Select mode now moves the full selected set together.
  - Resize handles only appear when exactly one resizable item is selected.
  - Selection overlay now draws each selected item plus an additional grouped bounds frame for multi-selection.

- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts)
    - completed the single-target to multi-target migration
    - added additive click selection and grouped move behavior
    - updated selection overlay and SVG export selection handling
  - [src/app/types.ts](/home/rok/sync/ideas/rpdf2/src/app/types.ts)
    - added multi-target selection snapshot support with backward compatibility
  - [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md)
    - marked `add-multi-select` done and promoted `add-marquee-selection` to current
  - [todos/add-multi-select](/home/rok/sync/ideas/rpdf2/todos/add-multi-select)
    - marked done
  - [todos/add-marquee-selection](/home/rok/sync/ideas/rpdf2/todos/add-marquee-selection)
    - marked current

# Results

- Outputs:
  - Canvas Mode can now keep multiple selected objects at once.
  - Multi-selected items move together predictably.
  - Selection survives normal canvas export/import flows by persisting selected ids.
  - SVG export now works on multiple selected vector items, while still refusing raster/PDF selections.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - Additive selection uses `Shift`, `Ctrl`, or `Cmd` click.
  - Assessment:
  - This keeps the first pass practical on both Linux/Windows-style and macOS-style modifier habits without needing a dedicated new UI control.

- Fact:
  - Multi-selection resize was intentionally not added here.
  - Assessment:
  - The task contract was about selection state and movement. Resize remains stable by staying single-selection-only for now.

# Limitations

- Manual runtime testing of multi-select on every item type was not performed in this turn.
- Marquee drag selection is still a separate follow-up task.
- Batch color/stroke-width editing is still a separate follow-up task.

# Next steps

1. Implement `add-marquee-selection` so empty-space drag can create multi-selections without modifier-clicking.
2. Implement `add-stroke-selection-editing` so freehand strokes participate more naturally in later batch style edits.
3. Implement `add-selection-style-editing` after marquee and stroke participation are in place.

# Reproducibility

1. Open Canvas Mode.
2. Switch to `Select`.
3. Use `Shift` / `Ctrl` / `Cmd` click to add or remove items from the current selection.
4. Drag one of the selected items and confirm the selected set moves together.
5. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
