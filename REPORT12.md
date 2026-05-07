# Title

rpdf reading and export service boundary report

# Context

This task implemented the current `TODO.md` item `formalize-reading-and-export-services`.

The previous two worker tasks established clearer file boundaries and clearer mode-specific UI state, but the reading-support and export behavior still lived as direct ad hoc calls inside the UI logic. That was the main blocker before adding OCR fallback, because more reading logic would otherwise continue to accumulate directly in the PDF-mode code.

The task needed to stay narrow. The goal was not to redesign the whole application architecture, but to create explicit internal boundaries for the current concrete reading-support and canvas-export behavior.

# Goals

- Introduce explicit internal boundaries for current reading-support operations.
- Introduce an explicit internal boundary for current canvas SVG export operations.
- Rewire the existing picker, PDF text extraction, reading-span construction, local TTS launch, and SVG export path through those boundaries.
- Keep the current behavior working without changing backends.
- Verify the result with `cargo check`.

# Approach

The smallest complete slice was to add service structs instead of jumping directly to a broader trait-heavy abstraction.

That produced two explicit boundaries:

- `ReadingSupportService`
- `CanvasExportService`

Those are owned by a new app-level `AppServices` container. This approach was chosen because it satisfies the task contract, gives later work a real place to land, and avoids speculative indirection that a senior engineer would likely consider premature at this project stage.

# Implementation

New service code lives in [src/app/services.rs](/home/rok/sync/ideas/rpdf/src/app/services.rs:1).

The file introduces:

- `AppServices`
- `ReadingSupportService`
- `CanvasExportService`

`ReadingSupportService` now owns the current internal boundary for:

- PDF file picking
- best-effort PDF page counting
- best-effort text extraction
- reading-span construction
- local TTS process launch

`CanvasExportService` now owns the current internal boundary for:

- SVG incompatibility detection for selected canvas items
- SVG document construction
- SVG file writing

The app root in [src/app/mod.rs](/home/rok/sync/ideas/rpdf/src/app/mod.rs:1) now creates and stores `services: AppServices` on `RpdfApp`.

Call sites were updated as follows:

- [src/app/pdf.rs](/home/rok/sync/ideas/rpdf/src/app/pdf.rs:1)
  - PDF picker now goes through `self.services.reading_support.pick_pdf_path()`
  - page-count estimation now goes through `self.services.reading_support.best_effort_pdf_page_count(...)`
  - text extraction now goes through `self.services.reading_support.best_effort_extract_pdf_text(...)`
  - reading-span construction now goes through `self.services.reading_support.build_reading_spans(...)`
  - local TTS launch now goes through `self.services.reading_support.start_local_tts(...)`

- [src/app/canvas.rs](/home/rok/sync/ideas/rpdf/src/app/canvas.rs:1)
  - SVG incompatibility detection now goes through `self.services.canvas_export.first_incompatibility(...)`
  - SVG document generation now goes through `self.services.canvas_export.build_svg_document(...)`
  - SVG file writing now goes through `self.services.canvas_export.write_svg_document(...)`

The backlog in [TODO.md](/home/rok/sync/ideas/rpdf/TODO.md:1) was updated so:

- `formalize-reading-and-export-services` is marked done
- `add-text-fallback-and-warning-flow` remains the current next task

# Results

The service-boundary slice completed successfully.

Observable outcomes:

- reading-support logic now has a dedicated internal service home instead of being only direct UI-owned calls
- canvas SVG export logic now has a dedicated internal service home instead of being only direct canvas-UI-owned calls
- the next OCR task can add fallback behavior against explicit service boundaries rather than expanding direct UI logic further

Verification:

- Ran `cargo check`
- Result: passed

# Decisions

- Used service structs rather than traits for this pass. That keeps the change small while still creating explicit boundaries.
- Kept helper internals in `src/app/util.rs` and used the new services as the public internal boundary. That avoided unnecessary churn.
- Did not swap TTS or export backends in this task. The task was about internal call boundaries, not backend replacement.

# Limitations

- These services are still concrete and app-local. They are not yet full adapter traits.
- Warning classification and OCR fallback policy are still implemented in the PDF flow, not yet centralized in a richer reading-support pipeline.
- The canvas export service currently covers only the SVG path, because that was the active export behavior already present in the app.
- Verification was compile-level only. No runtime export or TTS integration test was added in this pass.
- The worktree still contains unrelated pre-existing deletions of `REPORT1.md` through `REPORT9.md`, which were left untouched.

# Next steps

1. Complete `add-text-fallback-and-warning-flow`.
   This is now the right next task because the reading-support code has a clearer place for OCR and fallback logic to land.

2. Then complete `add-save-load-and-recovery`.
   Persistence remains a separate major behavior area and should stay isolated from the OCR task.

3. After that, complete `add-offline-acceptance-checks`.
   The fallback ladder and persistence work should both exist before writing the higher-level verification pass.

# Reproducibility

Working directory:

- `/home/rok/sync/ideas/rpdf`

Files changed for this task:

- [src/app/mod.rs](/home/rok/sync/ideas/rpdf/src/app/mod.rs:1)
- [src/app/services.rs](/home/rok/sync/ideas/rpdf/src/app/services.rs:1)
- [src/app/pdf.rs](/home/rok/sync/ideas/rpdf/src/app/pdf.rs:1)
- [src/app/canvas.rs](/home/rok/sync/ideas/rpdf/src/app/canvas.rs:1)
- [TODO.md](/home/rok/sync/ideas/rpdf/TODO.md:1)

Verification command:

```bash
cargo check
```

Expected result:

- the crate compiles successfully in the default development profile
