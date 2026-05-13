# Plan Summary

| Field                   | Value |
| ----------------------- | ----- |
| Optimization target     | Fastest possible offline desktop prototype |
| What it optimizes for   | Reaching a working Tauri app quickly with both modes, pen input, direct PDF markup, basic local TTS, and honest fallback states |
| What it sacrifices      | Clean architecture, cross-platform polish, deep math handling, annotation fidelity, long-term maintainability, and likely some future rewrites |
| Proposed stack          | Tauri + Rust backend + TypeScript frontend + browser canvas + Pdfium via `pdfium-render` + system TTS + Tesseract fallback |
| Architecture shape      | Thin vertical slices with minimal shared abstractions and feature-specific pipelines |
| Major risks             | Pdfium bundling, OCR reliability, weak PDF text order, rough PDF annotation export path, and unstable early data models |
| Estimated build speed   | Fast if Linux-first and scope is enforced |
| Dependency profile      | Moderate to high, chosen for speed over purity |
| Performance profile     | Good enough for a prototype, not optimized |
| User-experience profile | Functional and direct, but intentionally rough around edge cases |

## Philosophy

This plan is intentionally biased toward the fastest route to a convincing prototype. The goal is to prove the product shape quickly: separate canvas mode and PDF mode, tablet-first drawing, PDF markup, local read-aloud, and visible reliability fallback behavior. Anything that slows delivery without being required for that demo is deferred, simplified, or faked with temporary implementation choices.

What this plan optimizes for:
- first working prototype speed
- visible end-to-end behavior over internal elegance
- reuse of existing engines and system services
- quick validation of the two-mode workflow

What this plan sacrifices:
- clean persistence design
- portable long-term file formats beyond required outputs
- native-quality PDF annotation internals
- strong math-aware speech in V1
- refined UX, broad platform consistency, and future-proof abstractions

## Architecture

### Overall shape
Build one Tauri desktop app with two strongly separated frontend workspaces backed by a small Rust command layer:

1. **Canvas Mode**
   - browser canvas/Web UI for pressure-sensitive drawing
   - local document state in JSON-like app data
   - imports images and rendered PDF pages as placed items
   - SVG export only for clearly compatible selections

2. **PDF Mode**
   - PDF page viewer driven by Rust-side rendering through Pdfium
   - pen overlay for page annotations stored separately first
   - local TTS pipeline based on extracted text
   - fallback status shown per document or page: native text / OCR / unreliable

### Fast-prototype principles
- Keep canvas annotations and PDF annotations as app-managed overlay data first; do not aim for a perfect native editable PDF annotation model in V1.
- Use raster page rendering in the viewer and draw annotation overlays on top.
- Export annotated PDFs by composing rendered pages plus overlays rather than preserving sophisticated PDF internals.
- Treat math-aware reading as future work; in V1, deliver basic TTS with honest reliability states.
- Keep OCR on demand, not automatic for every file.

## Tools And Development Stack

### Required platform
- **Desktop shell:** Tauri
- **Native layer:** Rust
- **Frontend:** TypeScript
- **UI approach:** minimal custom UI with browser canvas and simple controls

### Core libraries and services
- **PDF engine:** `pdfium-render`
  - chosen because research says it fits Rust/Tauri pragmatically and already covers rendering, extraction, and annotations
  - accept bundling work and thread-safety constraints
- **TTS output:** Rust `tts` crate using system/local backend
  - chosen because audio synthesis is not the hard part
- **OCR fallback:** Tesseract invoked only after PDF page rasterization
  - only on demand or when extraction clearly fails
- **Pen input:** browser Pointer Events with `pressure`
- **Canvas export:** SVG string generation from vector/text selections only

### Non-goals for prototype stack
Do not add heavy experimental math parsing or semantic equation reconstruction in the first prototype. Do not chase lowest-dependency purity. Do not switch engines mid-prototype unless blocked.

## Project Structure

Use a simple split by mode and backend capability, not a polished domain architecture.

- `src/`
  - `app/`
    - app shell, mode switching, shared status UI
  - `canvas/`
    - canvas view
    - stroke tools
    - selection
    - item model for text/image/pdf-page placements
    - SVG export logic
  - `pdf/`
    - PDF viewer
    - page navigation
    - annotation overlay
    - read-aloud controls
    - fallback status UI
  - `shared/`
    - file dialogs
    - settings types
    - persistence helpers
- `src-tauri/`
  - `pdf/`
    - open/render/extract text
    - OCR trigger
    - export annotated PDF
  - `tts/`
    - system TTS wrapper
  - `files/`
    - save/load/autosave helpers
  - `commands/`
    - Tauri command handlers exposed to frontend

Keep only the minimum shared code needed. Duplicate some logic if that is faster than premature abstraction.

## Implementation Phases

### Phase 1: Skeleton and mode split
Goal: get a running Tauri app that visibly has two separate modes.

Tasks:
1. Create app shell with mode switch: Canvas / PDF.
2. Add minimal file menu/actions: new canvas, open PDF, save, export.
3. Add shared document status area for warnings and save state.

Deliverable:
- app launches offline
- mode separation is explicit

### Phase 2: Canvas mode first-pass
Goal: rapidly prove tablet-first note-taking.

Tasks:
1. Implement infinite-ish pannable/zoomable canvas.
2. Add pressure-sensitive freehand drawing using Pointer Events.
3. Add highlighter tool as a stroke style variant.
4. Add typed text placement.
5. Add image import.
6. Add scalable dot/line/square backgrounds.
7. Save canvas document in simple internal JSON format.

Shortcuts:
- use one straightforward scene model
- postpone advanced editing behaviors
- accept rough selection/transform UX initially

Deliverable:
- user can draw, write text, place images, and save/reopen

### Phase 3: PDF mode first-pass
Goal: make PDF reading and markup work fast.

Tasks:
1. Open PDF through Rust command.
2. Render pages to images/bitmaps via Pdfium.
3. Show one page at a time or a simple scroll list.
4. Add page navigation.
5. Add pen annotation overlay per page.
6. Store overlay strokes separately from original PDF.

Shortcuts:
- no complex native PDF annotation editing model
- no deep search/indexing layer
- prioritize visible direct markup over PDF purity

Deliverable:
- user can open a PDF and annotate pages directly in app

### Phase 4: Basic TTS plus visible reliability states
Goal: satisfy core reading-support prototype requirement without pretending hard problems are solved.

Tasks:
1. Extract native text from PDF pages.
2. Add a crude text-quality heuristic:
   - native text usable
   - native text weak, suggest OCR
   - text unreliable
3. Add read-aloud controls: play, pause, stop.
4. Feed extracted text to system TTS.
5. Add simple follow-along highlighting only when extraction maps cleanly enough.
6. If mapping is weak, show a visible fallback reading aid and warning instead of precise highlight.
7. Add OCR path for selected page/document when native text is weak.
8. Mark OCR-derived reading state clearly.

Shortcuts:
- no promise of sentence-perfect synchronization
- basic line/block highlighting is enough
- OCR can be manual-triggered to reduce complexity

Deliverable:
- app can read some PDFs aloud locally
- app visibly communicates native text vs OCR vs unreliable state

### Phase 5: Import/export required proof points
Goal: hit the specification-shaped outputs quickly.

Tasks:
1. Import PDF pages into canvas as placed visual content.
2. Represent imported PDF pages in canvas as rendered page assets plus source reference.
3. Add recolor toggle for PDF mode viewing.
4. Add simple recolor options for imported PDF page items in canvas.
5. Export annotated PDF by flattening rendered pages plus overlays into output PDF.
6. Export recolored PDF as a user-selected variant.
7. Export SVG only when current selection contains vector strokes and typed text only.
8. Disable SVG export for mixed raster/PDF selections with clear explanation.

Shortcuts:
- flatten export is acceptable for prototype
- recoloring can be visual-transform-based first
- imported PDF pages in canvas do not need deep live linkage

Deliverable:
- required prototype export stories are demonstrable

### Phase 6: Autosave, warnings, and hardening pass
Goal: make the prototype survivable for real personal use.

Tasks:
1. Add autosave for open canvas and PDF annotation sessions.
2. Add recovery prompt after interruption.
3. Surface warnings consistently:
   - weak native text
   - OCR in use
   - OCR unreliable
   - SVG unavailable for current export target
4. Smoke-test fully offline behavior.
5. Fix highest-friction bugs only.

Deliverable:
- prototype fails honestly and does not lose work easily

## Task Order

1. Tauri app shell and explicit mode switch
2. Canvas drawing with pressure
3. Canvas save/load and backgrounds
4. PDF open/render/navigation
5. PDF page annotation overlay
6. Native text extraction and basic TTS
7. Reliability states and fallback UI
8. OCR trigger path
9. PDF page import into canvas
10. Recoloring in PDF mode and canvas imports
11. Export flows: annotated PDF, recolored PDF, SVG
12. Autosave/recovery/warning cleanup

## Dependencies

- Phase 1 is required before everything else.
- Canvas export depends on canvas item model from Phase 2.
- PDF annotation depends on page rendering from Phase 3.
- TTS depends on PDF text extraction from Phase 4.
- OCR depends on rasterized page access from Phase 3 and fallback UI from Phase 4.
- Annotated PDF export depends on overlay persistence from Phase 3.
- Recolored export depends on basic recolor rendering path from Phase 5.
- Autosave depends on initial save/load model in both modes.

## Risks

### Highest risks accepted by this plan
1. **Pdfium integration risk**
   - bundling and runtime linking may cost time up front
   - accepted because changing engines later is still cheaper than over-planning now

2. **Annotation export quality risk**
   - flattening overlays may not preserve rich editability
   - accepted because the prototype only needs exported annotated PDFs, not perfect PDF-native semantics

3. **Reading-order reliability risk**
   - technical PDFs may break highlighting badly
   - mitigated by visible warning states and coarse fallback behavior

4. **OCR disappointment risk**
   - Tesseract output may be weak on difficult pages
   - mitigated by manual trigger, labeling OCR as weaker, and refusing fake precision

5. **Math reading gap**
   - generic TTS will not read math well enough in many cases
   - explicitly accepted for prototype; the app should demonstrate the pipeline and honesty model, not solve full math speech yet

6. **Data model rewrite risk**
   - quick internal save structures may need later replacement
   - accepted deliberately

## Likely Difficulties

- mapping extracted PDF text back to visible page regions well enough for follow-along highlighting
- keeping per-page overlay coordinates aligned across zoom and resize
- generating annotated/recolored PDF output fast without a full native annotation model
- making OCR invocation reliable on target machines
- handling messy technical PDFs without the UI feeling broken
- preserving low-resource feel once OCR or multi-page rendering starts

## Fast Decisions To Lock Early

1. **Prototype target OS**: prefer Linux-first if needed to reduce backend variance.
2. **PDF engine**: commit to Pdfium early and do not reopen the decision during prototype work unless blocked.
3. **Annotation model**: overlays first, native PDF semantics later.
4. **TTS scope**: local system TTS only; no advanced math speech in first prototype.
5. **OCR policy**: on-demand fallback, clearly labeled.
6. **Export policy**: flatten where necessary; honesty over sophistication.

## Definition Of Success For This Plan

The plan succeeds if a single-user offline prototype can do all of the following quickly:
- switch between a real canvas mode and a real PDF mode
- draw with pressure-sensitive pen input
- annotate a PDF directly in app
- read extracted PDF text aloud through local TTS
- show visible fallback/warning behavior when text quality is weak
- import PDF pages into the canvas
- export annotated PDFs and compatible SVG selections

If the prototype works but contains obvious technical debt, rough UX, flattened exports, and incomplete math handling, this plan still counts that as success.