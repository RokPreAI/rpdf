# Plan Summary

| Field                   | Value |
| ----------------------- | ----- |
| Optimization target     | Simplest possible correct Tauri implementation |
| What it optimizes for   | Shipping one understandable offline desktop app with both required modes using the fewest subsystems and the least custom infrastructure |
| What it sacrifices      | Feature depth, polished PDF editing fidelity, rich math handling, broad platform ambition, and sophisticated document intelligence in v1 |
| Proposed stack          | Tauri 2, Rust, TypeScript, Vite, plain HTML/CSS, Canvas 2D, Pdfium via Rust binding, system TTS, Tesseract OCR as optional fallback command |
| Architecture shape      | Thin frontend with two screens, Rust backend for PDF/file work, one shared local project format, raster-first PDF display and overlay annotations |
| Major risks             | Pdfium integration/bundling, OCR setup complexity, weak text-order reliability, pressure behavior differences across desktops |
| Estimated build speed   | Fast relative to other options |
| Dependency profile      | Low to moderate; prefer one PDF engine, system TTS, and no frontend framework |
| Performance profile     | Good enough for personal use, not heavily optimized |
| User-experience profile | Minimal and direct, but intentionally sparse |

## Planning stance

This plan intentionally cuts toward the smallest implementation that still satisfies the specification. It avoids clever architecture, avoids a rich component framework, avoids deep PDF semantics in v1, and avoids promising strong math understanding early. The goal is a correct Tauri app with two real modes, offline behavior, pen input, PDF annotation, basic read-aloud with honest fallback states, recoloring, and selection-aware export rules.

## What this plan deliberately does

- Keeps canvas mode and PDF mode as separate screens, not a unified model.
- Uses plain Canvas 2D for drawing instead of a scene graph or custom editor framework.
- Uses raster rendering for displayed PDF pages in both PDF mode and canvas-import mode.
- Stores annotations as app-side overlay data first, then exports flattened annotated PDFs.
- Treats math-aware reading as deferred; v1 only improves honesty and fallback behavior.
- Supports highlighting modes only when the extracted text supports them reliably.

## What this plan deliberately does not optimize for

- Editable native PDF annotations inside the source PDF model.
- Perfect text synchronization on complex layouts.
- Deep semantic math speech in v1.
- Fancy canvas object editing.
- Cross-platform parity beyond what Tauri, web pointer events, and system TTS already provide.

## Architecture

### 1. Frontend shape

Use the existing plain TypeScript frontend in `src/main.ts` and keep a single-window app.

Two top-level modes:

- **Canvas Mode**: infinite drawing canvas with image import, PDF page import as raster images, typed text blocks, simple selection, and export.
- **PDF Mode**: PDF page viewer with page navigation, pen/highlighter/text-note overlays, TTS controls, follow-along overlay, and recolor controls.

Keep mode state in one app store module rather than adopting a framework.

### 2. Backend shape

Use Rust in `src-tauri/src/lib.rs` for all native tasks:

- open/save project files
- open PDF files
- render PDF pages to images
- extract text spans for a page
- assess text reliability
- call OCR fallback when needed
- export annotated PDF
- export recolored PDF
- autosave/recovery file operations

### 3. Data model

Use one internal project file for editable work and keep user-facing exports portable.

Suggested internal structures:

- canvas document with strokes, text blocks, placed images, placed imported PDF-page snapshots
- PDF session document with source PDF path, per-page overlay annotations, recolor settings, and optional cached text/OCR metadata

This matches the spec: internal format is acceptable for autosave/recovery, while user-facing outputs remain PDF/SVG where possible.

## Tools and development stack

### Frontend

- `src/main.ts` - main app bootstrapping and mode switching
- `src/styles.css` - minimal UI styling
- HTML Canvas 2D for drawing and PDF page display overlays
- native DOM controls instead of a UI framework

### Backend

- Tauri 2
- Rust
- Pdfium Rust binding for rendering, text extraction, and PDF export work
- system TTS via backend or frontend bridge, preferring whatever is simplest to invoke reliably on desktop
- Tesseract only as an explicit OCR fallback path

### Why this stack is the simple choice

- Tauri and Rust are required by spec.
- Plain TypeScript avoids framework overhead.
- Canvas 2D already fits the existing repo direction.
- One PDF engine avoids split responsibilities.
- System TTS avoids shipping a speech engine.

## Project structure

Keep the repo shape simple and grow only where necessary.

### Existing files to extend

- `src/main.ts` - split current canvas prototype into app shell, canvas mode, PDF mode, input tools, and export triggers
- `src/styles.css` - add two-mode layout and minimal controls
- `src-tauri/src/lib.rs` - add Tauri commands for PDF/render/text/save/export/OCR operations
- `src-tauri/Cargo.toml` - add PDF/OCR/TTS/file-format dependencies
- `package.json` - only add frontend dependencies if absolutely required

### New frontend files

- `src/app/state.ts` - central app state and document state
- `src/app/types.ts` - shared TS types for modes, annotations, exports, reliability states
- `src/app/canvas-mode.ts` - canvas mode behavior
- `src/app/pdf-mode.ts` - PDF mode behavior
- `src/app/tools.ts` - pen/highlighter/eraser/text tool handling
- `src/app/export.ts` - SVG export eligibility checks and export UI wiring
- `src/app/tts.ts` - TTS controls and follow-along state
- `src/app/serialization.ts` - project save/load wiring

### New backend files

- `src-tauri/src/pdf.rs` - PDF open/render/text extraction/export helpers
- `src-tauri/src/ocr.rs` - OCR fallback wrapper and confidence status mapping
- `src-tauri/src/project.rs` - save/load/autosave/recovery logic
- `src-tauri/src/types.rs` - serde models returned to frontend

This is still intentionally modest: a few modules, each with a clear role.

## Implementation phases and task order

### Phase 1: Stabilize the shell and mode split

1. Turn the current prototype into an app shell with explicit mode switching.
2. Keep the current infinite canvas as the starting Canvas Mode.
3. Add a separate empty PDF Mode screen with placeholder controls.
4. Define shared TypeScript types for documents, tools, and export eligibility.

**Why first:** the project identity depends on two distinct modes. This is the smallest structural commitment that prevents the wrong architecture.

### Phase 2: Finish a minimal correct Canvas Mode

1. Refactor current drawing code from `src/main.ts` into `src/app/canvas-mode.ts`.
2. Make pen strokes pressure-sensitive using pointer pressure instead of fixed width.
3. Keep pan and eraser tools.
4. Keep background patterns, but reduce them to the required categories: dots, lines, squares.
5. Add typed text blocks in the simplest form: click to place a text box, store plain text.
6. Keep image import simple.
7. Add PDF page import into canvas as raster snapshots generated by the backend.
8. Add simple rectangular selection for export targeting.
9. Implement SVG export only for vector/text-only selections; disable it otherwise.

**Acceptance for this phase:** canvas mode supports required note-taking behaviors and honest SVG availability.

### Phase 3: Add backend PDF rendering and opening

1. Add Rust command to open a PDF and return page count and metadata.
2. Add Rust command to render any page to an image for frontend display.
3. Add page navigation in PDF mode.
4. Reuse the same pen/highlighter overlay approach from canvas mode, but per page.
5. Save PDF annotations as overlay data in the project file first.

**Acceptance for this phase:** PDF mode is document-focused, navigable, and annotatable without pretending to be the canvas.

### Phase 4: Add minimal TTS and reliability pipeline

1. Add Rust command to extract page text spans and coarse geometry.
2. Define reliability states: `native_reliable`, `native_weak`, `ocr_reliable`, `ocr_weak`, `unavailable`.
3. Add a frontend TTS panel with play/stop and highlighting mode selection.
4. If native extraction is good enough, speak native text and show follow-along overlays.
5. If native extraction is weak, offer OCR.
6. If OCR is weak too, show a clear warning and fall back to visible manual reading aid only.
7. Keep highlighting simple: line-level first; enable word/sentence only when extraction shape supports it.

**Acceptance for this phase:** the app follows the required fallback order honestly and never fakes precision.

### Phase 5: Add recoloring and export

1. Implement a small set of recolor presets plus custom foreground/background choice.
2. Apply recoloring to PDF viewing in PDF mode.
3. Apply recoloring to imported PDF snapshots in canvas mode.
4. Add separate annotation color sets for normal and recolored viewing.
5. Export annotated PDF as flattened output.
6. Export recolored PDF as an explicit user choice.

**Acceptance for this phase:** recoloring is available for reading and optional output, and annotation visibility remains usable in both contexts.

### Phase 6: Save, autosave, and recovery

1. Define a simple JSON-based project format plus asset references.
2. Add explicit save/load for canvas projects and PDF study sessions.
3. Add autosave snapshots in an app-local directory.
4. Add recovery prompt on restart if autosave is newer than saved work.

**Acceptance for this phase:** offline local work is protected and recoverable.

## Dependency choices

### Required external choices

- **PDF engine:** prefer Pdfium binding because research suggests practical Rust ergonomics. This is the main dependency decision.
- **OCR:** use Tesseract only as a fallback path, not in the hot path.
- **TTS:** use system/local TTS, not a bundled speech engine.

### Dependencies intentionally avoided in this plan

- frontend frameworks
- large canvas editor libraries
- collaboration/sync services
- cloud OCR or cloud TTS
- deep math speech engine integration in v1
- multiple PDF engines

## Risks

### 1. Pdfium bundling may still be the hardest “simple” part

Even the simple plan depends on one solid PDF engine. If packaging or annotation export support is weaker than expected, the schedule slips quickly.

### 2. OCR setup may complicate offline installs

Tesseract is simple conceptually but not always simple operationally. The fallback path should be optional and clearly reported if unavailable.

### 3. Text reliability may be hard to classify well

The spec requires honesty. A bad confidence heuristic could either overpromise or suppress useful TTS.

### 4. Pressure behavior may vary across desktops

Pointer pressure support exists broadly, but device behavior can still differ. This needs real-device validation.

### 5. Flattened PDF annotations are simpler but weaker

This plan accepts a weaker editing/export model in exchange for lower complexity. That is a deliberate sacrifice.

## Likely difficulties

- mapping extracted text spans to visible follow-along regions reliably on complex PDFs
- keeping imported PDF page snapshots in canvas mode aligned with recolor choices
- producing recolored PDF exports without introducing quality regressions
- keeping SVG export rules strict enough that the app never claims unsupported output
- structuring autosave so moved source PDFs do not silently break sessions

## Development order summary

1. Split the app into real Canvas Mode and PDF Mode.
2. Finish the simplest correct canvas workflow.
3. Add backend PDF rendering and open/navigate.
4. Add overlay-based PDF annotation.
5. Add native-text TTS and simple highlighting.
6. Add OCR fallback and warning states.
7. Add recoloring in view and export.
8. Add save/autosave/recovery.
9. Tighten export rules and cross-check correctness criteria.

## Validation checklist

The implementation should be considered successful only if it verifies all required spec points in the simplest possible way:

- Tauri app with Rust native side
- two separate first-class modes
- offline core workflows
- pressure-sensitive pen input
- direct PDF annotation
- TTS in PDF mode
- fallback order: native text, OCR, clear warning
- visible follow-along when reliable enough
- PDF recoloring
- selection-aware export with honest SVG availability
- understandable minimal UI

## Final recommendation from the “simple” perspective

Do not chase deep math understanding, editable PDF-native annotation internals, or a polished generalized document model in v1. Build the smallest honest tool that gets pen-first canvas work, document-focused PDF study, fallback-aware TTS, and portable export working inside one lightweight Tauri app.