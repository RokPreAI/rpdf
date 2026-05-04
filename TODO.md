# TODO

## Current TODOs

- [ ] 4 Ocr add-text-fallback-and-warning-flow

## Active TODOs

- [ ] 4 Persistence add-save-load-and-recovery
- [ ] 5 Validation add-offline-acceptance-checks

## Done TODOs

- [x] 3 Tts add-pdf-tts-and-highlight-modes
- [x] 3 Recolor add-pdf-recolor-and-annotation-visibility
- [x] 3 Export add-selection-aware-svg-export
- [x] 2 Annotation add-pdf-and-canvas-annotation-tools
- [x] 2 Pdf add-pdf-viewer-and-navigation
- [x] 2 Canvas add-canvas-assets-and-backgrounds
- [x] 1 Canvas implement-canvas-pen-and-viewport
- [x] 1 Foundation bootstrap-desktop-app-shell
- [x] 0 Foundation define-document-models

## Task Details

### define-document-models

Priority: 0
Area: Foundation
Status: done
Depends on: none

Goal:
Define the core editable data models and boundaries needed by the specification so later implementation work has a stable contract for canvas documents, placed assets, annotations, view settings, and export targets.

Context:
`SPEC.md` requires two equal modes, portable export rules, recoloring-aware behavior, reliable annotation on weak PDFs, and selection-aware SVG export. Those behaviors need clear internal object boundaries before implementation work starts, otherwise later tasks will guess incompatible representations.

Expected changes:
- New source files for the application's document and state model.
- Definitions for infinite-canvas items, imported PDF pages, images, typed text, pen strokes, annotations, selections, recolor settings, and export targets.
- Definitions for PDF-mode annotation state and reading-support state.
- Notes in code or adjacent docs that distinguish user-facing portable outputs from internal autosave/recovery state.

Acceptance criteria:
- The project has a single documented representation for canvas content, PDF annotation state, selection targets, and export eligibility.
- The representation distinguishes SVG-compatible content from incompatible content in a way that later export logic can use directly.
- The representation distinguishes temporary view settings from persistent/exportable output choices.
- A later worker can implement canvas, PDF, TTS, and export tasks without redefining the core state model.

Notes:
Keep this task at the contract/data-boundary level. Do not lock in rendering or storage technology beyond what is necessary to make later tasks concrete.

### bootstrap-desktop-app-shell

Priority: 1
Area: Foundation
Status: done
Depends on: define-document-models

Goal:
Create the minimal desktop application shell that can host two first-class modes and basic offline startup without yet implementing the full workflows.

Context:
The specification requires a desktop app with a minimal UI, two equal modes, and no dependence on internet access. A clean shell is needed before feature work can be verified interactively.

Expected changes:
- Application entrypoint and launch flow.
- Top-level window and mode-switching shell.
- Minimal layout for Infinite Canvas Mode and PDF Mode.
- Basic local configuration and startup state.

Acceptance criteria:
- The app starts locally without network access.
- The user can open the app and switch between a canvas workspace and a PDF workspace.
- The shell is minimal and does not embed office-suite-style clutter by default.
- Later workers have a stable place to attach mode-specific behavior.

Notes:
This task is about the host application structure and user-visible shell behavior, not full feature completeness.

### implement-canvas-pen-and-viewport

Priority: 1
Area: Canvas
Status: done
Depends on: define-document-models, bootstrap-desktop-app-shell

Goal:
Implement the core infinite-canvas interaction loop: pen drawing, pressure-sensitive strokes, panning, zooming, and item placement on a workspace larger than a fixed page.

Context:
The specification makes the canvas a first-class mode for visual note-taking. This is one of the strongest validated parts of the product and unlocks many later tasks.

Expected changes:
- Canvas rendering and interaction code.
- Pen-input handling with pressure-sensitive stroke creation.
- Viewport pan/zoom behavior.
- Creation and movement of core canvas items.

Acceptance criteria:
- A user can create a canvas and draw pressure-sensitive strokes with a drawing tablet.
- A user can pan and zoom the workspace.
- The workspace is not constrained to a single page.
- Canvas interaction remains usable without any PDF or TTS features enabled.

Notes:
Prioritize directness and low-friction interaction over advanced editing features.

### add-canvas-assets-and-backgrounds

Priority: 2
Area: Canvas
Status: done
Depends on: implement-canvas-pen-and-viewport

Goal:
Add typed text placement, image import, PDF page import, and zoom-aware reference backgrounds to Infinite Canvas Mode.

Context:
The specification requires mixed-content study canvases and scalable dots/lines/squares backgrounds that remain useful as spatial references.

Expected changes:
- Canvas text placement behavior.
- Image import behavior.
- PDF-page import behavior.
- Background pattern controls for dots, lines, and squares.
- Zoom-aware pattern scaling logic.

Acceptance criteria:
- A user can place typed text on the canvas.
- A user can import an image and at least one PDF page into the canvas.
- A user can choose dots, lines, or squares as the background pattern.
- The background remains a meaningful size reference while zooming in and out.

Notes:
Imported PDF pages are visual elements in canvas mode and must remain usable regardless of text quality inside the source PDF.

### add-pdf-viewer-and-navigation

Priority: 2
Area: Pdf
Status: done
Depends on: define-document-models, bootstrap-desktop-app-shell

Goal:
Implement PDF Mode as a document-focused workspace with opening, page navigation, and stable viewing behavior separate from the infinite canvas.

Context:
The specification requires PDF Mode to stay distinct from the canvas while supporting later annotation, recoloring, and TTS features.

Expected changes:
- PDF open/load flow.
- Page rendering and page navigation behavior.
- Document-focused viewport behavior.
- Mode-specific controls or commands relevant to PDF reading.

Acceptance criteria:
- A user can open a PDF in PDF Mode.
- A user can move through the document reliably.
- PDF Mode behaves as a document reader, not as an infinite canvas.
- The mode can later host annotation, recoloring, and TTS without structural rework.

Notes:
Keep the PDF workspace narrow and reading-focused.

### add-pdf-and-canvas-annotation-tools

Priority: 2
Area: Annotation
Status: done
Depends on: implement-canvas-pen-and-viewport, add-pdf-viewer-and-navigation

Goal:
Implement the core annotation tools shared across the product: freehand strokes, highlighting, and simple text notes, with dependable behavior in both modes.

Context:
The specification treats annotation as core in both the canvas and PDF workflows, including scanned or messy PDFs.

Expected changes:
- Tool selection and tool-state behavior.
- Freehand markup for canvas and PDF mode.
- Highlighting behavior.
- Simple text-note behavior.
- Shared annotation rendering rules and editing affordances.

Acceptance criteria:
- A user can annotate both a canvas and an opened PDF.
- A user can draw, highlight, and place simple text notes.
- Annotation remains usable on a scanned or poorly structured PDF without depending on text extraction.
- Annotation interaction remains low-friction and direct.

Notes:
This task should not depend on TTS or OCR to be useful.

### add-selection-aware-svg-export

Priority: 3
Area: Export
Status: done
Depends on: add-canvas-assets-and-backgrounds, add-pdf-and-canvas-annotation-tools

Goal:
Implement canvas export behavior that defaults to the whole canvas, switches to the current selection when present, and offers SVG export only for compatible targets.

Context:
The specification explicitly requires portable export behavior and clear refusal for incompatible SVG targets.

Expected changes:
- Export target selection logic.
- SVG-eligibility checks for canvas content.
- Whole-canvas vs selected-items export behavior.
- Clear unavailable/refusal behavior for incompatible SVG targets.

Acceptance criteria:
- With no active selection, export targets the whole canvas.
- With a selection, export targets that selection.
- A vector-and-text-only target offers SVG export.
- A target containing incompatible content such as imported PDF pages or raster images does not offer misleading SVG export.
- A vector-only selection taken from a mixed canvas can still export as SVG.

Notes:
Be explicit about why a target is ineligible instead of silently degrading the result.

### add-pdf-recolor-and-annotation-visibility

Priority: 3
Area: Recolor
Status: done
Depends on: add-pdf-viewer-and-navigation, add-pdf-and-canvas-annotation-tools

Goal:
Implement PDF recoloring as a reading aid and ensure annotation appearance remains usable in both normal and recolored viewing contexts.

Context:
The specification requires recoloring both as a live reading feature and as an optional export choice, with separate annotation-appearance control for normal and recolored viewing.

Expected changes:
- PDF recolor controls in PDF Mode.
- Recolor support for imported PDF pages on the canvas.
- Annotation-appearance settings for normal and recolored viewing.
- Export-time recolor inclusion choice for annotated PDFs.

Acceptance criteria:
- A user can recolor the PDF view during reading.
- A user can configure annotation appearance for normal view and recolored view separately.
- Imported PDF pages on the canvas can be recolored per page or across a multi-selection.
- The user can choose whether exported annotated PDF output includes recoloring.

Notes:
Keep the behavior consistent with “viewing aid first, optional output choice second.”

### add-pdf-tts-and-highlight-modes

Priority: 3
Area: Tts
Status: done
Depends on: add-pdf-viewer-and-navigation

Goal:
Implement PDF text-to-speech and configurable visual follow-along highlighting modes in PDF Mode for readable text-based documents.

Context:
The specification makes reading support equal in importance to annotation and requires user-controlled visual follow-along behavior.

Expected changes:
- TTS playback controls and state.
- Highlighting modes tied to reading progress.
- Visual presentation for word-level, line-level, and sentence-level follow-along behavior when supported by source quality.
- Mode behavior for starting, stopping, and resuming reading.

Acceptance criteria:
- A user can trigger TTS while in PDF Mode.
- The app provides visible follow-along behavior while TTS is active.
- The user can choose among at least multiple highlighting granularities when source quality allows it.
- The result is useful on normal text PDFs even before OCR fallback work is added.

Notes:
This task can start with native text flows. OCR fallback belongs to a later task.

### add-text-fallback-and-warning-flow

Priority: 4
Area: Ocr
Status: current
Depends on: add-pdf-tts-and-highlight-modes

Goal:
Add the required fallback order for unreliable PDF text: native text first, OCR second, and a clear user warning when reliable text-based reading support still cannot be produced.

Context:
This is the main risk-control behavior identified by both `SPEC.md` and `research/RESEARCH.md`. It prevents the product from silently pretending that weak text extraction is reliable.

Expected changes:
- Reliability checks for native PDF text extraction.
- OCR invocation and result-handling behavior.
- Warning states for unreliable text-based reading support.
- Fallback highlighting behavior when precise text-linked highlighting is not available.

Acceptance criteria:
- A user on a weak-text PDF triggers TTS/highlighting and the system attempts native text before OCR.
- If OCR improves usability enough, the system proceeds using that result.
- If neither path is reliable enough, the system warns the user clearly.
- Annotation remains available regardless of text support quality.

Notes:
This task should focus on correctness of fallback behavior, not perfection of OCR quality.

### add-save-load-and-recovery

Priority: 4
Area: Persistence
Status: pending
Depends on: define-document-models, implement-canvas-pen-and-viewport, add-pdf-and-canvas-annotation-tools

Goal:
Implement save, load, autosave, and recovery behavior for editable user work while preserving the project's offline-first and portable-output goals.

Context:
The specification allows an internal format for autosave and recovery, but user-facing workflows should still prefer portable outputs where practical.

Expected changes:
- Editable document persistence for canvases and annotation state.
- Autosave and recovery flow.
- Recovery prompts or recovery-state handling after interrupted work.
- Clear separation between editable working state and export outputs.

Acceptance criteria:
- A user can save and reopen editable work.
- A user can recover work after an interrupted session using the app's recovery behavior.
- Recovery/internal state does not replace explicit user export options.
- Core save/load workflows work without internet access.

Notes:
Do not let recovery-only storage become the only way users retain their work.

### add-offline-acceptance-checks

Priority: 5
Area: Validation
Status: pending
Depends on: bootstrap-desktop-app-shell, add-canvas-assets-and-backgrounds, add-pdf-and-canvas-annotation-tools, add-selection-aware-svg-export, add-pdf-recolor-and-annotation-visibility, add-pdf-tts-and-highlight-modes, add-text-fallback-and-warning-flow, add-save-load-and-recovery

Goal:
Add repeatable verification for the specification's acceptance checks so future workers can confirm the product remains aligned with the contract.

Context:
The repo currently has only idea/spec artifacts. Once implementation starts, the project needs explicit acceptance checks that map back to `SPEC.md` instead of relying on memory.

Expected changes:
- Test cases, scripted checks, or a manual verification checklist mapped to the specification.
- Sample assets for normal PDFs, messy/scanned PDFs, mixed-content canvases, and vector-only selections.
- Documentation of what can be verified automatically and what still needs manual validation with a drawing tablet.

Acceptance criteria:
- The project has a documented way to verify each acceptance area in `SPEC.md`.
- Offline operation is explicitly checked.
- SVG-export eligibility and refusal behavior are explicitly checked.
- Weak-PDF fallback and warning behavior are explicitly checked.
- Any manual-only checks are clearly identified instead of being left implicit.

Notes:
Keep the acceptance mapping close to the specification so the contract stays executable.
