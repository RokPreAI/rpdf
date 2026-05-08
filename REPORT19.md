# Title

rpdf PDF-open hardening plus first-pass selection and eraser tools handoff report

# Context

This worker slice finished two tightly coupled pieces of work in one pass:

- the in-progress PDF-open crash hardening work already present in the local tree
- the current `TODO.md` task `add-selection-and-eraser-tools`

Those had to be combined because the PDF hardening changes already modified `src/app/pdf.rs`, and the new tool work also needed the same file. Splitting them into separate commits at this point would have forced either a risky manual rollback or an artificial partial commit.

Before this task:

- PDF opening trusted the selected path too much and used a fragile whole-file page-count path
- the app only had ink and highlighter as real tools
- canvas selection existed mostly as a checkbox/export concept, not as a direct interaction tool
- there was no eraser workflow for canvas or PDF annotations

# Goals

- Finish the in-progress PDF-open hardening so bad paths or non-PDF files fail cleanly instead of contributing to app crashes.
- Add real `Selection` and `Eraser` tools with a bounded first version.
- Keep the new editing behavior explicit and honest: whole-item selection and whole-item erase, not a fake partial vector editor.
- Verify the combined slice with formatting and automated tests.

# Approach

The chosen approach was to keep the interaction model intentionally simple:

- click-based selection
- click-based whole-item erase
- visible selection outlines
- no partial stroke erasing or geometry editing yet

For PDF opening, the approach was to harden the pre-open inspection step rather than wait for downstream code to fail:

- catch picker panics
- reject obviously invalid files early
- stream-scan page markers instead of reading the whole file at once just to guess page count

Rejected alternative:

- trying to add transform handles, partial erasing, or drag-move behavior in this same slice
  - That would have widened scope sharply and made verification much weaker.

# Implementation

PDF-open hardening:

- [src/app/util.rs](/home/rok/sync/ideas/rpdf/src/app/util.rs:1)
  - added PDF open inspection helpers that:
    - catch picker panics
    - verify the selected path is a non-empty file
    - check for a `%PDF-` header signature
    - stream-count `/Type /Page` markers instead of loading the full file into memory
- [src/app/services.rs](/home/rok/sync/ideas/rpdf/src/app/services.rs:1)
  - added a service-level `inspect_pdf_open_path(...)` preparation step
  - added focused tests for:
    - valid PDF-like path inspection
    - rejecting obvious non-PDF files
- [src/app/pdf.rs](/home/rok/sync/ideas/rpdf/src/app/pdf.rs:1)
  - now routes PDF open through the inspection step and reports bounded `status_message` failures instead of proceeding blindly

Selection and eraser tools:

- [src/app/mod.rs](/home/rok/sync/ideas/rpdf/src/app/mod.rs:1)
  - extended `AnnotationTool` with:
    - `Selection`
    - `Eraser`
  - exposed both in the shared annotation toolbar
  - added PDF UI state for the selected annotation id

Canvas interaction:

- [src/app/canvas.rs](/home/rok/sync/ideas/rpdf/src/app/canvas.rs:1)
  - added click hit-testing for:
    - strokes and highlights
    - text items and notes
    - imported images
    - imported PDF pages
  - added whole-item erase for the same supported item set
  - added visible selection outlines for selected canvas items
  - ensured switching to selection/eraser cancels any active stroke instead of committing stale drawing state

PDF interaction:

- [src/app/pdf.rs](/home/rok/sync/ideas/rpdf/src/app/pdf.rs:1)
  - added click hit-testing for:
    - PDF pen-stroke annotations
    - PDF text notes
  - added whole-item erase for those supported PDF annotations
  - added visible selection outlines for the selected PDF annotation
  - ensured selection/eraser modes cancel active drawing state cleanly

Geometry helpers:

- [src/app/util.rs](/home/rok/sync/ideas/rpdf/src/app/util.rs:1)
  - added helpers for:
    - point-in-rect checks
    - stroke hit-testing
    - stroke bounding boxes

Backlog update:

- [TODO.md](/home/rok/sync/ideas/rpdf/TODO.md:1)
  - marks `add-selection-and-eraser-tools` done
  - keeps `stabilize-pdf-open-path-and-crash-handling` done
  - promotes `add-space-pan-and-fit-to-content` to current

# Results

Verification completed:

- Ran `cargo fmt`
- Ran `cargo test`
- Ran `./scripts/run_acceptance_checks.sh`

Observed results:

- automated tests passed
- the new PDF-open inspection tests passed
- the acceptance script still passed after the interaction-tool changes
- the app now has real selection and eraser tool modes instead of only draw/highlight modes

Concrete behavioral outcomes:

- non-PDF files now fail PDF-open inspection with a bounded error message
- canvas items can now be selected directly by clicking them
- canvas items can now be erased directly by clicking them with the eraser tool
- PDF annotations can now be selected and erased directly in a bounded first-pass form

# Decisions

- Combined the PDF hardening and selection/eraser work into one commit.
  - Fact: both slices needed `src/app/pdf.rs`, and the PDF-hardening changes were already locally in progress.
  - Assessment: bundling them was safer than trying to surgically separate overlapping file edits.

- Kept erase behavior whole-item only.
  - Fact: eraser currently removes entire supported items or annotations.
  - Assessment: this is a clear and verifiable first version, unlike partial-stroke editing.

- Reused the existing canvas selection model for direct interaction instead of inventing a new selection store there.
  - Fact: canvas click selection writes into the existing `SelectionTarget::ItemIds(...)` flow.
  - Assessment: this keeps export and direct selection aligned.

- Added a separate PDF selected-annotation UI field instead of forcing PDF selection into the canvas export-oriented selection structure.
  - Fact: PDF selection now lives in `PdfInteractionState`.
  - Assessment: PDF Mode needs a document-annotation mental model, not canvas-export semantics.

# Limitations

- Canvas selection is currently single-click, whole-item selection only; it does not yet support moving, resizing, or multi-select editing.
- PDF selection is currently a visual and erase-ready state, not a full annotation editing workflow.
- Eraser support is bounded to supported notes and annotation items; it does not do partial stroke erasing.
- Full live GUI reproduction of the original PDF crash is still limited by the current headless environment, so the hardening is verified through code-path inspection and tests rather than a real local GUI opening session here.
- The worktree still contains unrelated pre-existing deletions of `REPORT1.md` through `REPORT9.md` plus untracked planning/skill artifacts, which were left untouched.

# Next steps

1. Implement `add-space-pan-and-fit-to-content`.
   That is now the promoted current task and is the cleanest next interaction improvement.

2. Revisit clipboard image paste after the navigation model is stable.
   That remains a useful lower-friction canvas import path, but it does not block the current core editing interaction.

3. Decide later whether selection should grow into move/transform behavior.
   The current selection tool is useful now, but richer editing should be its own bounded follow-up, not an unplanned expansion of this slice.

# Reproducibility

1. Work in `/home/rok/sync/ideas/rpdf`.
2. Reformat:

```bash
cargo fmt
```

3. Run tests:

```bash
cargo test
```

4. Run the acceptance subset:

```bash
./scripts/run_acceptance_checks.sh
```

5. In a graphical local session, verify manually that:

- opening a bad non-PDF path now fails cleanly
- selection highlights supported canvas items
- eraser removes supported canvas items
- selection highlights supported PDF annotations
- eraser removes supported PDF annotations
