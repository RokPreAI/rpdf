# TODO

## Current TODOs

- [ ] 2 Canvas add-selection-and-move-tools

## Active TODOs

- [ ] 2 Canvas add-selection-and-move-tools
- [ ] 2 Reading add-tts-and-reliability-pipeline
- [ ] 2 Storage add-autosave-and-recovery
- [ ] 2 Export add-svg-export-eligibility
- [ ] 3 PDF add-pdf-page-import-and-recolor
- [ ] 3 Canvas add-draw-shapes
- [ ] 3 UX add-config-and-toolbar-icons

## Done TODOs

- [x] 0 Architecture architecture-foundation-and-pdfium-gate
- [x] 0 Storage versioned-project-and-session-model
- [x] 1 App split-main-into-shell-and-modes
- [x] 1 Canvas add-tablet-pressure-and-stroke-width
- [x] 6 Canvas add-color-picker
- [x] 6 Canvas add-eraser-tool
- [x] 6 Canvas add-image-paste

## Task Details

### architecture-foundation-and-pdfium-gate

Priority: 0
Area: Architecture
Status: done
Depends on: none

Goal:
Lock the durable architecture baseline and the early Pdfium-backed PDF direction into the codebase structure so later work does not drift into an ad hoc prototype.

Context:
This is the first execution task from `PLAN.md`. It combines the robust architecture choice with the fast engine-commitment choice from `DECISION.md`. It also subsumes part of the old README TODO `Split main.ts into multiple files`.

Expected changes:
- define or refine frontend/backend module boundaries
- add the initial Rust PDF engine adapter shape under `src-tauri`
- prepare IPC contracts and shared DTOs for mode switching, PDF rendering, text extraction, save/load, and reliability state
- update Rust dependencies for the chosen Pdfium path

Acceptance criteria:
- The repo has a clear boundary-first structure that separates app shell, canvas, PDF, storage, and reading concerns.
- The backend contains a concrete Pdfium-oriented integration point behind a Rust abstraction.
- The implementation no longer depends on a later “pick a PDF engine” task.
- If a dependency or packaging blocker appears, it is documented in code comments or the task report rather than hidden.

Notes:
- Do not build user-facing PDF features here beyond what is needed to establish the adapter direction.
- Keep the integration replaceable enough that future engine changes are still possible if the chosen path fails.

### versioned-project-and-session-model

Priority: 0
Area: Storage
Status: done
Depends on: architecture-foundation-and-pdfium-gate

Goal:
Create the durable internal document/session model for canvas work, PDF study state, autosave state, and derived cache metadata.

Context:
`PLAN.md` requires a versioned internal persistence model early. This is the robust alternative to rough prototype-only JSON.

Expected changes:
- schema/types for `CanvasDocument`
- schema/types for `PdfStudyDocument`
- version fields and migration-ready structure
- derived cache metadata for OCR/text analysis

Acceptance criteria:
- Canvas and PDF study state can be represented without UI-only fields.
- The internal format contains explicit versioning.
- Derived OCR/text-analysis data is clearly separate from user-authored data.

Notes:
- Keep the model simple, but do not collapse both modes into one fake shared document format.

### split-main-into-shell-and-modes

Priority: 1
Area: App
Status: done
Depends on: architecture-foundation-and-pdfium-gate

Goal:
Refactor the current frontend prototype into an app shell with explicit Canvas Mode and PDF Mode entry points.

Context:
This directly migrates the old README TODO `Split main.ts into multiple files` and supports the product identity requirement that the app has two distinct modes.

Expected changes:
- split `src/main.ts` into app shell and mode-specific modules
- add mode switch and shared app state
- create placeholder tool surfaces for both modes

Acceptance criteria:
- `src/main.ts` is no longer the whole application.
- The app visibly separates Canvas Mode from PDF Mode.
- Shared state and mode-specific behavior are moved into clearer files/modules.

Notes:
- Avoid pulling in a frontend framework unless clearly required.

### build-pdf-mode-shell

Priority: 1
Area: PDF
Status: done
Depends on: split-main-into-shell-and-modes, architecture-foundation-and-pdfium-gate

Goal:
Create the first real PDF workspace with document-focused layout, page navigation scaffolding, and annotation surfaces.

Context:
This migrates the old README TODO `Add pdf mode`, but in the stronger form required by the plan and spec.

Expected changes:
- PDF mode view/controller files
- page navigation UI
- PDF open flow through backend command wiring
- annotation overlay surface

Acceptance criteria:
- The app can enter a dedicated PDF Mode.
- PDF Mode is visually and behaviorally distinct from the canvas.
- The PDF workspace is structured around document reading, not a canvas background metaphor.

Notes:
- This task does not need full TTS or OCR yet.

### add-tablet-pressure-and-stroke-width

Priority: 1
Area: Canvas
Status: done
Depends on: split-main-into-shell-and-modes

Goal:
Upgrade drawing input so pen pressure affects stroke behavior and stroke width becomes a controllable part of the drawing model.

Context:
This migrates the README TODOs `Add drawing tablet supportb with preassure sensitivity.` and `Add stroke thickness`. The plan and spec both treat pressure-sensitive drawing as core.

Expected changes:
- pointer-pressure aware stroke logic
- stroke-width settings in tool state or settings
- data-model support for variable stroke width

Acceptance criteria:
- Pressure-sensitive input changes stroke output on supported devices.
- Stroke thickness can be controlled even when pressure is unavailable.
- Mouse input still works without pretending to be a pressure device.

Notes:
- Validate behavior on real hardware if available, or document what could not be tested.

### add-save-load-project-files

Priority: 1
Area: Storage
Status: done
Depends on: versioned-project-and-session-model

Goal:
Add explicit save/load behavior for canvas projects and PDF study sessions using the internal versioned format.

Context:
This migrates the old README TODO `Add saving to file`.

Expected changes:
- frontend save/open actions
- Rust commands for file IO
- serialization/deserialization for the internal models

Acceptance criteria:
- A canvas project can be saved and reopened.
- A PDF study session can be saved and reopened.
- Save/load flows use the versioned internal model rather than UI snapshots.

Notes:
- Do not postpone this too long; later autosave and recovery depend on it.

### add-selection-and-move-tools

Priority: 2
Area: Canvas
Status: pending
Depends on: split-main-into-shell-and-modes

Goal:
Add bounded selection behavior plus movement of selected items in Canvas Mode.

Context:
This migrates the README TODOs `Add selection tool` and `Add a move tool that moves selected items`.

Expected changes:
- selection model and hit testing
- move interaction for selected items
- selection state wired into export eligibility

Acceptance criteria:
- Users can select canvas items intentionally.
- Selected items can be moved without corrupting document state.
- Selection state can later be reused for export targeting.

Notes:
- Keep the first version simple and reliable rather than feature-rich.

### add-tts-and-reliability-pipeline

Priority: 2
Area: Reading
Status: pending
Depends on: build-pdf-mode-shell, architecture-foundation-and-pdfium-gate

Goal:
Add native text extraction, TTS playback, explicit reliability states, OCR fallback entry points, and honest trust-state UI.

Context:
This migrates the old README TODO `Add TTS`, but the plan requires a full reading-support pipeline rather than a raw speak-text button.

Expected changes:
- backend text extraction commands
- reliability-state model
- TTS controls and playback state
- OCR trigger path
- trust-state warnings and fallback behavior

Acceptance criteria:
- The app can read extracted PDF text aloud locally.
- The UI distinguishes native reliable, native weak, OCR-derived, and unavailable states.
- The app follows the required fallback order: native text, OCR, warning.
- The app does not pretend exact follow-along exists when confidence is weak.

Notes:
- Math-aware reading is deferred; do not block this task on advanced math interpretation.

### add-autosave-and-recovery

Priority: 2
Area: Storage
Status: pending
Depends on: add-save-load-project-files

Goal:
Protect local work with autosave snapshots and recovery prompts after interruption.

Context:
This migrates the old README TODO `Add autosaving` and follows the robust foundation choice from `DECISION.md`.

Expected changes:
- autosave scheduling/triggers
- app-local snapshot storage
- restart recovery detection and prompt flow

Acceptance criteria:
- Open work is autosaved locally.
- Recovery is offered after an interrupted session when newer autosave data exists.
- Recovery data is separate enough that failed caches do not destroy authored work.

Notes:
- Prefer simple dependable behavior over aggressive background automation.

### add-svg-export-eligibility

Priority: 2
Area: Export
Status: pending
Depends on: add-selection-and-move-tools

Goal:
Implement selection-aware SVG export with strict compatibility checks.

Context:
This migrates the old README TODO `Add svg export`, but it must obey the spec rule that SVG export is only valid for compatible vector/text content.

Expected changes:
- export eligibility checks
- UI messaging for unavailable SVG export
- SVG generation for compatible selections or whole documents where valid

Acceptance criteria:
- SVG export works for vector/text-only compatible targets.
- SVG export is disabled or clearly blocked for mixed raster/PDF targets.
- Export targeting respects current selection when one exists.

Notes:
- Do not claim full-canvas SVG support when PDF pages or raster images are present.

### add-pdf-page-import-and-recolor

Priority: 3
Area: PDF
Status: pending
Depends on: build-pdf-mode-shell, architecture-foundation-and-pdfium-gate

Goal:
Support importing rasterized PDF pages into the canvas and apply recoloring in both PDF Mode and imported-page workflows.

Context:
This task comes from the plan rather than the README, but it is central to the product identity.

Expected changes:
- backend PDF page raster generation for canvas import
- canvas-side PDF page placement support
- recolor controls in PDF Mode
- recolor support for imported PDF page items

Acceptance criteria:
- PDF pages can be imported into Canvas Mode as placed visual items.
- Recoloring works in PDF Mode.
- Imported PDF page items can reflect recolor settings per page or per selection path.

Notes:
- Keep the first recolor system simple and testable.

### add-draw-shapes

Priority: 3
Area: Canvas
Status: pending
Depends on: add-selection-and-move-tools

Goal:
Add basic shape drawing in Canvas Mode.

Context:
This directly migrates the old README TODO `Add draw shapes`.

Expected changes:
- tool state for shapes
- shape rendering/storage in the canvas model
- shape selection compatibility

Acceptance criteria:
- Users can create a bounded set of basic shapes.
- Shapes persist correctly in saved documents.
- Shape content participates correctly in selection and export eligibility checks.

Notes:
- Keep the first shape set minimal.

### add-config-and-toolbar-icons

Priority: 3
Area: UX
Status: pending
Depends on: split-main-into-shell-and-modes

Goal:
Add a simple configuration surface and toolbar icon system for the current tools and settings.

Context:
This migrates the old README TODOs `Add config file` and `Add icons to the toolbar`. It is lower priority than architecture, storage, and reading support.

Expected changes:
- config/settings persistence path
- toolbar icon assets or icon mapping
- settings for pen/tool defaults and likely recolor defaults

Acceptance criteria:
- Core user settings can be stored and reloaded.
- Toolbar actions are visually identifiable without relying on text only.
- The config surface does not undermine the minimal UI direction.

Notes:
- Keep this small; do not turn it into a large preferences subsystem yet.

### add-color-picker

Priority: 6
Area: Canvas
Status: done
Depends on: none

Goal:
Preserve the earlier completed color-picker work from `README.md`.

Context:
This was listed under `Done TODOs` in the current README and should remain visible in the migrated backlog.

Expected changes:
- none required unless later verification shows drift

Acceptance criteria:
- The completed color-picker work remains recorded in `Done TODOs`.

Notes:
- If later regressions appear, create a new corrective task instead of rewriting this done record.

### add-eraser-tool

Priority: 6
Area: Canvas
Status: done
Depends on: none

Goal:
Preserve the earlier completed eraser-tool work from `README.md`.

Context:
This was listed under `Done TODOs` in the current README and should remain visible in the migrated backlog.

Expected changes:
- none required unless later verification shows drift

Acceptance criteria:
- The completed eraser-tool work remains recorded in `Done TODOs`.

Notes:
- If later regressions appear, create a new corrective task instead of rewriting this done record.

### add-image-paste

Priority: 6
Area: Canvas
Status: done
Depends on: none

Goal:
Preserve the earlier completed image-paste work from `README.md`.

Context:
This was listed under `Done TODOs` in the current README and should remain visible in the migrated backlog.

Expected changes:
- none required unless later verification shows drift

Acceptance criteria:
- The completed image-paste work remains recorded in `Done TODOs`.

Notes:
- If later regressions appear, create a new corrective task instead of rewriting this done record.
