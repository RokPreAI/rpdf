# Title

Canvas element resize report

# Context

The selected worker slice was `add-element-resize` from `TODO.md`.

This was the right next action because:

- it was the next bounded canvas task called out in `REPORT19.md`
- selection and move support already existed, so resize had a clear integration point
- the task could be kept local to Canvas Mode without widening into shortcuts or broader PDF/UI work

Constraints for this slice:

- keep the change inside the existing select-tool workflow
- prioritize predictable first-pass resizing over a feature-rich editor model
- preserve selection state, save/load behavior, and SVG export logic

# Goals

- Primary success criteria:
  - add visible resize affordances for selected resizable items
  - allow direct pointer resizing of supported selected items
  - keep resized state persisted correctly through the existing document model

- Secondary success criteria:
  - avoid redesigning the shape model or selection system
  - leave clear limits for any unsupported first-pass item types

# Approach

- Chosen approach:
  - add corner resize handles to the existing selection overlay
  - treat resizing as part of the select-tool pointer flow
  - support the first pass for shapes, pasted images, and imported PDF pages
  - leave stroke resizing out of this pass, matching the task note that strokes may need a narrower first version

- Rejected options:
  - adding resize as a separate tool would have fractured the interaction model
  - trying to resize strokes in the same pass would have widened the scope substantially
  - adding per-shape bespoke handle logic would have made the first version more complex than needed

# Implementation

- Architecture / flow:
  - Selected resizable items now render four corner handles in the selection overlay.
  - When the select tool presses a handle, Canvas Mode enters a resize session instead of a move session.
  - The dragged handle updates a normalized bounds box while the opposite corner stays fixed.
  - Shapes remap their `start` and `end` points from the original bounds into the new bounds.
  - Images and imported PDF pages update their `x`, `y`, `width`, and `height` directly.

- Key files or components:
  - `src/features/canvas/workspace.ts`
    - added resize handle and resize session types
    - added bounds helpers for selected targets
    - added handle hit testing and resize cursor behavior
    - added corner-handle drawing in the selection overlay
    - added first-pass resize application for shapes, images, and PDF page imports
  - `TODO.md`
    - marked `add-element-resize` done

# Results

- Outputs:
  - Canvas Mode now shows resize handles for selected shapes, pasted images, and imported PDF pages.
  - Those item types can be resized directly with the select tool.
  - Resized state flows through the existing persistence model because the underlying document fields are updated in place.

- Metrics or observations:
  - The slice stayed inside the existing select/move architecture.
  - No Rust/domain schema changes were required because resize only changes existing persisted fields.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success
  - Manual live-window resize interaction testing was not performed in this turn.

# Decisions

- Fact:
  - Resize handles were implemented as corner handles on the selection overlay.
  - Assessment:
  - This gives a direct visible affordance without introducing a second editing mode.

- Fact:
  - Strokes are intentionally excluded from the first-pass resize support.
  - Assessment:
  - This matches the backlog note that strokes may need a narrower first version and keeps the slice bounded to predictable box-based resizing.

# Limitations

- Strokes are not resizable in this first pass.
- This slice does not add edge-only handles, rotation, or aspect-ratio locking.
- Manual runtime validation is still needed to tune handle size and confirm the feel of line/arrow resizing in practice.

# Next steps

1. Implement `add-excalidraw-style-tool-and-color-shortcuts` so the growing canvas toolset stays fast to use.
2. Implement `add-input-polling-rate-setting` as the next canvas quality-of-input task.
3. Revisit `fix-pdf-mode-annotations` only with live manual verification, since the code-side fix already exists but the backlog still needs human confirmation.

# Reproducibility

1. Inspect:
   - `src/features/canvas/workspace.ts`
   - `TODO.md`
2. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch:
   - `npm run tauri dev`
4. In Canvas Mode, draw a shape or arrow, paste an image or import a PDF page, select it, drag a corner handle, then save/load the canvas to confirm the resized state persists.
