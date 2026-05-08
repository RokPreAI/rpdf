# TODO

## Current TODOs

- [ ] 3 Ux run-study-session-ux-hardening

## Active TODOs

## Done TODOs

- [x] 2 Validation add-offline-acceptance-checks
- [x] 1 Persistence add-save-load-and-recovery
- [x] 0 Ocr add-text-fallback-and-warning-flow
- [x] 1 Services formalize-reading-and-export-services
- [x] 1 Ui extract-mode-shells-and-shared-ui-state
- [x] 0 Architecture refactor-app-into-modules
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

### refactor-app-into-modules

Priority: 0
Area: Architecture
Status: done
Depends on: define-document-models, bootstrap-desktop-app-shell, implement-canvas-pen-and-viewport, add-canvas-assets-and-backgrounds, add-pdf-viewer-and-navigation, add-pdf-and-canvas-annotation-tools

Goal:
Move the current prototype away from a single large `src/app.rs` implementation and into a clearer module layout that matches the selected plan direction.

Context:
`PLAN.md` now makes maintainability the main priority and explicitly calls for clearer domain, application, service, adapter, and UI boundaries. The repo currently has most user-facing behavior concentrated in `src/app.rs`, which will make OCR fallback, persistence, and future PDF/TTS work harder to extend safely if left as-is.

Expected changes:
- Introduce new source modules and directories under `src/` that separate app shell concerns from canvas behavior, PDF behavior, shared UI state, and reusable helpers.
- Move substantial logic out of `src/app.rs` without changing the user-visible feature set completed in the priority `0-3` band.
- Update `src/lib.rs`, `src/main.rs`, and imports to match the new layout.
- Keep the existing `model` layer aligned with the new structure instead of duplicating state definitions ad hoc.

Acceptance criteria:
- `src/app.rs` no longer acts as the single home for most product behavior.
- The codebase has a clear starting separation between app shell, mode-specific behavior, and shared helpers.
- Existing canvas, PDF, recolor, TTS, and SVG-export behaviors still compile and remain reachable after the refactor.
- A later worker can add OCR fallback or persistence behavior without first re-planning the file layout.

Notes:
This is a structure-first task, not a user-visible feature expansion. Prefer moving real ownership boundaries into code over inventing empty folders.

### extract-mode-shells-and-shared-ui-state

Priority: 1
Area: Ui
Status: done
Depends on: refactor-app-into-modules

Goal:
Separate Infinite Canvas Mode and PDF Mode into clearer UI shells while preserving shared tool state only where it genuinely belongs.

Context:
The plan keeps both modes first-class but distinct. After the initial structural refactor, the next UI-level step is to stop treating mode behavior as one long control flow and instead give each mode clearer ownership over its rendering, commands, and visible tools.

Expected changes:
- Dedicated modules for Canvas Mode UI and PDF Mode UI.
- Clearer ownership of mode-specific commands, toolbars, and view state.
- Shared UI state extracted only for truly shared concerns such as common tool settings, selection state, or global commands.
- Reduced mode-switch branching spread across unrelated functions.

Acceptance criteria:
- Canvas Mode and PDF Mode have clearer code-level boundaries in the UI layer.
- Mode-specific tools and rendering paths are easier to trace without reading one large mixed file.
- Shared UI state is explicit rather than hidden in broad app-level structs.
- The visual distinction between the two modes remains intact after the extraction.

Notes:
Do not force all behavior into shared abstractions. Duplication is acceptable when it keeps the mode mental models cleaner.

### formalize-reading-and-export-services

Priority: 1
Area: Services
Status: done
Depends on: refactor-app-into-modules

Goal:
Introduce explicit service boundaries for PDF reading support, TTS launching, and export decisions so later OCR and persistence work no longer depends on direct ad hoc calls from UI code.

Context:
`PLAN.md` selected a modular-monolith direction with adapter boundaries. The current prototype already performs PDF reading, TTS triggering, and export gating, but those behaviors should be moved behind clearer interfaces before more fallback logic is layered on top.

Expected changes:
- Service traits or equivalent internal interfaces for PDF text access, reading-support fallback decisions, TTS invocation, and export eligibility.
- Adapter modules for the current concrete behavior, including the local speech path already in use.
- Call-site cleanup so UI code requests capabilities instead of owning low-level behavior directly.
- Shared warning/error types where needed for later OCR fallback and export refusal states.

Acceptance criteria:
- Reading-support and export behavior can be invoked through explicit internal boundaries instead of scattered direct calls.
- The current TTS and export behavior still works through the new interfaces.
- OCR fallback can be added later without first untangling UI-owned service logic.
- Export refusal and reading-support warnings have a clearer home than ad hoc UI conditionals.

Notes:
This task is about internal boundaries, not swapping backends yet. Keep the current concrete implementations unless a small change is required to make the interface real.

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

Priority: 0
Area: Ocr
Status: done
Depends on: add-pdf-tts-and-highlight-modes, formalize-reading-and-export-services

Goal:
Add the required fallback order for unreliable PDF text: native text first, OCR second, and a clear user warning when reliable text-based reading support still cannot be produced.

Context:
This is the main risk-control behavior identified by `PLAN.md` and the earlier specification work. The current code already has `ReadingSupportService`, `TextSupportSource`, `ReadingReliability`, `WarningCode`, and `HighlightMode::ManualFallback`, but `start_pdf_tts()` still only attempts native extraction and then stops with a pre-OCR warning. The next worker should extend the existing reading-support boundary instead of pushing more fallback logic directly into the PDF UI flow.

Expected changes:
- Add reliability checks around the existing native PDF extraction path in `src/app/services.rs` and `src/app/pdf.rs`.
- Introduce an OCR-capable fallback path or an explicit placeholder adapter boundary if a concrete OCR backend is still deferred.
- Update `ReadingSupportState` transitions so `text_source`, `reliability`, `warning`, and `tts.active_span` stay internally consistent across native success, OCR fallback success, and total failure.
- Use `HighlightMode::ManualFallback` or equivalent coarse follow-along behavior when precise text-linked highlighting is not possible.
- Keep annotation and basic PDF navigation usable even when reading support degrades.

Acceptance criteria:
- A user on a weak-text PDF triggers TTS/highlighting and the system attempts native text before any OCR path.
- If OCR improves usability enough, the app proceeds with `TextSupportSource::OcrDerivedText` and a visible warning or status that fallback was used.
- If neither native extraction nor OCR yields usable text, the app sets an explicit warning and does not pretend playback/highlighting is reliable.
- Annotation remains available regardless of text support quality.
- `cargo check` passes after the fallback flow is added.

Notes:
This task should focus on correctness of fallback behavior, not perfection of OCR quality. Keep the OCR integration narrow and behind the service boundary so backend changes remain isolated later.

### add-save-load-and-recovery

Priority: 1
Area: Persistence
Status: done
Depends on: define-document-models, add-pdf-and-canvas-annotation-tools, refactor-app-into-modules

Goal:
Implement save, load, autosave, and recovery behavior for editable user work while preserving the project's offline-first and portable-output goals.

Context:
The model layer already includes `AutosaveState`, dirty tracking is already toggled from canvas edits, and the UI currently exposes document dirtiness, but there is no real serialization, reopen flow, or recovery path yet. This task should turn the existing state markers into an actual offline persistence workflow for both canvas and PDF-study state.

Expected changes:
- Define a concrete editable-file format and file locations for canvas and PDF-study sessions.
- Implement save and load behavior for the current document/session models rather than only export behavior.
- Add autosave snapshot writing and interrupted-session recovery handling using the existing `AutosaveState` fields or an expanded equivalent.
- Keep editable working-state persistence separate from user-facing exports such as SVG or recolored PDF output.
- Update any minimal UI controls or status messages needed to make save/load/recovery understandable.

Acceptance criteria:
- A user can save editable work and reopen it in a later app session.
- A user can recover recent unsaved work after an interrupted session using a documented in-app recovery path.
- Recovery/internal state does not replace explicit user export options.
- Core save/load workflows work without internet access.
- `cargo check` passes after the persistence flow is added.

Notes:
Do not let recovery-only storage become the only way users retain their work. Prefer a simple, explicit format and flow over speculative sync or database complexity.

### add-offline-acceptance-checks

Priority: 2
Area: Validation
Status: done
Depends on: refactor-app-into-modules, extract-mode-shells-and-shared-ui-state, formalize-reading-and-export-services, add-text-fallback-and-warning-flow, add-save-load-and-recovery

Goal:
Add repeatable verification for the specification's acceptance checks so future workers can confirm the product remains aligned with the contract.

Context:
The repo now has a working Rust prototype plus multiple worker reports, but verification is still mostly compile-level and memory-driven. This task should convert the plan and existing feature slices into a durable verification layer that covers offline use, weak-PDF behavior, export gating, and persistence.

Expected changes:
- Add test cases, scripted checks, and/or a manual verification checklist mapped to the implemented plan milestones.
- Add or document sample assets for normal PDFs, messy or scanned PDFs, mixed-content canvases, and vector-only selections.
- Record what can be verified automatically with `cargo test`, `cargo check`, or deterministic sample-driven checks versus what still needs manual tablet testing.
- Make the acceptance mapping easy for later workers to rerun without rereading all reports.

Acceptance criteria:
- The project has a documented way to verify each relevant acceptance area in the current plan/spec artifacts.
- Offline operation is explicitly checked.
- SVG-export eligibility and refusal behavior are explicitly checked.
- Weak-PDF fallback and warning behavior are explicitly checked.
- Save/load and recovery behavior are explicitly checked.
- Any manual-only checks are clearly identified instead of being left implicit.

Notes:
Keep the acceptance mapping close to the specification so the contract stays executable. Avoid writing checks that depend on network services.

### run-study-session-ux-hardening

Priority: 3
Area: Ux
Status: current
Depends on: add-offline-acceptance-checks

Goal:
Use real study-session validation to tighten pen feel, warning clarity, tool placement, and mode-specific friction before treating the app as a dependable study tool.

Context:
`PLAN.md` ends with a dedicated UX-hardening phase driven by actual use rather than abstract polish. That phase is currently missing from the backlog. The repo already has the core canvas, PDF, recolor, TTS, export, and pending fallback/persistence work; once verification exists, the remaining high-value task is to run realistic study sessions and turn observed friction into targeted fixes.

Expected changes:
- Run structured manual study-session checks on both Infinite Canvas Mode and PDF Mode using realistic PDFs and annotation flows.
- Capture concrete friction points around pen behavior, reading controls, warnings, export explanations, and switching between reading and note-taking.
- Implement a bounded set of code or UI improvements driven by those findings.
- Write down the remaining known UX issues that should stay deferred.

Acceptance criteria:
- The project has a documented study-session validation pass, not just feature-by-feature spot checks.
- At least one concrete UX issue discovered during real use is fixed or explicitly deferred with rationale.
- Mode boundaries remain clear while reducing friction in normal study flows.
- Any remaining tablet-only or long-session validation gaps are called out explicitly.

Notes:
Keep this phase evidence-driven. Do not invent generic polish work before the acceptance checks and real-use pass expose concrete problems.
