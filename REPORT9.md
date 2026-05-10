# Title

PDF reading pipeline and local TTS report

# Context

After the canvas shape cycle, the remaining highest-priority unfinished task in the requested `0-5` band was `add-tts-and-reliability-pipeline`. PDF Mode already had a shell, annotations, and a trust-state panel, but the actual reading path was still mostly structural because native extraction returned placeholder data and there was no local speech control or OCR fallback path.

This was the correct next cycle because:

- it was the only remaining priority-2 task
- it is one of the product-defining differences between a generic PDF annotator and this app
- the repo already had the PDF shell and trust surfaces needed to host a real reading pipeline
- the machine has local `pdfinfo`, `pdftotext`, `pdftoppm`, and `tesseract`, so the first offline implementation could be real rather than mocked

# Goals

- Primary success criteria:
  - extract native PDF text locally
  - allow local page-level TTS playback
  - expose explicit native/OCR/unavailable reliability states
  - add an OCR fallback trigger that stays honest about trust

- Secondary success criteria:
  - keep the fallback order explicit: native first, OCR second, warning third
  - avoid pretending exact follow-along confidence when extraction is weak
  - improve page metadata like page count while touching the reading path

# Approach

- Chosen approach:
  - Use local system tools instead of adding new crates:
    - `pdfinfo` for page count
    - `pdftotext` for native text extraction
    - `pdftoppm` + `tesseract` for OCR fallback
  - Keep OCR fallback manual and page-scoped so the trust transition is visible to the user.
  - Use the browser runtime’s local speech synthesis for the first offline TTS layer.
  - Extend the extraction DTO with a `sourceKind` field so PDF Mode can distinguish native text from OCR-derived text explicitly.

- Rejected options:
  - Leaving the pipeline as a placeholder until Pdfium extraction exists would keep PDF Mode structurally correct but functionally hollow.
  - Pretending OCR and native extraction are equivalent would violate the trust-state requirement.
  - Adding a large new PDF/text crate stack in the same cycle would widen scope more than necessary when the local toolchain already exists.

# Implementation

- Backend:
  - `src-tauri/src/app/services.rs`
    - `open_pdf_document` now tries to read page count via `pdfinfo`
    - added `extract_pdf_page_ocr`
  - `src-tauri/src/infrastructure/pdf_engine/mod.rs`
    - native extraction now uses `pdftotext`
    - OCR fallback now rasterizes a page with `pdftoppm` and runs `tesseract`
    - extraction responses now classify text as `native_reliable`, `native_weak`, `ocr_reliable`, `ocr_weak`, or `unavailable`
    - backend notes now reflect the real local tool availability instead of claiming extraction is entirely unimplemented
  - `src-tauri/src/contracts/dto.rs`, `src-tauri/src/app/commands.rs`, `src-tauri/src/lib.rs`
    - added `source_kind` to extraction responses
    - exposed the OCR command through Tauri IPC

- Frontend:
  - `src/features/pdf/workspace.ts`
    - PDF Mode now keeps separate native and OCR extraction state
    - added `Read page`, `Stop`, and `Run OCR fallback` controls
    - local TTS now reads the current page text through `SpeechSynthesisUtterance`
    - OCR fallback is enabled only when native extraction is weak or unavailable
    - the trust badge, copy, source label, and extracted-text preview now all reflect the active reading source honestly
    - stopping or page changes cancel active speech so playback stays page-scoped
  - `src/app/types.ts`
    - `PageTextExtraction` now includes `sourceKind`
  - `src/styles.css`
    - added styling for the reading-text preview panel

# Results

- Outputs:
  - PDF Mode can now extract native page text locally when the PDF exposes it.
  - The app can now read extracted page text aloud locally.
  - OCR fallback can now be triggered explicitly when native extraction is weak or unavailable.
  - The UI now distinguishes native text from OCR text and keeps trust messaging visible.
  - `open_pdf_document` now reports page count when `pdfinfo` succeeds.
  - `TODO.md` now marks `add-tts-and-reliability-pipeline` done and advances the current task to `add-pdf-page-import-and-recolor`.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - Native extraction uses `pdftotext` before OCR is offered.
  - Assessment:
  - This directly enforces the required fallback order instead of making OCR the default.

- Fact:
  - OCR fallback is manual and explicit.
  - Assessment:
  - This keeps trust surfaces honest and prevents silent source switching.

- Fact:
  - TTS is implemented with local runtime speech synthesis rather than a cloud service.
  - Assessment:
  - This matches the offline-first requirement and keeps the first reading loop lightweight.

# Limitations

- Page rendering itself is still behind the Pdfium boundary; the reading pipeline is now more complete than the render pipeline.
- The extracted text preview is line-based rather than geometry-faithful follow-along text layout.
- OCR quality depends on local tool availability and the rasterized page quality.
- Math-aware speech is still deferred, as planned.

# Next steps

1. Complete `add-pdf-page-import-and-recolor` so PDF pages can move between PDF Mode and Canvas Mode with a first recolor workflow.
2. Complete `add-config-and-toolbar-icons` so the tool surfaces are easier to scan now that both modes have grown.
3. Revisit PDF rendering later so the viewer side catches up with the improved reading pipeline.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Rust backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Inspect the PDF reading UI:
   - `src/features/pdf/workspace.ts`
4. Inspect the extraction/OCR backend:
   - `src-tauri/src/infrastructure/pdf_engine/mod.rs`
5. Inspect the service-layer page-count wiring:
   - `src-tauri/src/app/services.rs`
