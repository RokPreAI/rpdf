# PLAN

## Summary

`rpdf` will be built as an offline-first Tauri desktop application with Rust as the native backend core and TypeScript as the frontend. The plan follows a robust architecture bias: clear subsystem boundaries, durable internal document/session models, explicit reliability states, and recovery-aware persistence. At the same time, the plan accepts one major speed-oriented decision early: commit now to a Pdfium-style Rust-integrated PDF path instead of delaying the PDF engine choice.

This is intentionally a hybrid plan, not a pure copy of any single source plan. The main structure comes from `plans/PLANROBUST.md`, while the concrete early PDF direction comes from `plans/PLANFAST.md` and `plans/PLANSIMPLE.md`. UX-first material from `plans/PLANUX.md` remains a secondary influence, especially for mode clarity, pen-first behavior, and trust-state communication.

## Objectives

The plan must deliver a desktop study tool that satisfies the current `SPEC.md` with the following execution priorities:

1. preserve the required Tauri + Rust platform direction
2. support two first-class modes: Infinite Canvas Mode and PDF Mode
3. keep the product offline-first for core workflows
4. establish a durable foundation for annotations, reading reliability, export, save, and recovery
5. make an early concrete PDF engine commitment so implementation can proceed without a prolonged architecture gate
6. remain honest about text extraction, OCR fallback, and follow-along reliability

## Constraints And Assumptions

### Hard constraints

- The system must be a Tauri desktop application.
- Rust must own the native/system side of the application.
- The core workflows must remain usable offline.
- PDF Mode and Canvas Mode must remain distinct.
- The fallback order for reading support must remain:
  1. native PDF text
  2. OCR fallback
  3. clear warning if still unreliable

### Planning assumptions

- The first target is a personal offline desktop tool, not a collaboration platform.
- A Pdfium-style Rust-integrated PDF path is the assumed implementation direction unless later blocked.
- A versioned internal format for working state is acceptable and desirable, even if user-facing outputs remain portable.
- Math-aware speech is a future extension point, not a first-version completion gate.
- Linux-first validation is acceptable if needed to stabilize the early stack, but the product is not being defined as Linux-only.

### Deferred choices

- The exact low-level Pdfium integration method is not fixed yet.
- The precise internal file schema versions and migration rules are not finalized yet.
- The specific form of later math-aware reading enhancement remains open.

## Recommended Stack

### Required platform stack

- Tauri desktop shell
- Rust backend core
- TypeScript frontend
- Browser Pointer Events and canvas-based interaction for pressure-sensitive pen input

### Recommended core libraries and integrations

- PDF engine: Pdfium via a Rust binding such as `pdfium-render`
- OCR fallback: Tesseract, invoked only after page rasterization and only when needed
- TTS output: local/system TTS via a Rust-facing adapter

### Why this stack was chosen

- Tauri and Rust are required by the specification.
- The decision record explicitly chose a faster early PDF-engine commitment instead of a later engine gate.
- Pdfium is the pragmatic fit carried forward from the fast/simple plans.
- The robust plan still requires engine integration to sit behind a stable Rust interface so that the rest of the architecture is not tightly coupled to one engine implementation.

### Dependency stance

Dependencies are accepted when they materially improve capability or reduce product risk, but they must remain bounded and auditable. This plan is not trying to minimize dependencies at all cost, but it also should not accumulate large libraries without clear responsibility boundaries.

## Architecture

## 1. System shape

Build `rpdf` as a layered modular monolith inside one Tauri desktop application.

Primary layers:

1. Frontend presentation layer
2. Application service layer
3. Domain layer
4. Infrastructure layer

This is not a microservice split. The goal is one local application with internal boundaries strong enough to preserve maintainability and future replaceability.

## 2. Two first-class mode models

Do not force Canvas Mode and PDF Mode into one fake shared document model.

Use separate domain models:

- `CanvasDocument`
  - infinite spatial workspace
  - vector strokes
  - typed text
  - raster images
  - imported PDF page placements
  - background pattern configuration
- `PdfStudyDocument`
  - source PDF identity and metadata
  - page navigation state
  - annotation state
  - recolor settings
  - TTS session state
  - text reliability state per page or region

Mode switching belongs in the application layer, not in a merged persistence format.

## 3. Reading-support pipeline

Treat PDF reading support as a structured pipeline with explicit provenance.

For each PDF or page, the system should model:

1. native text extraction attempted
2. native text reliability assessed
3. OCR attempted if needed
4. OCR reliability assessed
5. follow-along alignment confidence determined
6. user-visible warning state produced

This pipeline should produce an intermediate reading representation that records:

- text blocks/spans/regions
- reading-order confidence
- source kind: native or OCR
- confidence metadata
- optional future hooks for math fragments

This keeps future Speech Rule Engine or MathCAT style integration possible without forcing it into the first build.

## 4. Annotation ownership model

Visible marks should not be owned directly by renderer-specific primitives.

Use an internal annotation model that can represent:

- freehand strokes
- highlights
- text notes
- mode-specific annotation settings
- normal-view and recolored-view appearance mappings

Then translate that model into:

- PDF display overlays
- annotated PDF export behavior
- canvas rendering behavior

This keeps export and display decoupled from the engine details.

## 5. Persistence and recovery model

Use a versioned internal persistence format for working state, autosave, and recovery.

Persist separately:

- canvas working documents
- PDF study session state
- autosave snapshots
- export artifacts
- derived OCR/text-analysis caches

Derived caches must be invalidatable and rebuildable. User-authored work must not depend on caches for recovery.

## 6. Failure and trust surfaces

Failure states are part of the product architecture.

Model and surface explicit states for:

- missing reliable text
- OCR available but weak
- follow-along not trustworthy
- SVG export unavailable for selected content
- recolor export limitations
- optional subsystem unavailable on this machine

These should be enforced in application services and reflected in the UI. The app must not hide uncertainty behind vague behavior.

## Project Structure

The repo should evolve toward a boundary-first structure, while staying compatible with the current Tauri + TypeScript shape.

Suggested direction:

```text
src/
  app/
    shell/
    navigation/
    state/
    trust-state/
  features/
    canvas/
      ui/
      interactions/
      document-model/
      export/
    pdf/
      ui/
      interactions/
      viewer/
      tts/
      follow-along/
      recolor/
      trust/
    shared/
      warnings/
      settings/
      commands/
  platform/
    tauri/
    ipc/

src-tauri/
  src/
    app/
      services/
      commands/
    domain/
      canvas/
      pdf/
      annotations/
      export/
      reading/
      recovery/
    infrastructure/
      pdf_engine/
      ocr/
      tts/
      storage/
      export/
    contracts/
      dto/
      errors/
    tests/
      integration/
      fixtures/
```

Rules for this structure:

- frontend feature code should not embed PDF parsing logic
- infrastructure code should not own business rules
- cross-layer communication should use explicit DTOs/contracts
- domain models should remain serializable without UI dependencies

## Implementation Phases

## Phase 0: Foundation and early engine commitment

Goal: establish the durable architecture baseline while resolving the chosen early PDF direction.

Work:

1. define subsystem boundaries and IPC contracts
2. define internal persistence/versioning strategy
3. formalize the annotation domain model
4. formalize the reading reliability model and warning taxonomy
5. integrate the chosen Pdfium-style Rust path behind a PDF engine adapter
6. validate the chosen engine against:
   - rendering needs
   - text extraction needs
   - rasterization for OCR
   - annotation/export feasibility
   - packaging constraints

Exit criterion:

- the architecture boundaries are explicit
- the Pdfium-based direction is concretely wired into the backend behind an adapter boundary
- the plan no longer depends on a later engine-selection process

## Phase 1: Durable storage and document models

Goal: create the internal models and storage path that later features depend on.

Work:

1. implement `CanvasDocument` and `PdfStudyDocument` schemas
2. implement project/session save and load behavior
3. implement autosave snapshot support
4. implement recovery metadata and restart detection
5. establish rebuildable derived caches for OCR/text-analysis results

Exit criterion:

- working state is versioned, reloadable, and recoverable
- user-authored data is protected independently of derived caches

## Phase 2: App shell and mode separation

Goal: make the product structurally correct before deep feature completion.

Work:

1. build the Tauri app shell
2. create explicit entry points for Canvas Mode and PDF Mode
3. add shared navigation and document-open flows
4. add shared settings for pen behavior, annotation colors, recoloring presets, and trust-state presentation
5. create placeholder mode-specific tool surfaces so the app already feels like two distinct workspaces

Exit criterion:

- the app visibly behaves like a two-mode product, not one generic workspace

## Phase 3: Canvas Mode first-class baseline

Goal: deliver a durable, pen-first canvas workflow on top of the new foundation.

Work:

1. implement infinite canvas viewport and pan/zoom behavior
2. implement pressure-sensitive drawing using Pointer Events
3. add typed text items
4. add image import
5. add required background patterns: dots, lines, squares
6. add selection and movement
7. add export-eligibility checks for selection versus whole-canvas export
8. support PDF page import into canvas as backend-generated raster placements

Exit criterion:

- Canvas Mode supports the required personal note-taking workflow
- SVG export is honest and selection-aware

## Phase 4: PDF Mode reading and annotation baseline

Goal: deliver a document-focused PDF workflow instead of a canvas-like approximation.

Work:

1. open PDFs through Rust backend commands
2. render pages via the Pdfium-backed adapter
3. implement page navigation
4. implement PDF annotation overlays using the internal annotation model
5. keep overlay state in `PdfStudyDocument` instead of mutating the source PDF model directly
6. add visible mode-specific tools and layout

Exit criterion:

- PDF Mode is document-focused, navigable, and annotatable
- the mode does not behave like the infinite canvas

## Phase 5: TTS, reliability states, and OCR fallback

Goal: satisfy the core reading-support requirements without pretending hard problems are solved.

Work:

1. extract native text spans and coarse geometry through the backend
2. assess native-text reliability
3. wire local/system TTS output
4. implement follow-along behavior only at the level supported by current confidence
5. add OCR fallback via page rasterization plus Tesseract
6. distinguish clearly between:
   - native text reliable
   - native text weak
   - OCR-derived text
   - unreliable or unavailable text support
7. show visible fallback reading aids and warnings when exact synchronization is not trustworthy

Exit criterion:

- the app follows the required `native -> OCR -> warning` order
- trust-state behavior is explicit and not misleading

## Phase 6: Recoloring and export

Goal: finish the core output and reading-comfort behaviors required by the spec.

Work:

1. implement PDF recoloring presets and custom foreground/background control
2. apply recoloring in PDF Mode
3. apply recoloring to imported PDF snapshots in Canvas Mode
4. support separate annotation appearance mappings for normal and recolored viewing
5. export annotated PDF as flattened output
6. export recolored PDF as an explicit user-selected result
7. keep SVG export restricted to compatible vector/text-only targets

Exit criterion:

- recoloring works for study use and optional output
- annotation visibility remains usable in both viewing contexts

## Phase 7: Hardening and trust cleanup

Goal: make the tool survivable for real personal use.

Work:

1. unify warning surfaces across modes
2. validate autosave and recovery behavior under interruption
3. smoke-test fully offline usage
4. validate export gating and error messaging
5. fix the highest-risk failures in persistence, PDF behavior, and trust-state presentation

Exit criterion:

- the app degrades honestly
- local work is not easily lost
- failure states are visible and understandable

## Task Order

1. formalize architecture boundaries and IPC contracts
2. commit the Pdfium-backed adapter path
3. implement document/session schemas and save/load behavior
4. add autosave and recovery primitives
5. build the app shell and explicit mode switch
6. implement Canvas Mode baseline
7. implement PDF Mode baseline
8. implement native text extraction and system TTS
9. add reliability assessment and OCR fallback
10. add recoloring
11. add annotated/recolored PDF export
12. tighten SVG export rules
13. run hardening and trust-surface cleanup

## Risks And Difficulties

### 1. Pdfium integration and packaging

The plan explicitly accepts an early engine commitment, so Pdfium bundling, adapter shape, and platform packaging are real schedule risks. This is the main speed-oriented tradeoff preserved from the decision record.

### 2. Annotation/export correctness

The hardest long-term correctness problem is not drawing strokes, but making sure visible annotations, stored annotation state, and exported PDF results stay meaningfully aligned.

### 3. Text reliability classification

The specification requires honesty. A weak confidence model can overpromise or hide useful capability. Reliability classification and user-visible wording are both product-critical.

### 4. OCR reliability and operational setup

Tesseract fallback is useful but fragile. It adds both operational complexity and the risk of low-confidence output that must still be explained clearly.

### 5. Follow-along precision on real PDFs

Complex layouts, messy structure, scanned pages, and math-heavy documents will make perfect synchronization impossible in many cases. The app must optimize for trustworthy fallback behavior, not false precision.

### 6. Robustness versus momentum

The user explicitly chose robustness at the architecture and foundation levels. That choice should be preserved, but it can still become counterproductive if it blocks a usable offline build for too long.

### 7. UX drift despite correct structure

The UX-first plan was not selected as the primary planning stance, but its warning remains valid: the app can become structurally sound yet unpleasant or unclear to use. Pen-first ergonomics, mode clarity, and trust surfaces must stay visible during implementation.

## Open Questions

1. What exact Pdfium binding and packaging approach should be used in this repo?
2. Should the first platform-targeting pass be Linux-first for operational simplicity, or should multi-platform packaging be exercised earlier?
3. How should annotated PDF export be validated for correctness against the internal annotation model?
4. What heuristics should determine the boundary between `native_reliable`, `native_weak`, `ocr_weak`, and `unavailable`?
5. Which specific UX surfaces from `PLANUX.md` should be pulled in early without changing the robust architecture bias?

## Decision Traceability

This plan preserves the selected tradeoffs from `DECISION.md`:

- robust architecture and durable internal models are primary
- the PDF engine decision is not deferred and is assumed to be a Pdfium-style Rust-integrated path
- UX-first guidance is retained only where it strengthens mode clarity, pen-first behavior, and trust-state communication
- dependency minimization from the lean plan is not the main direction, but license and packaging risks remain active review concerns
