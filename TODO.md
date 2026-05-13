# TODO

## Current TODOs

- [ ] 1 PDF fix-pdf-mode-annotations
- [ ] 2 Canvas add-element-resize
- [ ] 2 Canvas add-input-polling-rate-setting
- [ ] 2 Export harden-svg-export-and-save-path
- [ ] 2 PDF add-recent-pdf-quick-open-list
- [ ] 2 UX harden-responsive-layout-under-window-resize
- [ ] 3 UX add-excalidraw-style-tool-and-color-shortcuts

## Done TODOs

- [x] 0 Architecture architecture-foundation-and-pdfium-gate
- [x] 0 Storage versioned-project-and-session-model
- [x] 1 App preserve-mode-state-across-switches
- [x] 1 App split-main-into-shell-and-modes
- [x] 1 Canvas add-tablet-pressure-and-stroke-width
- [x] 1 Canvas fix-select-tool
- [x] 1 Canvas fix-select-tool
- [x] 1 PDF contain-pdf-within-viewport
- [x] 1 PDF contain-pdf-within-viewport
- [x] 1 Reading fix-read-page-action
- [x] 2 Reading add-tts-and-reliability-pipeline
- [x] 2 UX fix-recolor-controls-layout-and-state
- [x] 3 Canvas add-draw-shapes
- [x] 3 PDF add-pdf-page-import-and-recolor
- [x] 3 UX add-config-and-toolbar-icons
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
Status: done
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
Status: done
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
Status: done
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
Status: done
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
Status: done
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
Status: done
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
Status: done
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

### fix-select-tool

Priority: 1
Area: Canvas
Status: done
Depends on: add-selection-and-move-tools, add-draw-shapes, add-pdf-page-import-and-recolor

Goal:
Fix the current selection workflow so item picking and post-selection behavior are dependable across the elements the app already supports.

Context:
The first selection implementation exists, but the user explicitly called out that the select tool still needs work. This is now corrective follow-up work on a previously completed baseline task.

Expected changes:

- repair hit testing and selection activation where it is currently inconsistent
- verify selection behavior for strokes, shapes, pasted images, and imported PDF pages
- make the selected-state feedback and interaction flow stable enough for later resize work

Acceptance criteria:

- The select tool can intentionally select each currently supported selectable element type.
- Selection does not get lost unexpectedly during normal click, drag, or mode-switch workflows.
- Selection behavior is stable enough to serve as the base for resize handles or equivalent resize interactions.

Notes:

- Treat this as a reliability fix, not a redesign of the entire interaction model.

### contain-pdf-within-viewport

Priority: 1
Area: PDF
Status: done
Depends on: build-pdf-mode-shell, add-pdf-page-import-and-recolor

Goal:
Keep PDF content contained within the visible application viewport instead of allowing it to spill outside the usable window area.

Context:
The user reported that the PDF can go out of the application viewport. This is a direct usability regression in PDF Mode.

Expected changes:

- constrain PDF rendering/layout to the available viewport
- fix overflow or sizing logic that lets the PDF escape the visible workspace
- verify behavior across normal window sizes and reduced window sizes

Acceptance criteria:

- PDF content stays within the visible PDF workspace bounds.
- Shrinking the window does not cause the PDF surface to escape the app viewport.
- Any intentional scrolling or zoom behavior remains usable and predictable.

Notes:

- This needs real human verification at runtime after implementation; do not treat a code-only pass as sufficient.

### fix-pdf-mode-annotations

Priority: 1
Area: PDF
Status: done
Depends on: build-pdf-mode-shell

Goal:
Make PDF Mode show its annotation layer or explicitly restore the currently missing annotation visibility behavior.

Context:
The user reported that they do not see annotations in PDF Mode. That means either the annotation surface is absent, hidden, or not wired to visible state.

Expected changes:

- inspect PDF Mode annotation rendering and visibility
- restore annotation drawing or overlay mounting
- verify annotations remain visible against current recolor and PDF rendering paths

Acceptance criteria:

- Annotation visuals appear in PDF Mode when expected.
- Annotation visibility does not silently break when the PDF view updates or recolor settings change.
- The annotation layer is clearly anchored to the PDF workspace rather than hidden behind it.

Notes:

- This is a product-critical PDF-mode behavior gap, not optional polish.

### fix-read-page-action

Priority: 1
Area: Reading
Status: done
Depends on: add-tts-and-reliability-pipeline, build-pdf-mode-shell

Goal:
Fix the `Read page` action so it produces an observable reading result instead of appearing to do nothing.

Context:
The user reported that pressing `Read page` does nothing. This could be a command wiring failure, a state-gating issue, or a silent error path.

Expected changes:

- trace the `Read page` action from UI event to backend call to playback state
- surface failures instead of allowing a silent no-op
- restore functional TTS playback for supported readable pages

Acceptance criteria:

- Pressing `Read page` triggers observable behavior on supported pages.
- If reading cannot start, the UI shows a clear reason instead of failing silently.
- The action remains aligned with the reliability-state model already built for reading support.

Notes:

- This task requires runtime debugging and human verification; the agent should not mark it complete from static inspection alone.

### preserve-mode-state-across-switches

Priority: 1
Area: App
Status: done
Depends on: split-main-into-shell-and-modes, add-save-load-project-files, build-pdf-mode-shell

Goal:
Preserve the active canvas and PDF workspace state when switching between modes instead of resetting the canvas and unloading the current PDF document.

Context:
The user reported that switching modes causes the canvas to become blank and the PDF to unload, which forces re-entry of the PDF path and breaks normal workflow continuity.

Expected changes:

- preserve in-memory state for both Canvas Mode and PDF Mode across mode switches
- prevent mode switching from clearing the active canvas document unexpectedly
- prevent mode switching from unloading the active PDF document and its working session state

Acceptance criteria:

- Switching from Canvas Mode to PDF Mode and back does not blank the canvas unexpectedly.
- Switching away from PDF Mode and back does not require retyping the current PDF path.
- Core per-mode session state survives normal mode switches until the user explicitly opens, loads, clears, or replaces it.

Notes:

- This is a workflow-continuity fix, not a request for full background persistence beyond the existing save/autosave model.

### add-recent-pdf-quick-open-list

Priority: 2
Area: PDF
Status: current
Depends on: preserve-mode-state-across-switches, add-save-load-project-files, build-pdf-mode-shell

Goal:
Add a quick-open list of the 5 most recent PDF paths inside the PDF workspace sidebar so users can reopen common documents without retyping long paths.

Context:
The user asked for 5 recent PDF paths as a quick-open option inside `#workspace-root > section > aside > div:nth-child(1)` after repeatedly losing the current PDF path on mode switches.

Expected changes:

- track and persist the 5 most recent opened PDF paths
- render a quick-open list in the first PDF sidebar section
- allow one-click reopening of recent PDF entries

Acceptance criteria:

- The PDF sidebar shows up to 5 recent PDF paths in the requested quick-open area.
- Recent entries update when a new PDF is opened and maintain a sensible most-recent-first order.
- Clicking a recent entry reopens that PDF without manual path re-entry.

Notes:

- Keep the first version path-based and practical; do not expand this into a full library browser.

### add-input-polling-rate-setting

Priority: 2
Area: Canvas
Status: current
Depends on: add-tablet-pressure-and-stroke-width

Goal:
Add a user-adjustable input polling or sampling-rate setting to improve line quality when strokes look jagged from undersampled pointer input.

Context:
The user reported that lines can end up a bit jagged, and suggested exposing a polling-rate setting. This is a drawing-quality control task rather than a pure visual styling issue.

Expected changes:

- add a configurable input polling or sampling control for drawing
- wire the setting into stroke capture so higher-quality sampling is possible where supported
- preserve sensible defaults so the app still behaves well without manual tuning

Acceptance criteria:

- Users can adjust the relevant drawing-input sampling setting from the UI or persisted config surface.
- Changing the setting has a real effect on stroke capture behavior or smoothing quality.
- The setting does not break mouse drawing, pressure-sensitive drawing, or saved preference loading.

Notes:

- If the real bottleneck is event interpolation or smoothing rather than literal polling rate, the implementation may use a differently named internal mechanism as long as the user-facing control honestly addresses the jagged-line problem.

### fix-recolor-controls-layout-and-state

Priority: 2
Area: UX
Status: current
Depends on: add-pdf-page-import-and-recolor

Goal:
Fix the recolor controls so their active indicators match the selected color and the controls are laid out horizontally instead of vertically stacked.

Context:
The recolor algorithm itself works, but the user reported two UX defects: the selected-state indicators do not track the chosen color, and the controls are stacked vertically when they should be horizontal.

Expected changes:

- repair active-state updates for recolor selections
- change recolor-control layout from vertical stacking to horizontal layout
- verify the indicator styling stays correct after repeated recolor changes

Acceptance criteria:

- The active recolor indicator matches the current selected recolor option.
- Recolor controls are arranged horizontally in a usable compact row or wrapped horizontal layout.
- Layout and indicator state remain stable after switching documents, pages, or recolor modes.

Notes:

- Keep this tightly scoped to recolor controls; do not reopen unrelated toolbar layout work here.

### harden-responsive-layout-under-window-resize

Priority: 2
Area: UX
Status: current
Depends on: contain-pdf-within-viewport, split-main-into-shell-and-modes

Goal:
Stabilize app layout and item presentation when the window is reduced in size so the interface does not get visibly messed up.

Context:
The user reported that reducing the window size breaks the item layout badly enough that it requires human intervention to diagnose; they explicitly said the agent cannot resolve this one purely on its own.

Expected changes:

- identify layout breakpoints and resizing failure paths in both shell and mode workspaces
- fix the most disruptive responsive layout issues for reduced window sizes
- leave any remaining hard-to-automate visual issues clearly documented for manual review

Acceptance criteria:

- Reducing the window no longer causes major layout corruption in the main interactive areas.
- Core controls remain reachable and legible at smaller window sizes.
- Any residual issues that still require human design judgment are explicitly documented instead of being silently ignored.

Notes:

- This is explicitly a human-verified task. The implementation can be assisted by the agent, but completion requires manual runtime review.

### add-element-resize

Priority: 2
Area: Canvas
Status: current
Depends on: fix-select-tool

Goal:
Allow selected canvas elements to be resized after they are created.

Context:
The app can already create and move several item types, but the user now wants direct resizing support as part of the core editing workflow.

Expected changes:

- resize interaction model for selected items
- visual affordances for resizeable selections
- document/model updates for resized shapes, images, and imported PDF page items

Acceptance criteria:

- Users can resize supported selected elements intentionally and predictably.
- Resizing updates persisted document state correctly.
- Resize behavior does not corrupt item position, selection state, or export eligibility logic.

Notes:

- Strokes may need a narrower first version than box-like items; document any intentional first-pass limits clearly in code or report artifacts.

### add-excalidraw-style-tool-and-color-shortcuts

Priority: 3
Area: UX
Status: current
Depends on: split-main-into-shell-and-modes, add-color-picker, add-draw-shapes

Goal:
Add fast keyboard shortcuts for tool switching and color switching, following the general workflow style users expect from Excalidraw-like canvas apps.

Context:
The user explicitly asked for Excalidraw-style shortcuts. This should improve flow without introducing a heavy command system.

Expected changes:

- keyboard shortcuts for the main canvas tools
- keyboard shortcuts for the existing color palette
- in-app discoverability for the added shortcuts

Acceptance criteria:

- Main tools can be switched from the keyboard without opening settings.
- The current color can be changed from the keyboard using a simple memorable mapping.
- Shortcuts do not break text inputs or other normal focused-control behavior.

Notes:

- Match the spirit of Excalidraw-style access patterns even if exact key choices need to adapt to the current tool set.

### harden-svg-export-and-save-path

Priority: 2
Area: Export
Status: current
Depends on: add-svg-export-eligibility, fix-select-tool

Goal:
Verify the current SVG export flow end to end and make sure users can intentionally choose where exported SVG files are saved.

Context:
The user specifically asked to test SVG export and select save path. The current implementation needs validation and likely a clearer export destination flow.

Expected changes:

- test the current SVG export behavior against real selections and eligible content
- fix any SVG export regressions uncovered by that testing
- add or verify explicit save-path selection for exported SVG output

Acceptance criteria:

- SVG export is confirmed to work for supported export targets.
- Export failure cases remain honest and clear for ineligible mixed-content selections.
- Users can choose the SVG export destination instead of relying on an unclear or fixed save location.

Notes:

- Keep this task scoped to SVG export reliability and destination control, not a full generic export framework.

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
