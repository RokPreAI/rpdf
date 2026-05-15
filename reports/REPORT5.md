# Title

Canvas selection and move tool report

# Context

After the PDF shell and persistence passes, `TODO.md` advanced to `add-selection-and-move-tools`. Canvas Mode could already draw, erase, pan, zoom, paste images, and save/load, but it still lacked a basic editing operation: selecting an existing item and repositioning it.

The constraints were:

- keep the change local to Canvas Mode
- implement one reliable first-pass selection model instead of a broad editor feature set
- support both strokes and pasted images
- avoid widening into shape editing, export logic, or autosave in the same cycle

# Goals

- Primary success criteria:
  - add an explicit selection tool
  - allow users to select a stroke or image intentionally
  - allow the selected item to move without corrupting document state

- Secondary success criteria:
  - make the selected item visibly obvious
  - keep the new behavior compatible with the existing save/load document model

# Approach

- Chosen approach:
  - Add a single `select` tool to the existing canvas tool rail.
  - Implement bounded hit testing:
    - images by rectangle
    - strokes by padded bounds plus point-distance confirmation
  - Move the selected item by dragging in world coordinates, reusing the current camera model.
  - Draw a dashed overlay around the selected item so the state is explicit.

- Rejected options:
  - Adding multi-select, resize handles, or rotation would widen the task beyond the current TODO.
  - Creating separate “select” and “move” tools would add UI complexity without improving the first reliable editing slice.
  - Restricting selection to only strokes would leave pasted images as second-class objects even though they are already part of the canvas document model.

# Implementation

- Architecture / flow:
  - `src/features/canvas/workspace.ts` now tracks:
    - `selectedItem`
    - `moveAnchorPoint`
  - The selection flow is:
    - pointer down in `select` mode
    - hit-test topmost image first, then latest stroke
    - store the selected target
    - drag to move it in world coordinates
    - redraw with a selection overlay
  - Selected images move by updating `x`/`y`.
  - Selected strokes move by translating every stored point.

- Key files or components:
  - `src/features/canvas/workspace.ts`
  - `TODO.md`

- Example:
  - A user can switch to `S`, click a pasted image to select it, then drag it to a new position.
  - The same tool can click a stroke, show a dashed bounding box, and move that stroke without changing its color, width, or pressure samples.

# Results

- Outputs:
  - Canvas Mode now has a `select` tool.
  - Both strokes and images can now be selected.
  - Selected items can be moved by dragging.
  - A visible dashed overlay marks the selected target.
  - `TODO.md` now marks `add-selection-and-move-tools` as done and advances the current task to `add-autosave-and-recovery`.

- Metrics or observations:
  - The implementation reuses the existing world-coordinate camera math, so moved items stay consistent with pan/zoom behavior.
  - Selection state is intentionally transient and is cleared on relevant destructive actions such as clear/erase/import.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - The first pass uses one `select` tool that also handles movement.
  - Assessment:
  - This is the simplest believable editing model and matches the task requirement without bloating the toolbar.

- Fact:
  - Hit testing prioritizes images before strokes and scans from newest to oldest.
  - Assessment:
  - That matches likely user expectations about topmost objects and recent edits without adding a full z-order system.

- Fact:
  - Selection uses a dashed bounding overlay instead of resize handles.
  - Assessment:
  - This makes the selected state obvious while keeping the interaction narrow and reliable.

# Limitations

- Only one item can be selected at a time.
- There are no resize, rotate, duplicate, or delete-from-selection actions yet.
- Keyboard nudging is not implemented.
- Selection state is not yet reused by export, although the canvas now has the needed target concept.

# Next steps

1. Complete `add-autosave-and-recovery` so the richer editable canvas state is safer against accidental loss.
2. Complete `add-svg-export-eligibility` so selection-aware canvas content can feed a trustworthy export path.
3. Complete `add-draw-shapes` so canvas editing grows from freehand-only into basic geometric authoring.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Rust backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Inspect the selection and move implementation:
   - `src/features/canvas/workspace.ts`
