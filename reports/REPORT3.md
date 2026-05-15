# Title

PDF workspace shell, backend open flow, and annotation stage report

# Context

`TODO.md` selected `build-pdf-mode-shell` as the next current task after the app-shell split and the canvas pressure pass. PDF Mode already existed as a visual placeholder, but it did not yet behave like a document workspace: there was no document-opening flow, no page state, no dedicated viewer stage, and no annotation surface tied to pages.

The main constraints were:

- keep the implementation bounded to the PDF workspace slice
- use the Rust backend command layer for document-opening flow instead of treating PDF Mode as frontend-only mock UI
- avoid pretending Pdfium rendering is already configured when the current adapter boundary still reports boundary-only status
- leave full TTS/OCR behavior for the later reading task while still exercising the existing extraction contract

# Goals

- Primary success criteria:
  - create a dedicated PDF workspace layout built around document reading rather than an infinite-canvas metaphor
  - add a backend-backed PDF open/inspect flow
  - add page navigation state and a viewer stage
  - add a page-scoped annotation overlay surface

- Secondary success criteria:
  - surface backend readiness and text-reliability state honestly
  - keep the workspace useful even when page rendering is still unavailable on the current machine

# Approach

- Chosen approach:
  - Add one narrow Rust command, `open_pdf_document`, that validates the path and returns document metadata suitable for the first PDF workspace pass.
  - Rebuild `src/features/pdf/workspace.ts` into a real document layout with a sidebar, document controls, navigation, trust-state panel, viewer stage, and annotation overlay.
  - Reuse the existing `render_pdf_page` and `extract_pdf_page_text` contracts for refresh/page-loading behavior, while keeping the UI explicit when rendering is still unavailable.

- Rejected options:
  - Waiting for full Pdfium integration before creating the workspace would have blocked several dependent tasks behind one large backend jump.
  - Rendering the PDF as just another background inside Canvas Mode would conflict with the two-mode product direction.
  - Adding file-picker plugins in this cycle would widen scope into dependency and capability work before the core PDF workspace structure was proven.

# Implementation

- Architecture / flow:
  - The Rust service layer now exposes `open_pdf_document`, which:
    - trims and validates the requested path
    - checks that the file exists
    - enforces a `.pdf` extension
    - returns document name, path, backend readiness, and backend notes
  - The frontend PDF workspace now owns a local `PdfWorkspaceState` with:
    - opened document metadata
    - current page index
    - last render result
    - last extraction result
    - page-level annotation strokes
  - When a document is opened:
    - the frontend invokes `open_pdf_document`
    - then refreshes page state through `get_pdf_backend_status`, `render_pdf_page`, and `extract_pdf_page_text`
    - then renders either a page image or an explicit placeholder/warning if rendering is not yet available

- Key files or components:
  - `src/features/pdf/workspace.ts`
  - `src/styles.css`
  - `src/app/types.ts`
  - `src-tauri/src/contracts/dto.rs`
  - `src-tauri/src/app/services.rs`
  - `src-tauri/src/app/commands.rs`
  - `src-tauri/src/lib.rs`
  - `TODO.md`

- Example:
  - A user can now enter a path such as `/home/rok/Documents/paper.pdf`, open it through the Rust command boundary, move between pages with the PDF navigation controls, and draw annotations directly on the page stage.
  - If Pdfium rendering is not configured, the workspace keeps the document open, shows backend notes and reliability state, and explains why page rendering is unavailable instead of silently failing.

# Results

- Outputs:
  - PDF Mode now has a document-oriented two-column layout instead of placeholder cards.
  - The backend now supports a dedicated `open_pdf_document` command for the first PDF open flow.
  - The PDF workspace now includes:
    - a path-based open flow
    - previous/next page controls
    - backend/trust-state summary
    - a viewer stage
    - a page-scoped annotation overlay canvas
  - `TODO.md` now marks `build-pdf-mode-shell` as done and advances the current task to `add-save-load-project-files`.

- Metrics or observations:
  - The page viewer degrades cleanly when the backend cannot render pages yet.
  - The annotation layer is page-scoped in memory, which gives later PDF persistence work a concrete place to attach saved annotations.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success, with the same existing dead-code warnings for domain models that are still not fully wired into persistence/features

# Decisions

- Fact:
  - The first PDF open flow uses a typed path field and Rust validation instead of a native file-picker plugin.
  - Assessment:
  - This kept the cycle bounded and still satisfied the requirement that document opening go through the backend command boundary.

- Fact:
  - The PDF workspace now calls `render_pdf_page` and `extract_pdf_page_text` even though the renderer still returns backend-limited results on this machine.
  - Assessment:
  - This was the right tradeoff because it proves the intended control flow now and keeps the UI honest about missing backend capability.

- Fact:
  - Page annotations are currently stored only in frontend memory.
  - Assessment:
  - This is acceptable for the workspace-shell task because persistent storage is already the next explicit task in `TODO.md`.

# Limitations

- The open flow currently requires entering a filesystem path manually; there is no native picker yet.
- `open_pdf_document` does not report page count yet because the current backend boundary is not doing real PDF introspection.
- Page rendering still depends on later Pdfium integration or configuration, so the viewer may remain in placeholder mode on the current machine.
- PDF annotations are not yet saved or restored across sessions.

# Next steps

1. Complete `add-save-load-project-files` so both canvas projects and PDF study sessions can persist the newer internal models instead of losing state on reload.
2. Complete `add-tts-and-reliability-pipeline` so the current trust-state panel and extraction calls become real reading support instead of a shell around unavailable states.
3. Complete `add-pdf-page-import-and-recolor` so PDF Mode grows beyond document staging into page appearance controls and import into Canvas Mode when needed.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Rust backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Inspect the PDF workspace implementation:
   - `src/features/pdf/workspace.ts`
4. Inspect the backend open-flow command:
   - `src-tauri/src/app/services.rs`
