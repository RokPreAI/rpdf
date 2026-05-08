# rpdf Acceptance Checks

This document is the executable acceptance map for the current `PLAN.md` and `SPEC.md` state.

It separates:

- automated offline checks that can run now
- manual checks that still require the GUI, a drawing tablet, or real study assets

## Quick Run

Run the automated subset from the repo root:

```bash
./scripts/run_acceptance_checks.sh
```

Today that script runs `cargo test`, which covers:

- reading-support text-quality heuristics
- offline persistence round trips for canvas and PDF sessions
- recovery snapshot round trips
- SVG export eligibility gating for compatible and incompatible canvas targets

For the latest UX-hardening findings and the remaining manual-only live-session gaps, also read:

- [STUDY_SESSION_VALIDATION.md](/home/rok/sync/ideas/rpdf/STUDY_SESSION_VALIDATION.md:1)

## Required Manual Assets

Use local offline assets for the manual checks below.

Minimum asset set:

1. One normal text PDF with a usable text layer.
2. One scanned or weak-text PDF where OCR fallback is likely to trigger.
3. One image file for canvas import.

Suggested naming:

- `samples/text-readable.pdf`
- `samples/scanned-or-weak.pdf`
- `samples/reference-image.png`

These assets do not need to live in the repo, but the acceptance run should record which local files were used.

## Acceptance Matrix

### App startup and offline behavior

Goal:
The app starts locally and core workflows do not require internet access.

Automated:

- `./scripts/run_acceptance_checks.sh`

Manual:

1. Disconnect the network or block access.
2. Start the app with:

```bash
cargo run
```

3. Confirm the app opens and the mode switcher renders.
4. Confirm Infinite Canvas Mode and PDF Mode are both reachable without any network prompt or failure.

Status:
Manual-only beyond compile/test coverage.

### Canvas mode and pen workflow

Goal:
Canvas drawing, zoom, panning, mixed content, and background references work in the live UI.

Automated:

- no reliable tablet simulation exists in the current test suite

Manual:

1. Switch to Infinite Canvas Mode.
2. Draw several strokes with a pen device.
3. Confirm pressure visibly affects stroke width.
4. Pan with secondary drag and zoom with scroll.
5. Switch background among dots, lines, and squares.
6. Confirm the background remains visually usable while zooming.
7. Add typed text, import an image, and import a PDF page.

Status:
Manual-only.

### SVG export gating

Goal:
Whole-canvas and selection export must only offer valid SVG output targets.

Automated:

- `cargo test`
- Covered by service-level tests for:
  - vector-only compatibility
  - raster incompatibility
  - imported-PDF incompatibility

Manual:

1. Build a canvas with:
   - one text item
   - one stroke
   - one imported image or imported PDF page
2. Try exporting the whole canvas to SVG and confirm the app refuses incompatible targets clearly.
3. Select only the vector/text items and export again.
4. Confirm the SVG file is written successfully.

Status:
Automated plus manual UI confirmation.

### PDF navigation and annotation

Goal:
PDF Mode remains document-focused while supporting direct annotation.

Automated:

- no full UI navigation harness exists yet

Manual:

1. Open `text-readable.pdf` in PDF Mode.
2. Navigate across multiple pages.
3. Add ink, highlight, and text-note annotations.
4. Confirm PDF Mode stays document-scoped rather than behaving like an infinite canvas.

Status:
Manual-only.

### Reading support fallback and warnings

Goal:
The app follows the reading-support ladder honestly:

1. native text
2. OCR fallback
3. clear warning on failure

Automated:

- `cargo test`
- Covered by:
  - text-quality classification tests
  - persistence tests that ensure reading-support state remains serializable

Manual:

1. Open `text-readable.pdf`.
2. Start TTS.
3. Confirm the UI reports a native text source and active playback.
4. Open `scanned-or-weak.pdf`.
5. Start TTS.
6. Confirm the app either:
   - uses OCR fallback and reports that fallback explicitly, or
   - warns clearly that usable reading text could not be recovered.
7. Confirm annotation remains usable even if reading support degrades.

Status:
Automated heuristics plus manual end-to-end behavior check.

### Save, load, autosave, and recovery

Goal:
Editable working state is durable for both major modes.

Automated:

- `cargo test`
- Covered by:
  - canvas save/load round trip
  - PDF session save/load round trip
  - canvas recovery snapshot round trip
  - PDF recovery snapshot round trip

Manual:

1. In Infinite Canvas Mode, create or modify content.
2. Save to a JSON file and reload it.
3. Confirm the canvas content reappears correctly.
4. Make another unsaved change and wait at least 2 seconds.
5. Use `Recover autosave` and confirm the latest snapshot is restorable.
6. Repeat the same flow in PDF Mode with an opened PDF and annotations.

Status:
Automated plus manual UI confirmation.

### Recolor behavior

Goal:
Recoloring works as a viewing feature and remains legible with annotations.

Automated:

- no dedicated recolor rendering test exists yet

Manual:

1. Open a PDF in PDF Mode.
2. Enable recolor view and change foreground/background colors.
3. Confirm annotations remain visible in both normal and recolored states.
4. On the canvas, apply recolor to selected imported PDF pages and verify the effect is visible.

Status:
Manual-only.

## Current Gaps

These areas are still not fully automated:

- tablet pressure behavior
- GUI rendering/layout behavior
- end-to-end TTS process launch and playback honesty
- recolor rendering correctness
- long-session UX validation

Those gaps are expected at the current stage and should feed the next remaining backlog item:

- `run-study-session-ux-hardening`
