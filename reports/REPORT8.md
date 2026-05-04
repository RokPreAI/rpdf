# Title

PDF recoloring and annotation palette controls added to rpdf

# Context

- Problem:
  The current task in `TODO.md` was `add-pdf-recolor-and-annotation-visibility`. PDF Mode already had document navigation and overlay annotation, but there was still no live recolor view, no separate annotation palettes for normal versus recolored viewing, and no way to recolor imported PDF pages on the canvas.
- Constraints:
  The task had to stay at the view-state and palette layer. It could not widen into true PDF export or OCR/TTS logic. It also needed to preserve overlay annotation behavior while changing the colors those annotations use under recolored viewing.

# Goals

- Primary success criteria:
  Add live PDF recolor controls, separate annotation palettes for normal and recolored viewing, and per-selection recolor behavior for imported PDF pages on the canvas.
- Secondary success criteria:
  Expose the export recolor choice in the PDF UI state so later PDF-export work already has a user-facing contract to attach to.

# Approach

- Chosen approach:
  Extended the existing PDF toolbar with recolor enable/disable state, foreground/background color editing, annotation palette editing, and export-mode selection. Imported PDF pages in the canvas now reuse the current PDF recolor profile when the user applies recolor to selected page items.
- Rejected options:
  Did not implement actual PDF export in this task because the requirement here was the view/export choice state, not the file-writing pipeline. Did not recolor non-PDF canvas items because the specification only called for recolor behavior on PDFs and imported PDF pages.

# Implementation

- Architecture / flow:
  `src/app.rs` now computes page fill/stroke colors from `PdfViewState.recolor`, computes annotation colors from the active normal/recolored palette, and exposes palette editors in the PDF toolbar. Canvas imported PDF pages can receive or clear a recolor override when their item IDs are selected in the canvas toolbar.
- Key files or components:
  - `src/app.rs`: added recolor controls, annotation palette editors, PDF-page color selection, and canvas imported-PDF recolor application.
  - `TODO.md`: advanced task status after completion.
  - `SUBTODO.md`: cleared after the task finished.
- Example:
  `current_annotation_palette` switches between `normal_view` and `recolored_view` palettes based on whether a PDF recolor profile is active, so the same annotation objects remain visible under both viewing modes without rewriting the underlying annotation data.

# Results

- Outputs:
  Updated:
  - `src/app.rs`
  - `TODO.md`
  - `SUBTODO.md`
- Metrics or observations:
  The application now supports:
  - enabling and disabling PDF recolor view
  - editing PDF foreground and background recolor values
  - choosing whether future PDF export should preserve original appearance or include recoloring
  - editing separate normal-view and recolored-view annotation palettes
  - applying or clearing recolor overrides on selected imported PDF pages in the canvas
- Verification:
  Ran `cargo check` successfully after the recolor and palette changes.

# Decisions

- Tradeoffs made:
  - Used the current PDF recolor profile as the source for canvas imported-PDF recolor overrides, which keeps the behavior coherent across both modes.
  - Left annotation data unchanged and swapped palettes at render time, which is simpler and keeps recolor behavior reversible.
  - Limited the task to view/export-choice state rather than trying to implement a full PDF export writer.

# Limitations

- Known issues, uncertainties, or risks:
  - PDF export still does not exist; only the recolor export choice state is implemented.
  - Imported PDF pages use placeholder cards, so recolor is visible as a placeholder tint rather than a true rasterized page recolor.
  - Runtime verification remains manual; the task was compile-verified only.
  - Text-to-speech and follow-along highlighting still are not active in PDF Mode.

# Next steps

1. Implement `add-pdf-tts-and-highlight-modes` because PDF Mode now has the document, annotation, and recolor layers needed to host reading support.
2. After priority-3 work is complete, move to `add-text-fallback-and-warning-flow` so PDF reading support can degrade honestly on weak-text documents.

# Reproducibility

1. From `/home/rok/sync/ideas/rpdf`, inspect `src/app.rs` to review the recolor controls, palette editors, and canvas imported-PDF recolor handling.
2. Verify compilation with `cargo check`.
