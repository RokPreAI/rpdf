# Title

Selection style editing report

# Context

The selected worker slice was `add-selection-style-editing` from [todos/add-selection-style-editing](/home/rok/sync/ideas/rpdf2/todos/add-selection-style-editing).

This was the right next action because:

- it was the current priority-2 canvas task in [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md)
- multi-select, marquee selection, and stroke selection editing were already in place
- the user explicitly wanted selected objects to be restylable after creation instead of only changing the active drawing defaults

# Goals

- Primary success criteria:
  - allow selected vector items to have color changed after creation
  - allow selected vector items to have stroke width changed after creation
  - keep the change local to the existing canvas controls instead of adding a new inspector surface

- Secondary success criteria:
  - preserve default drawing color and width behavior when nothing is selected
  - keep mixed selections stable, even when unsupported selected item types are also present

# Approach

- Chosen approach:
  - reuse the existing color palette and stroke-width slider as selection-edit controls whenever the current selection contains editable vector items
  - keep those same controls as the default drawing controls when no editable selection is active
  - sync the control state from selection contents so the UI reflects selected styles instead of only tool defaults

- Rejected options:
  - adding a separate inspector panel would have widened the surface area and fought the existing compact canvas UI direction
  - limiting the slice to single-selection style edits would have undercut the new multi-select workflow

# Implementation

- Task hash:
  - `add-selection-style-editing`

- Architecture / flow:
  - Added helpers to resolve the currently selected editable vector items, compute shared selection color/width state, and apply batch style changes across the selected vector set.
  - Updated the existing palette buttons so a click now edits the current selection when editable selected vector items exist; otherwise it still updates the default drawing color and keeps the old tool-default behavior.
  - Updated the stroke-width slider so it applies directly to selected strokes and shapes when present, while still acting as the default width control when there is no editable selection.
  - Added style-control syncing so the active palette button and width readout reflect the current selection state, including a `Mixed` width label for multi-selection cases that do not share one width yet.

- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts)
    - added selection-aware style application helpers
    - made the existing palette and stroke-width controls selection-aware
    - synced control state on selection and preference changes
  - [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md)
    - marked `add-selection-style-editing` done
    - promoted `add-multi-selection-resize` to current
  - [todos/add-selection-style-editing](/home/rok/sync/ideas/rpdf2/todos/add-selection-style-editing)
    - marked done
  - [todos/add-multi-selection-resize](/home/rok/sync/ideas/rpdf2/todos/add-multi-selection-resize)
    - marked current

# Results

- Outputs:
  - Selected strokes and shapes can now have their color changed after creation using the existing canvas palette.
  - Selected strokes and shapes can now have their stroke width changed after creation using the existing stroke-width slider.
  - Mixed selections that also include images or imported PDF pages stay stable; vector items update, while unsupported raster-like items are left unchanged.
  - The canvas style controls now reflect selection state instead of only the active tool defaults.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - Selection style editing was implemented by reusing the existing compact controls instead of introducing a new panel.
  - Assessment:
  - This matches the current canvas UI direction and keeps the task bounded.

- Fact:
  - Only vector items are edited in this pass.
  - Assessment:
  - This is the honest first pass because raster images and imported PDF page recolor are separate semantics from vector stroke/fill styling.

# Limitations

- Manual runtime validation of all mixed multi-selection combinations was not performed in this turn.
- When a multi-selection contains different vector widths, the slider readout shows `Mixed` until the user applies a new width.
- Raster image and imported PDF page appearance is intentionally unchanged by these controls.

# Next steps

1. Implement `add-multi-selection-resize` so the new multi-select flow can resize supported items as a group.
2. Implement `add-input-polling-rate-setting` if the stroke capture smoothness issue is still visible during manual testing.
3. Revisit keyboard shortcuts once the selection editing and grouped resize surfaces are both stable.

# Reproducibility

1. Open Canvas Mode.
2. Draw multiple strokes or shapes with different colors and widths.
3. Switch to `Select` and select one or more vector items.
4. Click a palette color and confirm the selected vectors change color.
5. Move the stroke-width slider and confirm the selected vectors change width.
6. Deselect everything and confirm the same controls still behave as drawing defaults.
7. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
