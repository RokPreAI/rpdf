# Title

Canvas pen and viewport loop established for rpdf

# Context

- Problem:
  With the desktop shell in place, the next current task in `TODO.md` was `implement-canvas-pen-and-viewport`. Infinite Canvas Mode still had only a placeholder surface, so the project needed a real interaction loop for drawing, panning, zooming, and storing pen strokes in the document model.
- Constraints:
  The task had to remain focused on the core canvas loop. It could not expand into image/PDF import, patterned backgrounds, PDF-mode behavior, annotation tools, or persistence. The implementation also needed to fit the chosen `egui` shell and keep pressure-aware input realistic within the GUI stack’s available event model.

# Goals

- Primary success criteria:
  Make Infinite Canvas Mode usable for basic drawing, panning, and zooming on a workspace larger than a single page.
- Secondary success criteria:
  Record strokes into the existing model layer, use available pressure values when the backend provides them, and leave the canvas surface ready for later asset and annotation tasks.

# Approach

- Chosen approach:
  Extended the shell with a dedicated canvas interaction state, then implemented a single custom drawing surface in `egui` that handles primary-drag drawing, secondary-drag panning, and scroll-wheel zooming. Finished strokes are committed into `CanvasDocument` as `CanvasItem::PenStroke` values.
- Rejected options:
  Did not wait for a future annotation tool system before adding stroke creation, because pen strokes are already part of the core canvas task. Did not attempt a separate rendering engine or custom input backend because this task needed a small, integrated slice on the existing app shell.

# Implementation

- Architecture / flow:
  `src/app.rs` now contains `CanvasInteractionState` and `PendingStroke` to track in-progress drawing separate from committed document items. The canvas view allocates a painter surface, transforms between canvas-space and screen-space using the existing `ViewportState`, applies zoom and pan to the viewport, collects stroke points from pointer/touch input, and paints both committed strokes and the active in-progress stroke.
- Key files or components:
  - `src/app.rs`: added canvas interaction state, viewport transforms, stroke collection, panning, zooming, and canvas painting.
  - `TODO.md`: advanced task status after completion.
  - `SUBTODO.md`: cleared after the task finished.
- Example:
  `handle_canvas_drawing` starts or extends a `PendingStroke` while the primary pointer button is down, sampling touch force when available and falling back to a default pressure for mouse input. `commit_active_stroke` converts that pending stroke into a `CanvasItem::PenStroke` stored in the document model.

# Results

- Outputs:
  Updated:
  - `src/app.rs`
  - `TODO.md`
  - `SUBTODO.md`
- Metrics or observations:
  Infinite Canvas Mode now supports:
  - a workspace surface larger than one page
  - pressure-aware stroke point capture when force data is available
  - fallback stroke capture for normal pointer dragging
  - secondary-drag panning
  - scroll zoom
  - rendering of committed strokes and the active stroke
  - a live session summary showing zoom, origin, item count, and active-stroke state
- Verification:
  Ran `cargo check` successfully after the canvas interaction changes.

# Decisions

- Tradeoffs made:
  - Used touch-force events when available and a constant fallback pressure otherwise, because the shell stack exposes force opportunistically rather than guaranteeing pressure on every platform.
  - Treated pen strokes as the first real canvas item placement mechanism, which satisfies the current task without prematurely adding general asset manipulation.
  - Kept panning on secondary-drag to avoid overloading the primary drawing path.

# Limitations

- Known issues, uncertainties, or risks:
  - There is no runtime test coverage yet; verification is compile-level only.
  - Pressure sensitivity depends on backend event support and currently falls back to a constant for ordinary mouse input.
  - The canvas does not yet support typed text, imported images, imported PDF pages, or configurable background pattern rendering.
  - Strokes are append-only at this stage; selection and editing of existing items are not implemented.

# Next steps

1. Implement `add-canvas-assets-and-backgrounds` because the canvas loop is now stable enough to host typed text, imported assets, and zoom-aware visual references.
2. Implement `add-pdf-viewer-and-navigation` after that or in a later cycle to bring PDF Mode closer to feature parity with the canvas shell.

# Reproducibility

1. From `/home/rok/sync/ideas/rpdf`, inspect `src/app.rs` to review the canvas interaction loop and viewport transforms.
2. Verify compilation with `cargo check`.
