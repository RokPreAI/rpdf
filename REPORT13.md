# Title

rpdf offline OCR fallback and warning flow report

# Context

This task implemented the current `TODO.md` item `add-text-fallback-and-warning-flow`.

The project already had a modular `ReadingSupportService`, PDF text-to-speech (TTS) controls, and simulated follow-along highlighting, but the reading path still stopped after one native text attempt. That meant weak or scanned PDFs produced a pre-OCR failure even on a machine that already had local OCR tools installed.

The plan requires an honest fallback ladder:

- native PDF text first
- OCR second
- clear warning if neither path is usable

This work needed to stay narrow. The goal was not to redesign PDF rendering or add persistence, only to make reading support degrade honestly through the existing service boundary.

# Goals

- Add a real offline OCR fallback path behind `ReadingSupportService`.
- Improve native text extraction so the service tries a tool-built path before falling back to raw printable bytes.
- Keep `ReadingSupportState` consistent across native success, OCR fallback success, and total failure.
- Surface enough UI state that the user can see which reading-support path is active.
- Verify the change with focused automated checks.

# Approach

The chosen approach was to extend the existing app-local service boundary rather than introducing a new trait layer.

That produced one bounded slice:

- keep native extraction in the service
- add text-quality assessment there
- add OCR invocation there
- let `src/app/pdf.rs` consume one resolved reading outcome instead of re-implementing fallback rules in UI code

Rejected option:

- adding a placeholder OCR adapter without real OCR execution
  - This was unnecessary because the environment already had `pdftotext`, `pdftoppm`, and `tesseract`, so a real offline path was available now.

# Implementation

The main implementation lives in [src/app/services.rs](/home/rok/sync/ideas/rpdf/src/app/services.rs:1).

Key additions:

- `ReadingSupportResolution`
  - carries resolved spans, text source, reliability, effective highlight mode, and any user-visible warning
- `ReadingSupportService::resolve_reading_support(...)`
  - runs the full fallback ladder and returns one consistent result object
- `ReadingSupportService::best_effort_ocr_pdf_text(...)`
  - renders up to the first 3 PDF pages with `pdftoppm`
  - runs `tesseract` on the generated images
  - joins recognized text into one normalized reading string
- text-quality helpers
  - classify extracted text as `Good`, `Weak`, or `Unusable`
  - map that quality to `ReadingReliability`

Native extraction in [src/app/util.rs](/home/rok/sync/ideas/rpdf/src/app/util.rs:127) was also improved:

- first try `pdftotext <pdf> -`
- normalize that output if successful
- only then fall back to the older printable-byte scan

PDF-mode integration lives in [src/app/pdf.rs](/home/rok/sync/ideas/rpdf/src/app/pdf.rs:1).

Important behavior changes:

- opening a PDF now resets prior reading playback and sets a status warning that reading support will be evaluated on TTS start
- starting TTS now calls `resolve_reading_support(...)` instead of directly handling one native text attempt
- OCR success sets:
  - `TextSupportSource::OcrDerivedText`
  - `ReadingReliability::BestEffort`
  - `WarningCode::OcrFallbackUsed`
  - `HighlightMode::ManualFallback`
- total failure sets:
  - `TextSupportSource::Unavailable`
  - `ReadingReliability::Unreliable`
  - `WarningCode::ReadingSupportUnavailable`
- the toolbar now shows the active text source and reliability so the fallback path is visible to the user
- manual fallback is also selectable in the highlight-mode controls

Backlog state in [TODO.md](/home/rok/sync/ideas/rpdf/TODO.md:1) was advanced:

- `add-text-fallback-and-warning-flow` marked done
- `add-save-load-and-recovery` promoted to current

# Results

Observable outcomes:

- weak native PDF text no longer ends the TTS flow immediately if OCR can recover usable text
- OCR fallback is real and offline, using tools already present on the machine
- the reading UI now exposes:
  - current text source
  - reliability classification
  - warning message when fallback or failure occurs
- the current backlog item moved forward to persistence work

Verification:

- Ran `cargo fmt`
- Ran `cargo test`
- Result: passed

Automated tests added:

- whitespace normalization helper behavior
- unusable short/sparse extracted text classification
- reasonable native text classification
- garbled long-token text classification

# Decisions

- Used the existing service boundary instead of introducing a new abstraction layer.
  - Fact: fallback logic now lives in `ReadingSupportService`.
  - Assessment: this keeps OCR-related behavior out of the PDF UI code without broadening scope.

- Limited OCR to the first 3 pages at 150 DPI.
  - Fact: `pdftoppm` renders only a bounded page subset for OCR.
  - Assessment: this keeps the fallback responsive enough for a first reading-support slice while still handling scanned openings realistically.

- Forced OCR-derived playback into `HighlightMode::ManualFallback`.
  - Fact: OCR success does not pretend to provide precise structure-aware follow-along.
  - Assessment: this keeps the behavior honest, which is more important here than mimicking precise highlighting.

# Limitations

- OCR quality still depends on the local `tesseract` binary and its installed language data.
- The fallback currently scans only the first 3 pages, so later-page text is not yet included in OCR-derived playback.
- Native-text quality heuristics are intentionally simple and may misclassify edge-case PDFs.
- No end-to-end UI automation was added; verification is compile and unit-test level.
- `spd-say` launch success is still not checked before the UI moves into `Playing` state.
- The worktree still contains unrelated pre-existing changes such as deleted `REPORT1.md` through `REPORT9.md` and untracked planning artifacts; those were left untouched.

# Next steps

1. Complete `add-save-load-and-recovery`.
   This is the next current task in `TODO.md`, and the app now needs durable editable-state persistence to match the reading-support work.

2. Add acceptance checks for weak-PDF fallback scenarios.
   The new OCR path should be covered by repeatable verification once the acceptance-check task starts.

3. Revisit TTS process-state honesty after persistence.
   The app still assumes local speech launch succeeds; later work should confirm or report launch failures explicitly.

# Reproducibility

1. Work in `/home/rok/sync/ideas/rpdf`.
2. Verify the task with:

```bash
cargo fmt
cargo test
```

3. The OCR fallback path depends on these local binaries being available in `PATH`:

```bash
command -v pdftotext
command -v pdftoppm
command -v tesseract
```

4. Relevant files changed for this task:

- [src/app/services.rs](/home/rok/sync/ideas/rpdf/src/app/services.rs:1)
- [src/app/pdf.rs](/home/rok/sync/ideas/rpdf/src/app/pdf.rs:1)
- [src/app/util.rs](/home/rok/sync/ideas/rpdf/src/app/util.rs:127)
- [TODO.md](/home/rok/sync/ideas/rpdf/TODO.md:1)
- [REPORT13.md](/home/rok/sync/ideas/rpdf/REPORT13.md:1)
