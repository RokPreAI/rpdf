# Plan Summary

| Field                   | Value |
| ----------------------- | ----- |
| Optimization target     | Most robust long-term architecture |
| What it optimizes for   | Clear subsystem boundaries, durable file and data models, honest reliability states, testability, and safe future extension for PDF, OCR, TTS, and math-aware reading |
| What it sacrifices      | Fast delivery, low upfront complexity, minimal dependency count, and short-path prototyping convenience |
| Proposed stack          | Tauri desktop app, Rust backend core, TypeScript frontend, browser canvas for pen input, Rust PDF integration layer, local OCR pipeline, system/local TTS bridge, structured document-understanding pipeline |
| Architecture shape      | Layered modular monolith with explicit domain boundaries and versioned internal document models |
| Major risks             | PDF engine fit, annotation/export correctness, OCR reliability, math-reading complexity, cross-platform desktop integration complexity |
| Estimated build speed   | Slow |
| Dependency profile      | Moderate to high, chosen deliberately for stability and clear responsibility boundaries |
| Performance profile     | Good enough but not aggressively optimized early; correctness and recoverability first |
| User-experience profile | Predictable, trustworthy, and explicit about capability limits rather than aggressively streamlined |

## Robustness Bias

This plan intentionally optimizes for maintainability, resilience, testability, recoverability, and future extension. It assumes that the hardest long-term problem is not drawing strokes, but building a trustworthy desktop study tool that can survive poor PDFs, evolving math-reading support, annotation/export edge cases, and future format changes without collapsing into ad hoc code.

This plan intentionally sacrifices speed to first release. It accepts more modules, more explicit internal models, more validation logic, and more test infrastructure than a fast prototype would use.

## Architecture

### 1. System shape: layered modular monolith

Build `rpdf` as one Tauri application with a strict internal boundary structure instead of a loosely organized feature pile.

Primary layers:

1. **Frontend presentation layer**
   - Tablet-first UI
   - Canvas rendering and interaction
   - PDF reading/annotation UI
   - Capability/warning surfaces
2. **Application service layer**
   - Commands, workflow orchestration, autosave triggers, document lifecycle
3. **Domain layer**
   - Canvas document model
   - PDF session model
   - annotation model
   - export eligibility model
   - text reliability/confidence model
4. **Infrastructure layer**
   - PDF engine adapter
   - OCR adapter
   - TTS adapter
   - filesystem/project storage
   - serialization and recovery

This should stay a modular monolith at first, not microservices, but every major subsystem should be replaceable behind stable Rust interfaces.

### 2. Two first-class modes with separate internal models

Do not fake a shared model for canvas mode and PDF mode.

Create separate domain models:

- **CanvasDocument**
  - infinite spatial workspace
  - vector strokes
  - typed text
  - raster images
  - imported PDF page placements
  - background pattern configuration
- **PdfStudyDocument**
  - source PDF identity and metadata
  - page navigation state
  - annotation state
  - recolor settings
  - TTS session state
  - text reliability state per page/region

Keep mode switching at the application layer, not by forcing both modes into one persistence structure.

### 3. Explicit document-understanding pipeline

Treat reading support as a pipeline with recorded provenance.

Per PDF/page, model these states explicitly:

1. native text extraction attempted
2. native text reliability assessed
3. OCR attempted if needed
4. OCR reliability assessed
5. follow-along alignment confidence determined
6. user-visible warning state produced

Do not allow UI features to infer reliability implicitly.

Create a structured intermediate representation for reading support:

- document/page text blocks
- reading order confidence
- span/line/region anchors
- source kind: native vs OCR
- confidence metadata
- optional math fragments for future processing

This intermediate layer is the key extensibility point for later Speech Rule Engine / MathCAT style math support.

### 4. Annotation ownership model

Avoid coupling visible marks directly to renderer-specific primitives.

Use an internal annotation domain model that represents:

- freehand strokes
- highlights
- text notes
- per-mode color settings
- normal-view and recolored-view appearance mappings

Then add infrastructure translators for:

- PDF display overlay
- annotated PDF export
- canvas rendering

This reduces future rewrite cost if PDF engine or export implementation changes.

### 5. Durable persistence and recovery model

Use a versioned internal persistence format for working state and autosave/recovery, even if user-facing outputs stay portable.

Persist separately:

- canvas working documents
- PDF study session state
- autosave snapshots
- export artifacts
- derived OCR/text-analysis caches

Derived caches must be invalidatable and rebuildable. User-authored content must never depend on caches for recovery.

### 6. Honest failure surfaces as architecture, not decoration

Failure states are core product behavior.

Build explicit state and UI contracts for:

- missing reliable text
- OCR available but weak
- follow-along not trustworthy
- SVG export unavailable for selected content
- recolor export unsupported for a given workflow path
- optional subsystem unavailable offline or on this machine

This is required by the spec and should be enforced in application services and tests.

## Tools And Development Stack

### Desktop shell and frontend

- **Tauri** for desktop runtime
- **Rust** for native/backend core
- **TypeScript** for frontend
- **Browser canvas / Pointer Events** for pen input and pressure sensitivity

### Core backend subsystems

- **PDF engine abstraction in Rust** with an adapter implementation chosen only after a license/capability review
- **OCR pipeline adapter** behind a Rust trait
- **System/local TTS adapter** behind a Rust trait
- **Serialization layer** for internal working-state files and autosave

### Recommended dependency posture

Choose dependencies for stability and subsystem fit, not minimal count.

Preferred decision rule:

1. clear licensing
2. strong maintenance story
3. good Rust integration
4. testability/mocking feasibility
5. only then convenience

Because research flagged licensing as a first-order risk, PDF engine selection must be treated as an architecture gate, not a late implementation detail.

## Project Structure

Use a boundary-first repository layout.

```text
src/
  app/
    commands/
    state/
    workflows/
  features/
    canvas/
      ui/
      interactions/
      view-models/
    pdf/
      ui/
      interactions/
      view-models/
    shared/
      warnings/
      settings/
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
      events/
      errors/
    tests/
      integration/
      fixtures/
```

Key rules:

- frontend feature code should not embed PDF parsing logic
- infrastructure code should not own business rules
- cross-layer communication should use explicit DTOs/contracts
- domain models should be serializable without UI dependencies

## Implementation Phases

### Phase 0: Architecture and feasibility gates

1. define subsystem boundaries and IPC contracts
2. choose persistence/versioning strategy
3. evaluate PDF engine candidates against:
   - licensing
   - rendering
   - text extraction
   - annotation support
   - export behavior
4. define reading reliability model and warning taxonomy
5. define internal annotation model

Exit criterion: architecture decisions are written down and the PDF engine choice is justified.

### Phase 1: Durable document and storage foundation

1. implement internal document schemas
2. implement filesystem project/session storage
3. implement autosave and crash recovery skeleton
4. implement fixture-based serialization tests
5. implement migration/version hooks for future schema changes

Exit criterion: canvas and PDF session state can be saved, recovered, and version-checked safely.

### Phase 2: Canvas mode as a stable subsystem

1. implement pressure-sensitive stroke model
2. implement infinite canvas viewport/navigation architecture
3. implement typed text items
4. implement image import
5. implement PDF page placement abstraction
6. implement scalable background patterns
7. implement selection model
8. implement export eligibility checks for SVG

Exit criterion: canvas documents are structurally sound and export rules are enforced honestly.

### Phase 3: PDF mode core with annotation integrity

1. implement PDF open/render/navigation service
2. implement PDF session model wiring
3. implement annotation overlays using internal annotation model
4. implement direct markup workflows
5. implement recolor viewing pipeline
6. implement export path for annotated PDFs
7. validate behavior on normal, messy, and scanned PDFs

Exit criterion: PDF mode is usable and annotations remain dependable across reopen/export cycles.

### Phase 4: Reading pipeline without math ambition creep

1. implement native text extraction path
2. implement reliability assessment heuristics/state assignment
3. implement OCR fallback path
4. implement page/region text cache
5. implement warning states for weak results
6. implement follow-along anchors only when confidence allows
7. implement visible fallback reading aid when exact sync is not trustworthy

Exit criterion: the app can read aloud locally, surface reliability honestly, and avoid false precision.

### Phase 5: TTS integration and configurable follow-along

1. integrate local/system TTS output layer
2. implement playback session state machine
3. implement word/line/sentence modes only where alignment supports them
4. connect UI configuration for highlight style
5. test interruption, pause, resume, page changes, and degraded input cases

Exit criterion: TTS is operational, stateful, and reliable about what it can and cannot sync.

### Phase 6: Extensibility hooks for future math-aware reading

1. formalize intermediate reading representation for math fragments
2. mark extension points for structured math speech engines
3. isolate math processing behind its own adapter boundary
4. add fixtures from math-heavy documents to measure future progress

Exit criterion: future math support can be added without rewriting the PDF or TTS core.

## Task Order

1. Finalize architecture boundaries and contracts.
2. Decide PDF engine after license/capability review.
3. Implement internal persistence, autosave, and recovery.
4. Implement shared annotation domain model.
5. Build stable canvas subsystem.
6. Build stable PDF mode rendering/navigation/annotation subsystem.
7. Build export rules and export pipelines.
8. Build reading reliability pipeline.
9. Integrate local TTS and follow-along state machine.
10. Add future math extension points and fixture coverage.
11. Harden with failure-case integration tests.

This order is intentionally conservative: storage and domain correctness come before feature polish.

## Testing Strategy

Robustness depends on tests that match the hard cases named in the spec and research.

### Required test layers

- **Domain unit tests**
  - annotation rules
  - export eligibility rules
  - reliability state transitions
  - save/recovery semantics
- **Adapter tests**
  - PDF engine wrappers
  - OCR wrappers
  - TTS wrappers
- **Integration tests**
  - open PDF → annotate → save → reopen → export
  - poor text → OCR fallback → warning state
  - mixed canvas selection → SVG unavailable / available as appropriate
  - recolored viewing with annotation color profiles
- **Fixture-based regression tests**
  - clean text PDF
  - multi-column technical PDF
  - scanned PDF
  - math-heavy PDF
  - mixed-content canvas document

### Failure cases to lock down

- no text layer
- unusable OCR
- annotation colors invisible under recolor settings
- autosave interrupted mid-write
- imported PDF pages on canvas making full SVG export invalid
- follow-along confidence too low for exact highlighting

## Risks

### 1. PDF engine choice may force painful reversal

Mitigation:
- choose before deep implementation
- build adapter boundary first
- validate annotation/export scenarios with fixtures early

### 2. Long-term architecture may slow visible progress too much

Mitigation:
- require phase exits with demonstrable vertical slices
- keep modular monolith, not distributed complexity

### 3. Reading reliability heuristics may be underspecified

Mitigation:
- encode explicit confidence states early
- test against real troublesome fixture PDFs

### 4. Math-aware reading may pressure the design into premature complexity

Mitigation:
- keep math support as a future adapter point
- do not entangle V1 TTS with speculative equation understanding

### 5. Cross-platform stylus/TTS behavior may drift

Mitigation:
- isolate platform adapters
- document support expectations per platform
- keep Linux-first validation acceptable if scope must narrow early

## Dependencies

- PDF engine decision blocks robust PDF mode, annotation export, text extraction, and OCR rasterization details.
- Persistence/versioning design blocks autosave, recovery, and cross-feature save semantics.
- Annotation domain model blocks both canvas and PDF annotation implementations.
- Reading reliability model blocks follow-along UI and warning behavior.
- OCR adapter depends on rasterization path chosen with the PDF engine.
- Future math subsystem depends on the intermediate reading representation.

## Likely Difficulties

1. Keeping PDF mode genuinely document-native while still sharing annotation concepts with canvas mode.
2. Designing export behavior that stays honest for mixed-content canvases.
3. Avoiding duplicated state between frontend UI and Rust domain models.
4. Defining reliability heuristics that are useful without pretending certainty.
5. Preserving low-friction pen interaction while using a stricter architecture.
6. Choosing a PDF stack that does not create licensing or packaging pain later.
7. Building recovery/autosave that never corrupts user-authored work.

## Why This Plan Is Intentionally Not Balanced

This plan does not optimize for the quickest route to a usable demo. It front-loads decisions, internal models, test infrastructure, and subsystem boundaries because the project’s hardest requirements are long-term ones: dependable annotation, offline resilience, honest degraded behavior, and extensibility toward better technical-document reading. A simpler or faster plan would likely ship earlier, but it would increase the risk of major architectural rewrites once PDF export, OCR reliability, and math-aware reading start to matter.