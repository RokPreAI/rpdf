# Title

Canvas assets and zoom-aware backgrounds added to rpdf

# Context

- Problem:
  The current task in `TODO.md` was `add-canvas-assets-and-backgrounds`. Infinite Canvas Mode could already draw pen strokes, pan, and zoom, but it still lacked typed text, imported image/PDF-page items, and background patterns that act as real study references while zooming.
- Constraints:
  The task needed to stay inside canvas mode. It could not expand into PDF-mode loading, shared annotation tools, or export logic. It also needed to avoid dependency churn, so imported assets were added as stable visual placeholders instead of trying to introduce full media-rendering or PDF-rendering stacks at this stage.

# Goals

- Primary success criteria:
  Let the user place typed text, import image placeholders, import PDF-page placeholders, and switch between dots, lines, and squares backgrounds that scale meaningfully with zoom.
- Secondary success criteria:
  Keep the new items grounded in the existing model layer and make them render on the live canvas without disturbing the existing stroke, pan, and zoom loop.

# Approach

- Chosen approach:
  Extended the canvas toolbar in `src/app.rs` with direct controls for background selection, typed text insertion, image-path import, and PDF-path/page import. Imported images and PDF pages are currently represented as visible canvas items with labeled placeholder cards, which is enough to make the canvas mixed-content and prepares the ground for later richer rendering.
- Rejected options:
  Did not add a full file picker or media/PDF rasterization pipeline in this task. That would have introduced avoidable surface area and dependency complexity before the PDF viewer and export tasks are complete.

# Implementation

- Architecture / flow:
  The canvas toolbar writes into `CanvasInteractionState`, and button actions convert those pending values into `CanvasItem::Text`, `CanvasItem::ImportedImage`, or `CanvasItem::ImportedPdfPage` entries. Background selection updates the document's `BackgroundPattern`, and the painter renders dots, lines, or squares in screen space using spacing derived from world-space spacing multiplied by the current zoom.
- Key files or components:
  - `src/app.rs`: added the canvas toolbar, typed text insertion, image/PDF-page import actions, background pattern selection, and rendering for text and imported placeholder items.
  - `TODO.md`: advanced the current task after completion.
  - `SUBTODO.md`: cleared after the task finished.
- Example:
  `paint_canvas_background` renders dots, lines, or squares using the stored pattern style and the current viewport zoom, so the visible grid spacing grows when the user zooms in and shrinks when the user zooms out.

# Results

- Outputs:
  Updated:
  - `src/app.rs`
  - `TODO.md`
  - `SUBTODO.md`
- Metrics or observations:
  Infinite Canvas Mode now supports:
  - typed text insertion
  - image-path import into visible canvas placeholders
  - PDF-path/page import into visible canvas placeholders
  - dots, lines, and squares backgrounds
  - zoom-aware background spacing
  - rendering of mixed-content canvas items alongside pen strokes
- Verification:
  Ran `cargo check` successfully after the canvas asset and background changes.

# Decisions

- Tradeoffs made:
  - Treated imported images and PDF pages as stable visible placeholders instead of full rendered previews, because the current task required mixed canvas content, not a finished rendering pipeline.
  - Kept text insertion simple and direct through a toolbar field instead of building a more complex in-canvas text editor.
  - Reused the existing viewport transforms for background rendering so zoom-aware reference behavior stays coupled to the same world/screen mapping as strokes.

# Limitations

- Known issues, uncertainties, or risks:
  - Imported images and PDF pages are placeholder cards and do not yet display their full underlying content.
  - There is still no item selection or editing flow for the newly added assets.
  - Runtime verification remains manual; the task was compile-verified only.
  - Canvas background style is switchable, but not yet user-tunable beyond the pattern category.

# Next steps

1. Implement `add-pdf-viewer-and-navigation` because PDF Mode still lacks real document opening and page movement behavior.
2. Implement `add-pdf-and-canvas-annotation-tools` after PDF navigation so both workspaces can share a coherent annotation surface.

# Reproducibility

1. From `/home/rok/sync/ideas/rpdf`, inspect `src/app.rs` to review the canvas toolbar and mixed-content rendering logic.
2. Verify compilation with `cargo check`.
