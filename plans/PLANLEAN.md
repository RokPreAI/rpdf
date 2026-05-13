# Plan Summary

| Field                   | Value |
| ----------------------- | ----- |
| Optimization target     | Lowest dependency approach |
| What it optimizes for   | Minimal third-party surface, low license risk, simple packaging, offline reliability, and clear control over native components |
| What it sacrifices      | Fast feature delivery, polished cross-platform integrations, rich PDF editing depth, and early advanced math/TTS capability |
| Proposed stack          | Tauri + Rust backend + TypeScript frontend + browser canvas/SVG + system TTS + external system tools for OCR/PDF text where needed |
| Architecture shape      | Thin frontend, Rust-first core, adapter boundaries around optional system integrations, separate Canvas and PDF modes with minimal shared abstractions |
| Major risks             | PDF capability gaps without heavy libraries, uneven platform support, weaker V1 highlighting/TTS accuracy, more custom implementation work |
| Estimated build speed   | Moderate to slow |
| Dependency profile      | Very low direct dependencies; prefer standard platform capabilities and narrowly scoped crates only when unavoidable |
| Performance profile     | Good baseline resource use; some operations slower if routed through simple single-threaded or external-tool paths |
| User-experience profile | Honest, minimal, functional, but less feature-rich and less forgiving in difficult PDF/TTS cases |

## Optimization stance

This plan deliberately minimizes third-party libraries, license exposure, bundled native engines, and packaging complexity.

It prefers:

- Tauri because it is required
- Rust ownership of file and document workflows
- browser-native pointer and rendering features for pen input
- system-provided TTS and OCR/PDF utilities where possible instead of bundling large engines
- strict separation between core features and optional integrations

It intentionally avoids a “best available library for every problem” approach.

## What this plan refuses to optimize for

- best-in-class PDF engine breadth in V1
- full cross-platform consistency at the start
- early math-aware speech sophistication
- rich dependency-heavy editor frameworks
- aggressive OCR and document-understanding pipelines bundled into the app

## Architecture

### Core principle

Build the smallest credible offline desktop study tool that satisfies the specification through a narrow, auditable architecture.

### Mode split

Keep the product as two explicit modes from day one:

1. **Canvas Mode**
   - freeform drawing, text, image import, PDF page placement
   - implemented mostly in the webview with custom rendering and document model
2. **PDF Mode**
   - document reading, navigation, annotation overlay, TTS, recoloring, follow-along
   - implemented as a dedicated viewer pipeline, not as “canvas with PDF background”

### Dependency-minimizing design choices

- Use **custom canvas/SVG rendering** instead of large editor frameworks.
- Use **system TTS** through a thin Rust adapter instead of bundling a speech engine.
- Prefer **system Poppler utilities** such as `pdftotext` and `pdftoppm` on Linux-first targets rather than bundling Pdfium or taking MuPDF license risk.
- Treat **OCR as optional system integration** via external tools if present, instead of shipping Tesseract in the first lean build.
- Store annotations in an **app-owned sidecar model** first, then export annotated/recolored PDFs through a Rust export pipeline later.

### PDF strategy

This plan chooses the lowest packaging-risk route:

- **Linux-first V1**
- rely on **system-installed Poppler command-line tools** for text extraction and page rasterization
- do not bundle Pdfium
- do not use MuPDF due to AGPL/commercial risk
- do not promise native editable PDF annotation internals in V1

This is intentionally conservative. It reduces dependency lock-in and licensing risk, but shifts more work into app-side coordination and may limit portability early.

### Data model

Use simple app-owned formats:

- canvas documents: JSON + referenced assets
- PDF study state: JSON sidecar per document for annotations, recolor preferences, reading metadata, confidence/warning state
- autosave/recovery snapshots: internal JSON/state files

User-facing exports remain portable, but internal editing stays in simple app-controlled data.

## Tools

### Required tools

- **Tauri** for desktop shell
- **Rust** for backend commands, filesystem, export orchestration, and system tool integration
- **TypeScript** for UI state and interaction
- **HTML/CSS/SVG/Canvas APIs** for rendering and pen behavior

### Preferred system integrations

- **system TTS backend** via Rust `tts`-style adapter or direct platform invocation if needed
- **Poppler CLI tools** when available for PDF text extraction and rasterization
- **optional OCR tool invocation** only if installed locally and explicitly detected

### Tools to avoid initially

- heavy frontend drawing/editor libraries
- bundled Pdfium runtime
- MuPDF because of license complexity
- bundled OCR engine
- cloud APIs
- complex database layers

## Development stack

### Frontend

- TypeScript
- small custom UI components
- browser Pointer Events for pressure-sensitive pen input
- SVG for vector shapes/text where practical
- Canvas for high-frequency stroke rendering if needed

### Backend

- Rust commands exposed through Tauri
- simple serialization for project state
- filesystem-first asset management
- shell-out adapters for system PDF/OCR tooling behind strict interfaces

### Packaging target

- Start **Linux-first** to keep system dependency assumptions realistic for Poppler and speech backends.
- Only expand to other platforms after the adapter boundaries are stable.

## Project structure

- `src/`
  - frontend app shell, mode routing, state, rendering, tool UI
- `src/canvas/`
  - canvas document model, tools, selection, export eligibility logic
- `src/pdf/`
  - PDF mode UI, navigation, overlay annotation model, recolor controls, reading-state UI
- `src/shared/`
  - common types for colors, geometry, documents, warnings
- `src-tauri/`
  - Rust command handlers and app lifecycle
- `src-tauri/src/pdf/`
  - PDF text extraction, page rasterization, export orchestration, system tool adapters
- `src-tauri/src/tts/`
  - local TTS adapter and playback control
- `src-tauri/src/storage/`
  - save/load, autosave, recovery, asset paths
- `src-tauri/src/ocr/`
  - OCR availability detection and invocation wrapper
- `plans/`
  - planning artifacts only

## Implementation phases

### Phase 1: Establish the dependency-light foundation

1. Create Tauri app shell and file-based workspace model.
2. Define document/state formats for:
   - canvas documents
   - PDF sidecar state
   - autosave/recovery
3. Build Linux-first Rust adapters for:
   - checking `pdftotext` availability
   - checking page rasterization tool availability
   - checking local TTS availability
   - checking optional OCR availability

Outcome: a thin, testable core with no heavy bundled engines.

### Phase 2: Canvas Mode first, fully app-owned

1. Implement infinite canvas viewport and pan/zoom.
2. Add pen drawing with pressure via Pointer Events.
3. Add typed text items.
4. Add image placement.
5. Add background patterns: dots, lines, squares, scaling with zoom.
6. Add selection and movement.
7. Add export eligibility logic:
   - whole canvas vs selection
   - SVG only for vector/text-compatible selections

Outcome: strong canvas workflow without external rendering libraries.

### Phase 3: Minimal PDF Mode using system PDF tooling

1. Open PDF and rasterize pages for viewing.
2. Add document navigation.
3. Add annotation overlay model on top of page views.
4. Add simple text notes and highlights in overlay state.
5. Add recolored viewing as a visual transform on rendered pages.
6. Add import of PDF pages into canvas as placed visual content.

Outcome: real PDF mode exists without bundling a full PDF engine.

### Phase 4: Reading support with honest confidence tiers

1. Extract native text through system PDF text tools.
2. Build confidence states:
   - reliable native text
   - weak extracted text
   - no usable text
3. Connect local/system TTS to extracted text.
4. Add follow-along only at coarse granularity first, likely line/block level.
5. If no reliable text exists, show clear warnings.
6. If optional OCR tool exists, add OCR fallback behind explicit status messaging.

Outcome: specification-aligned fallback behavior with minimal bundled dependencies.

### Phase 5: Save/export/recovery

1. Explicit save/load for canvas and PDF sidecar state.
2. Autosave and recovery snapshots.
3. Export annotated PDFs through a Rust-managed composition path.
4. Add optional recolored PDF export if output quality is acceptable.
5. Finalize SVG export rules for vector-only targets.

Outcome: portable outputs where valid, without pretending unsupported exports work.

### Phase 6: Targeted hardening

1. Test messy PDFs, scanned PDFs, and multi-column documents.
2. Verify visible warning behavior.
3. Measure resource use and simplify hot paths.
4. Only then evaluate whether a bundled PDF engine is actually necessary.

Outcome: defer heavy dependency decisions until real evidence demands them.

## Task order

1. **Scaffold the Tauri + Rust + TypeScript app foundation**
   - Changes: establish app shell, frontend/backend wiring, file-open/save commands
   - Acceptance: app launches and can create/load empty workspace state

2. **Define internal file/state formats**
   - Changes: create Rust and TypeScript shared schemas for canvas docs, PDF sidecars, autosave
   - Acceptance: state round-trips without lossy conversion

3. **Implement system capability detection**
   - Changes: add Rust adapters for TTS, PDF text extraction, PDF rasterization, optional OCR presence
   - Acceptance: app reports capability matrix and missing-tool states clearly

4. **Build Canvas Mode core interaction**
   - Changes: pan/zoom, pressure-sensitive drawing, text items, image items, patterns
   - Acceptance: stylus-first canvas works offline with low latency

5. **Add canvas selection and SVG eligibility logic**
   - Changes: selection model, mixed-content export checks, SVG export for valid selections
   - Acceptance: vector-only selections export; mixed incompatible targets are blocked honestly

6. **Build PDF page viewing pipeline**
   - Changes: page rasterization, paging UI, zoom, basic caching
   - Acceptance: PDFs open and pages render reliably through system tooling

7. **Add PDF annotation overlay**
   - Changes: pen markup, highlights, text notes stored in sidecar state
   - Acceptance: annotations save/reload and remain editable in app state

8. **Add recolored viewing and imported-page handling**
   - Changes: recolor controls in PDF mode and canvas-import workflow for PDF pages
   - Acceptance: user can recolor document view and imported pages visually

9. **Implement native-text reading pipeline**
   - Changes: text extraction, segmentation, confidence labeling, TTS playback
   - Acceptance: text PDFs can be read aloud with visible follow-along when confidence is sufficient

10. **Add warning-first fallback behavior**
   - Changes: OCR path only if local tool exists, weak-text warnings, coarse fallback highlighting
   - Acceptance: poor PDFs degrade honestly and visibly

11. **Implement save/export/recovery flows**
   - Changes: autosave, recovery, annotated PDF export, optional recolor export
   - Acceptance: work survives restarts and user-facing exports are generated where supported

12. **Harden against real technical PDFs**
   - Changes: test and refine extraction, UI messaging, resource use
   - Acceptance: app behavior stays understandable on messy and scanned inputs

## Dependencies

- Phase 1 is required before all others.
- Phase 2 can proceed before deep PDF work and should be prioritized.
- Phase 3 depends on system PDF tool adapters from Phase 1.
- Phase 4 depends on Phase 3 page/text pipeline and system TTS detection.
- Phase 5 depends on stable internal data models from Phases 2 and 3.
- Phase 6 depends on all earlier phases and should inform whether heavier dependencies are ever justified.

## Risks

### 1. System-tool dependence may reduce portability

A low-dependency app can still depend on external system tools. That reduces bundled complexity but may create uneven setup across machines.

### 2. Linux-first bias may delay broader desktop support

This plan keeps packaging simpler by assuming Linux-first realities for Poppler and speech backends.

### 3. PDF export may be harder than viewing

Viewing through rasterized pages is much simpler than producing high-quality annotated and recolored PDF exports.

### 4. Honest fallback behavior may feel less magical

Because this plan avoids heavy bundled document-intelligence stacks, the app will need to say “not reliable enough” more often.

### 5. More custom code replaces third-party convenience

Minimizing libraries does not minimize work. It moves effort into app-owned implementations.

## Likely difficulties

- mapping annotation overlays back into exportable PDF outputs cleanly
- maintaining acceptable zoom/render performance without a richer PDF engine
- segmenting extracted text into useful read-aloud units for highlighting
- handling scanned PDFs gracefully when OCR is optional rather than bundled
- keeping TTS controls consistent across local backends
- preserving a minimal UI while exposing capability/warning states clearly

## Why this plan is intentionally biased

This is not the fastest or most feature-complete route. It is the route that keeps control local, dependencies sparse, and licensing exposure low.

If the project later proves that stronger PDF or math-reading capabilities truly require a bundled engine or larger subsystem, those should be added only after this lean architecture demonstrates where the real gaps are.