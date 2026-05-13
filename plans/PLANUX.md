# Plan Summary

| Field                   | Value |
| ----------------------- | ----- |
| Optimization target     | User-experience-first desktop study workflow |
| What it optimizes for   | Pen-first ergonomics, mode clarity, low-friction annotation, readable follow-along TTS, honest reliability feedback, visual comfort, and discoverable export behavior |
| What it sacrifices      | Initial implementation speed, architectural minimalism, lower dependency count, and some raw performance simplicity |
| Proposed stack          | Tauri desktop shell, Rust backend, TypeScript frontend, web canvas for pen interaction, Rust PDF engine layer, local OCR fallback, system/local TTS, optional math-speech subsystem |
| Architecture shape      | UX-led dual-mode app with separate Canvas and PDF workspaces sharing common document services, annotation styling, and trust-state feedback |
| Major risks             | PDF engine choice may limit UX polish, follow-along reliability will vary by document quality, OCR confidence may be hard to communicate well, math-aware speech may delay polished reading UX |
| Estimated build speed   | Moderate to slow because workflow polish is prioritized early |
| Dependency profile      | Moderate to high; chosen to support better reading, annotation, OCR, and feedback quality |
| Performance profile     | Good enough for personal desktop use, but not aggressively optimized ahead of UX needs |
| User-experience profile | Very high emphasis on comfort, clarity, trust, and tablet-first study flow |

## UX Planning Stance

This plan is intentionally not balanced. It prioritizes the feeling of studying well over the fastest or leanest build. The app should feel trustworthy, calm, and direct for a solo technical student using a drawing tablet. If extra implementation complexity is required to make mode boundaries obvious, pen behavior feel natural, follow-along reading understandable, and failure states honest, that complexity is accepted.

## Architecture

### 1. Product shape: two strongly separated workspaces

Build the app around two explicit top-level workspaces rather than one generic document surface:

- **Canvas Workspace**: infinite visual note-taking, placement of imported images and PDF pages, typed text, pen drawing, background patterns, selection-aware export
- **PDF Workspace**: document reading, direct PDF annotation, TTS playback, follow-along highlighting, recoloring, text reliability feedback

These should share some services, but not share the same interaction model.

Why this is UX-first:

- reinforces the product identity from `IDEA.md`
- prevents the Xournal++-style “PDF as background only” confusion warned about in the research
- lets each mode have its own tools, defaults, and mental model

### 2. Shared service layer, separate mode controllers

Use a shared backend service layer and separate frontend mode controllers:

- **Shared services**
  - filesystem and save/recovery
  - project/document registry
  - annotation color profiles
  - export routing
  - PDF text reliability assessment
  - OCR pipeline access
  - TTS session control
- **Canvas controller**
  - pen tools
  - spatial selection
  - placement and transform behavior
  - SVG export eligibility checks
- **PDF controller**
  - page navigation
  - viewport and reading focus
  - PDF annotation overlay
  - TTS/follow-along state
  - recolor controls
  - trust-state messaging

This increases complexity, but makes each mode feel purpose-built.

### 3. UX-critical state model: reliability and trust are first-class

Treat reading confidence as a product surface, not internal metadata.

Define explicit states in PDF mode:

- native text reliable
- native text weak
- OCR-derived text
- no reliable text support

Use these states to control:

- available highlight granularity
- warning banners or badges
- wording near TTS controls
- export notes when relevant

This follows the spec requirement to degrade honestly instead of pretending precision exists.

## Tools And Development Stack

### Required foundation

- **Desktop shell**: Tauri
- **Native layer**: Rust
- **Frontend**: TypeScript
- **Pen interaction**: browser-side pointer events and canvas-based drawing

### PDF and reading stack

Choose a Rust-integrated PDF engine that supports:

- rendering pages cleanly
- text extraction
- annotation access/output
- page image rasterization for OCR fallback

From the research, realistic candidates are:

- **Pdfium via `pdfium-render`**: best fit for pragmatic Rust integration, but thread-safety and bundling must be handled carefully
- **Poppler**: strong open-source option, but likely more glue for product behavior
- **Avoid early MuPDF lock-in** unless license implications are explicitly accepted

UX-first recommendation: pick the engine that enables the most controllable annotation, extraction, and export behavior for a polished user flow, even if setup is more involved.

### OCR and TTS stack

- **OCR**: Tesseract fallback via rasterized page images
- **Speech output**: local/system TTS backend
- **Math-aware enhancement**: phased integration path for Speech Rule Engine and/or MathCAT when structured math is available

UX-first interpretation:

- ordinary TTS is not enough, but it is enough to build the first honest reading workflow
- math-aware improvements should only appear where the app can explain them clearly

## Project Structure

Use a structure that mirrors the two-workspace product shape:

```text
src/
  app/
    shell/
    navigation/
    shared-ui/
    trust-state/
  canvas/
    tools/
    rendering/
    document-model/
    import/
    export/
  pdf/
    viewer/
    annotation/
    tts/
    follow-along/
    recolor/
    trust/
  common/
    annotation-style/
    autosave/
    project-files/
    settings/
    commands/
src-tauri/
  pdf/
  ocr/
  tts/
  export/
  files/
  recovery/
  diagnostics/
plans/
research/
```

Why this matters:

- preserves mental clarity in code the same way the app should preserve mental clarity in UI
- avoids collapsing PDF and canvas behaviors into one generic subsystem too early

## Interaction Principles

### 1. Pen-first by default

Set defaults to reduce friction for tablet use:

- large-enough hit targets for tool switching and page navigation
- visible active tool state at all times
- pressure-sensitive drawing enabled by default
- minimal modal dialogs during annotation and reading
- common actions accessible without keyboard dependence

### 2. Minimal visible chrome, strong contextual affordances

Use a sparse UI, but not a hidden one.

- keep the screen mostly for document/canvas content
- show only mode-relevant tools
- make export eligibility and text reliability visible without forcing the user to inspect logs or settings

### 3. Follow-along readability over false precision

When TTS runs:

- prefer stable, understandable focus indicators over flashy but unreliable synchronization
- only offer word-level highlighting when confidence is high enough
- fall back to line/region emphasis or visible reading-guidance overlay when precision is weak
- always communicate when the current behavior is approximate

### 4. Comfort controls are core, not secondary

Surface recoloring and annotation appearance controls prominently in PDF mode.

- allow quick switching between normal and recolored viewing
- allow separate annotation color profiles for normal and recolored contexts
- make these controls easy to test live on the current page

## Implementation Phases

### Phase 1: UX skeleton and mode separation

Goal: make the product feel like the right product before deep feature completeness.

Tasks:

1. Build the Tauri app shell with explicit entry points for Canvas Workspace and PDF Workspace.
2. Implement shared navigation and document-opening flows that route clearly into one mode or the other.
3. Create a shared settings model for pen behavior, annotation colors, recoloring presets, and trust-state presentation.
4. Build placeholder panels for mode-specific tools so the UI layout and affordance model are established early.

Acceptance:

- opening a canvas never feels like opening a PDF reader
- opening a PDF never feels like entering a generic canvas
- the mode split is obvious to a first-time user

### Phase 2: Canvas workflow feel first

Goal: make the infinite canvas comfortable enough for real visual study notes.

Tasks:

1. Implement pressure-sensitive freehand drawing with responsive stroke rendering.
2. Add typed text placement and editing.
3. Add image import and PDF-page placement as visual objects.
4. Implement scalable background patterns: dots, lines, squares.
5. Add move/select/arrange behavior optimized for pen-led use.
6. Implement selection-aware export eligibility logic, especially SVG-compatible selection detection.
7. Add SVG export for vector/text-only targets and clear disabled-state messaging otherwise.

Acceptance:

- pen input feels direct and low-latency enough for note-taking
- background patterns remain useful across zoom levels
- export affordances clearly explain why SVG is or is not available

### Phase 3: PDF reading and annotation UX baseline

Goal: make PDF mode dependable and calm before advanced read-aloud behavior.

Tasks:

1. Integrate PDF rendering and document navigation.
2. Implement direct PDF annotation with pen strokes, highlights, and simple text notes.
3. Add recolored viewing with live preview.
4. Add separate annotation color settings for normal and recolored viewing.
5. Preserve a document-focused layout: reading area, simple navigation, annotation tools, reading-status area.

Acceptance:

- a user can read and annotate a technical PDF without mode confusion
- annotation remains visible in both normal and recolored views
- the UI does not expose canvas concepts in PDF mode

### Phase 4: Trustworthy TTS and follow-along system

Goal: make read-aloud useful and honest, not just available.

Tasks:

1. Build PDF text extraction pipeline with reliability classification.
2. Implement TTS playback using native text when reliable.
3. Add follow-along highlighting with capability tiers:
   - word-level when confidence is strong
   - line/sentence/region-level when confidence is moderate
   - visible fallback guidance when precision is weak
4. Add OCR fallback path for scanned or weak-text PDFs.
5. Add clear user-visible trust-state messaging for native text, OCR-derived text, and unreliable results.
6. Prevent silent failure by disabling misleading precision modes when the source does not support them.

Acceptance:

- the user always knows whether follow-along is exact, approximate, OCR-based, or unreliable
- weak PDFs still produce a usable reading aid or an honest warning

### Phase 5: Recovery, export, and polish of study workflow

Goal: make the product safe and dependable for daily study use.

Tasks:

1. Implement save, autosave, and recovery flows for both canvas and PDF work.
2. Implement annotated PDF export and optional recolored export.
3. Add project/file-level recovery messaging that is calm and understandable.
4. Polish workspace switching, last-opened state, and interruption recovery.
5. Tune tool defaults for minimal friction during repeated study sessions.

Acceptance:

- user work is hard to lose
- exports are understandable and predictable
- reopening the app restores the study session cleanly where practical

### Phase 6: Math-aware reading enhancement

Goal: improve the value of technical read-aloud without lying about coverage.

Tasks:

1. Identify limited contexts where structured math can be extracted or reconstructed reliably enough.
2. Integrate a math-speech subsystem for those cases.
3. Keep math enhancement clearly bounded by confidence and document quality.
4. Expose the difference between plain text reading and math-aware reading in a user-comprehensible way.

Acceptance:

- math-aware output improves some documents measurably
- unsupported equations do not masquerade as successfully interpreted math

## Task Order

1. App shell and explicit mode split
2. Shared settings and trust-state model
3. Canvas pen workflow and export affordances
4. PDF rendering, navigation, and annotation
5. Recoloring and annotation appearance handling
6. Native text extraction and TTS baseline
7. Follow-along behavior tiers and user messaging
8. OCR fallback with reliability communication
9. Save/autosave/recovery
10. PDF export polish
11. Math-aware reading enhancement

This order is intentionally UX-led, not technology-led. It establishes the product feel and study workflow before the hardest advanced subsystem.

## Dependencies

- Phase 2 depends on Phase 1 shell and shared settings foundation.
- Phase 3 depends on selecting and integrating a PDF engine.
- Phase 4 depends on Phase 3 rendering/navigation and the shared trust-state model.
- OCR fallback depends on page rasterization support from the PDF layer.
- Math-aware reading depends on a stable TTS pipeline and at least partial structured text/math handling.
- Export polish depends on annotation persistence design and PDF engine output capabilities.

## Risks

### 1. UX ambition may outpace implementation speed

This plan deliberately spends early effort on workflow quality, mode clarity, and trust messaging. That may slow delivery of a “feature checklist complete” build.

### 2. PDF engine choice may constrain polish

If the chosen engine makes annotation export, text extraction, or recolor output awkward, the intended reading experience may become inconsistent.

### 3. Follow-along trust UI may be hard to design well

Too much warning language creates noise; too little misleads the user. The balance is difficult and central.

### 4. OCR-derived reading quality may vary too much

Even with a clean fallback pipeline, scanned technical PDFs can produce frustrating results. The UX must communicate that clearly without making the app feel broken.

### 5. Math-aware reading could become a sinkhole

The research strongly suggests that generic TTS will not solve math. A UX-first plan must resist overpromising and keep this subsystem phased.

### 6. Pen behavior may differ across desktop setups

Pointer pressure support is validated in principle, but real tablet behavior may still vary by platform and hardware.

## Likely Difficulties

- making PDF annotation feel direct while also preserving export correctness
- deciding when to show word-level, line-level, or approximate highlighting
- presenting OCR confidence in a way that helps instead of annoys
- keeping the UI minimal while exposing enough controls for recoloring, annotation appearance, export eligibility, and reliability status
- supporting imported PDF pages in canvas mode while keeping their behavior visually coherent and simple
- preserving a calm interface as more advanced reading features are added

## Bias Check

This plan intentionally favors:

- stronger visible affordances
- clearer mode boundaries
- richer trust-state communication
- extra customization for reading comfort
- phased but user-facing honesty around TTS and math

It intentionally does **not** optimize for:

- fewest components
- fastest possible first prototype
- lowest dependency count
- simplest backend architecture
- maximum early performance tuning

That tradeoff is acceptable because the product’s value, according to the provided context, depends most on the quality and trustworthiness of the study experience.