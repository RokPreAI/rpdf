# Title

PDF document opening and page navigation added to rpdf

# Context

- Problem:
  The current task in `TODO.md` was `add-pdf-viewer-and-navigation`. PDF Mode still had only a placeholder panel, so the project needed a real document-focused workspace with an open flow and stable page navigation before annotation, recoloring, or TTS could be layered onto it.
- Constraints:
  This task had to remain focused on document identity and navigation. It could not expand into annotation, recoloring, or reading-support execution. It also avoided introducing a heavy PDF dependency by using a best-effort page-count heuristic from the opened file data rather than full rendering or parsing.

# Goals

- Primary success criteria:
  Allow the user to open a PDF path into PDF Mode and move through the document by page in a stable document-focused view.
- Secondary success criteria:
  Keep the page state attached to `PdfDocumentSession` and expose clear PDF-root UI entry points for later annotation, recoloring, and TTS work.

# Approach

- Chosen approach:
  Added a dedicated PDF toolbar with a path field, explicit open action, previous/next navigation, and direct page-number control. The opened document is represented by its path plus a best-effort page count derived from the file contents, and the workspace renders a page-shaped document viewport rather than behaving like the infinite canvas.
- Rejected options:
  Did not add full PDF rendering or parsing in this task because it would have widened the task considerably and overlapped with later PDF feature work. Did not reuse the canvas surface because the specification explicitly requires PDF Mode to stay document-focused.

# Implementation

- Architecture / flow:
  `src/app.rs` now includes `PdfInteractionState` for the pending open path and current page count. `render_pdf_toolbar` drives open and page navigation, `open_pdf_document` updates the active `PdfDocumentSession`, and `render_pdf_workspace` paints a document-shaped viewport card centered in the mode area.
- Key files or components:
  - `src/app.rs`: added `PdfInteractionState`, the PDF toolbar, best-effort page counting, page stepping, and document viewport rendering.
  - `TODO.md`: advanced task state after completion.
  - `SUBTODO.md`: cleared after the task finished.
- Example:
  `step_pdf_page` clamps page navigation between page `1` and the current `page_count`, so page movement remains deterministic even when the source PDF path is changed or reopened.

# Results

- Outputs:
  Updated:
  - `src/app.rs`
  - `TODO.md`
  - `SUBTODO.md`
- Metrics or observations:
  PDF Mode now supports:
  - opening a PDF path into the active session
  - best-effort page counting from the file contents
  - previous-page and next-page navigation
  - direct page-number changes
  - a page-shaped document viewport that is visually distinct from the infinite canvas
- Verification:
  Ran `cargo check` successfully after the PDF navigation changes.

# Decisions

- Tradeoffs made:
  - Chose a best-effort page-count heuristic instead of a full PDF engine to keep the task narrow and dependency-light.
  - Kept the document viewport as a stable placeholder card because later tasks need a real PDF-mode root before they need fully rendered page contents.
  - Stored PDF navigation state separately from canvas interaction state so later PDF features can grow independently.

# Limitations

- Known issues, uncertainties, or risks:
  - The page count is heuristic and may be inaccurate for some PDFs.
  - PDF pages are not yet rendered from the source document.
  - Runtime verification remains manual; the task was compile-verified only.
  - Annotation, recoloring, and TTS are still not attached to the PDF workspace.

# Next steps

1. Implement `add-pdf-and-canvas-annotation-tools` because both workspaces now have stable surfaces that can host shared markup behavior.
2. Implement `add-pdf-recolor-and-annotation-visibility` after annotation so recoloring and annotation palettes can be tested against a real PDF workspace.

# Reproducibility

1. From `/home/rok/sync/ideas/rpdf`, inspect `src/app.rs` to review the PDF toolbar and page navigation flow.
2. Verify compilation with `cargo check`.
