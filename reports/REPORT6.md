# Title

Shared canvas and PDF annotation tools added to rpdf

# Context

- Problem:
  The current task in `TODO.md` was `add-pdf-and-canvas-annotation-tools`. Both workspaces existed, but only the canvas had basic drawing behavior and neither workspace had a shared notion of annotation tool selection, highlighter behavior, or simple note placement.
- Constraints:
  The task had to stay focused on overlay-based annotation and not depend on PDF text extraction, OCR, or TTS. It also needed to respect page boundaries in PDF Mode, which required a small evolution of the PDF annotation model so pen strokes could belong to a specific page.

# Goals

- Primary success criteria:
  Let the user draw, highlight, and place simple text notes in both Infinite Canvas Mode and PDF Mode.
- Secondary success criteria:
  Keep annotation behavior low-friction, share tool state between workspaces, and preserve usability on scanned or structurally weak PDFs by using direct overlay annotation instead of text-dependent markup.

# Approach

- Chosen approach:
  Added a shared annotation toolbar with ink/highlighter tool selection and note text entry, then wired it into both workspaces. Canvas drawing now respects the selected tool, PDF Mode gained a page-local overlay drawing path, and both modes can place simple text notes through the same shared note flow.
- Rejected options:
  Did not wait for a later advanced tool system before adding shared annotation behavior because annotation is already a core product requirement. Did not make PDF annotation depend on text layout or PDF structure because the specification explicitly requires annotation to remain dependable even on scanned or messy documents.

# Implementation

- Architecture / flow:
  `src/app.rs` now includes `AnnotationToolState` in the shell, with one current tool and one pending note text. Canvas strokes use the selected brush style, PDF Mode maintains its own in-progress page-local stroke state, and committed PDF strokes are stored as `PdfPenStrokeAnnotation` values tagged with their page index.
- Key files or components:
  - `src/model/mod.rs`: updated the PDF annotation model so pen-stroke annotations track their target page.
  - `src/app.rs`: added the shared annotation toolbar, canvas/PDF note placement, PDF overlay drawing, and annotation rendering in both workspaces.
  - `TODO.md`: advanced task status after completion.
  - `SUBTODO.md`: cleared after the task finished.
- Example:
  `commit_pdf_stroke` converts the current in-progress PDF overlay stroke into a `PdfPenStrokeAnnotation` with the current `page_index`, which keeps annotations attached to the page they were drawn on when navigation changes later.

# Results

- Outputs:
  Updated:
  - `src/model/mod.rs`
  - `src/app.rs`
  - `TODO.md`
  - `SUBTODO.md`
- Metrics or observations:
  The application now supports:
  - shared ink and highlighter tool selection
  - shared note text entry for both workspaces
  - canvas note placement
  - PDF note placement
  - PDF overlay pen/highlighter drawing tied to the active page
  - rendering of PDF annotations and active in-progress PDF strokes
- Verification:
  Ran `cargo check` successfully after the model and annotation changes.

# Decisions

- Tradeoffs made:
  - Extended the PDF model to include page-local stroke annotations because page ownership is necessary for correct document behavior.
  - Used a shared annotation toolbar across both modes to keep the experience coherent and low-friction.
  - Reused overlay drawing rather than text-aware markup so annotation stays functional regardless of PDF structure quality.

# Limitations

- Known issues, uncertainties, or risks:
  - Annotation editing and selection are not implemented yet; annotations are append-only.
  - PDF annotation coordinates are currently tied to the placeholder page viewport rather than a real rendered PDF page.
  - Runtime verification remains manual; the task was compile-verified only.
  - Annotation appearance customization for recolored vs normal PDF viewing still belongs to the next recoloring task.

# Next steps

1. Implement `add-selection-aware-svg-export` because the canvas now has enough mixed content to make export-target eligibility meaningful.
2. Implement `add-pdf-recolor-and-annotation-visibility` after that so the PDF overlay tools can be tested against recolored viewing modes.

# Reproducibility

1. From `/home/rok/sync/ideas/rpdf`, inspect `src/app.rs` and `src/model/mod.rs` to review the shared annotation state and PDF page-local annotation model.
2. Verify compilation with `cargo check`.
