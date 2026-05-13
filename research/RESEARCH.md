# rpdf Research

Last updated: 2026-05-10

## Research Scope

This pass tests the idea in `IDEA.md` against the current landscape.

Focus:

- whether the product is worth building
- what strong existing tools already cover
- which engine and architecture directions look realistic
- what makes math-heavy PDF reading support hard
- which risks matter before planning implementation

This is a first-pass research synthesis for `rpdf2`. There was no previous `research/RESEARCH.md` in this repo.

## Research Questions

1. Is there still a meaningful gap between existing note-taking tools and the desired two-mode workflow?
2. Which existing tools already cover the canvas side well, and which cover the PDF side well?
3. What are the realistic options for PDF rendering, annotation, text extraction, OCR fallback, and TTS?
4. Is math-aware read-aloud a realistic differentiator, or is it too ambitious for an early version?
5. Which licensing or dependency choices could become expensive later?

## Summary

`rpdf2` still looks worth building as a personal tool, but not because the infinite canvas or PDF annotation ideas are novel by themselves. Those already exist. The strongest reason to build it is the specific combination of:

- tablet-first desktop interaction
- two clearly separated modes
- offline-first operation
- better-than-usual reading support for technical PDFs
- honest fallback behavior when PDF text quality is poor

The main conclusion from the research is that the canvas and annotation parts are realistic, while math-aware reading support is the true differentiator and the true risk. That means the project should avoid pretending that generic OCR plus generic TTS solves the hard part.

The best early strategy is:

1. build a strong offline stylus-first canvas and PDF annotation base
2. use a real PDF engine with reliable rendering and text extraction
3. treat reading support as a layered pipeline:
   - native PDF text first
   - OCR second
   - explicit warning third
4. keep math speech as a targeted subsystem, not as a vague promise

## Key Findings

### 1. The product gap is real, but it is narrow

There are already strong tools in each neighboring category:

- Rnote is strong on stylus-first canvas note-taking, spatial layout, background patterns, SVG export, and document import/export.
- Xournal++ is strong on pen input and PDF annotation.

What is still not well covered by a single offline desktop tool is the exact combination described in `IDEA.md`:

- one app for visual canvas notes and direct PDF reading
- drawing-tablet-first interaction
- low-complexity desktop UI
- serious math-heavy TTS expectations

This means `rpdf2` is plausible as a personal product, but weak as a generic “yet another notes app” pitch.

### 2. Existing tools validate the two-mode split

The research strengthens the idea that canvas mode and PDF mode should stay separate.

Xournal++ effectively treats PDF annotation as work on top of a journal background, and its docs expose the downsides of that model:

- the journal and PDF are separate
- path management can break
- exports flatten annotation editability
- existing PDF-native annotations are not fully preserved on export

That is useful evidence that “PDF as just another canvas background” is not enough for the PDF-mode half of `rpdf2`.

Rnote validates the opposite side:

- stylus-first infinite canvas
- background patterns
- selection export
- document import/export

So the two-mode idea is not just conceptual neatness. It matches how the neighboring tools naturally lean.

### 3. PDF reading order and highlighting reliability are genuinely hard

W3C’s PDF accessibility guidance makes the core problem explicit: correct reading order depends on proper tagged structure, and complex layouts often do not convert cleanly even when authors try to do the right thing.

Implication:

- line-by-line or sentence-by-sentence highlighting cannot be promised blindly
- multi-column pages, footnotes, sidebars, tables, and complex layouts will break naive reading flows
- “text exists in the PDF” is not the same as “the reading sequence is trustworthy”

This directly supports the fallback rule already implied in `IDEA.md` and `SPEC.md`.

### 4. OCR is viable, but only as a fallback and only with honesty

Tesseract’s own docs make two implementation constraints clear:

- Tesseract does not read PDF directly, so the app must rasterize PDF pages first.
- OCR quality is highly dependent on preprocessing, segmentation, skew, borders, and image quality.

This means OCR is useful, but it should not be used to fake confidence. It is a fallback path, not proof that the app now understands the document.

### 5. Math-aware speech is realistic only if it becomes a dedicated subsystem

Generic system TTS can speak text, but it does not solve mathematical notation interpretation on its own.

The most relevant evidence found:

- Speech Rule Engine is specifically built for generating speech from mathematical expressions and supports established speech rules like Mathspeak and Clearspeak.
- MathCAT provides an API around MathML, speech output, braille output, and navigation-aware highlighting hooks through node ids.
- Paper2Audio shows that better handling of equations, tables, figures, and scanned documents is a real market need, but it is a cloud product, not an offline desktop one.

Implication:

- math-aware reading support is possible
- but only if `rpdf2` can transform at least some equations into a structured representation such as MathML or equivalent semantic intermediate data
- for many PDFs, especially messy ones, that transformation will be incomplete or impossible

So “read math as intelligently as possible” is a valid direction, but only as a graded capability, not an all-or-nothing promise.

### 6. The current Tauri + web canvas direction is validated for stylus input

MDN confirms broad support for `PointerEvent.pressure`, with normalized values from `0` to `1` and cross-browser availability dating back to July 2020.

That matters because it means the browser-side canvas inside Tauri is a realistic place to implement:

- pressure-sensitive drawing
- pen vs mouse differentiation
- future tilt/twist extensions if needed later

This does not solve the PDF side, but it does validate the canvas interaction model already present in the repo.

## Existing Tools And Similar Projects

## Rnote

What it proves:

- there is demand for stylus-first infinite-canvas note-taking
- SVG export and spatial note workflows are practical
- a Rust desktop app can ship this kind of interaction model

Where it differs from `rpdf2`:

- it is broader as a note/drawing tool than as a study-reader tool
- it does not present math-heavy PDF TTS as a core identity
- its native file format is explicitly unstable

Takeaway:

Rnote is evidence that the canvas side is feasible, but also evidence that the canvas side alone is not enough to justify `rpdf2`.

## Xournal++

What it proves:

- strong pen-based PDF annotation is practical
- desktop users care about stylus support, PDF text interaction, and export

Where it differs from `rpdf2`:

- its PDF model is background-centric rather than a distinct reading mode
- exported PDF annotations become non-editable
- built-in PDF annotations are not fully preserved on export
- math-aware reading support is not its core promise

Takeaway:

Xournal++ is a strong reference for interaction quality, but also a clear warning that PDF handling should not be reduced to “draw on top of a file and flatten later.”

## Paper2Audio

What it proves:

- users do want document-to-audio workflows that handle math, tables, figures, and complex formatting better than ordinary TTS
- this is strong enough to support a dedicated product

Where it differs from `rpdf2`:

- cloud-first
- account-based
- not tablet-first
- not centered on direct PDF annotation or canvas notes

Takeaway:

Paper2Audio is validation for the reading-support problem, not a direct product template.

## Libraries, APIs, And Architectures

## Canvas and input

The current frontend direction looks reasonable:

- Tauri shell
- TypeScript/web canvas UI
- Rust for filesystem and native integration

Based on the pointer-event support evidence, pressure-sensitive drawing on the frontend is realistic and should stay browser-side unless a later performance issue forces a different design.

## PDF engine options

### Poppler

Strengths:

- mature open-source stack
- actively maintained
- proven text extraction utilities such as `pdftotext`
- accessible from Rust via bindings

Weaknesses:

- more system-library-shaped than product-library-shaped
- may require more glue for editing and app-level annotation workflows

### MuPDF

Strengths:

- strong rendering and extraction capabilities
- explicit annotation support
- lightweight performance-oriented positioning

Weaknesses:

- dual licensing is a real constraint
- choosing MuPDF means accepting AGPL implications or commercial licensing

### Pdfium via `pdfium-render`

Strengths:

- good Rust ergonomics
- runtime binding to bundled native library is possible
- covers rendering, extraction, forms, links, and annotations
- fits a Rust/Tauri architecture well

Weaknesses:

- native library bundling becomes part of the product work
- docs explicitly caution that Pdfium itself should be treated as not thread-safe

## TTS options

For ordinary offline reading, a system-backed TTS layer is realistic. The Rust `tts` crate is relevant because it exposes multiple platform backends, including Linux via Speech Dispatcher, Windows backends, macOS/iOS backends, Android, and WebAssembly.

Inference:

This makes a two-layer TTS design look realistic:

1. use local/system TTS for speech output
2. improve the input text before it reaches TTS, especially for math-heavy sections

The hard part is therefore text normalization and math interpretation, not audio synthesis itself.

## Math speech options

The most promising direction is not “invent a new math reader from scratch.” It is:

1. extract or reconstruct structured math
2. pass it through a math-aware speech engine
3. keep visual highlighting synchronized only where confidence is good

The two most relevant candidates from this pass are:

- Speech Rule Engine for rule-based math speech generation
- MathCAT for math speech plus navigation-aware structure

MathCAT looks especially relevant if synchronized navigation/highlighting becomes part of the user experience, because its API already thinks in terms of MathML, spoken text, and node ids.

## Useful Sources

- Rnote: https://rnote.flxzt.net/
- Rnote repo: https://github.com/flxzt/rnote
- Xournal++ repo: https://github.com/xournalpp/xournalpp
- Xournal++ PDF guide: https://xournalpp.github.io/guide/pdfs/
- Poppler: https://poppler.freedesktop.org/
- MuPDF Core: https://mupdf.com/core
- Artifex licensing: https://artifex.com/licensing
- pdfium-render: https://docs.rs/pdfium-render
- poppler-rs: https://docs.rs/poppler-rs/latest/poppler/
- Rust `tts` crate: https://docs.rs/tts
- Speech Rule Engine: https://speechruleengine.org/
- MathCAT callers guide: https://nsoiffer.github.io/MathCAT/callers.html
- W3C PDF3 reading-order guidance: https://www.w3.org/WAI/WCAG21/Techniques/pdf/PDF3
- Tesseract quality guide: https://tesseract-ocr.github.io/tessdoc/ImproveQuality.html
- Tesseract input formats: https://tesseract-ocr.github.io/tessdoc/InputFormats.html
- Paper2Audio about: https://www.paper2audio.com/about
- MDN `PointerEvent.pressure`: https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent/pressure

## Saved Material

Saved in this repo:

- `research/source-notes.md`

No external binaries or PDFs were downloaded in this pass. The saved material is a compact note file because that is enough for the next planning step.

## What This Changes About The Idea

The research narrows the project.

What becomes stronger:

- two truly separate modes
- tablet-first canvas UX
- offline-first architecture
- honest reliability model for read-aloud

What becomes weaker:

- any idea that a single generic PDF/text library will solve math-heavy TTS cleanly
- any idea that OCR can silently “fix” bad PDFs
- any idea that PDF mode can be treated as just imported pages on a canvas

What becomes more concrete:

- the app needs a deliberate document-understanding pipeline, not just a renderer
- math support should be phased and measurable
- licensing must be considered early when choosing the PDF engine

## Realistic Approaches

### Approach A: Rust PDF engine plus web canvas UI

Use:

- Tauri for shell
- TypeScript/web canvas for interaction
- Rust for file handling and PDF engine integration
- a dedicated PDF engine such as Pdfium, Poppler, or MuPDF

Why it looks realistic:

- fits the current repo shape
- keeps stylus interaction where pointer APIs are already good
- keeps document parsing and file output on the Rust side

### Approach B: Ship a strong V1 without “full math understanding”

V1 reading support target:

- normal text PDFs
- OCR fallback for scanned pages
- user-visible confidence/warning states
- basic follow-along highlighting only when alignment is good

Then later:

- targeted math normalization
- Speech Rule Engine and/or MathCAT integration
- better equation-region handling

Why this is realistic:

- it avoids blocking the whole product on the hardest subsystem
- it preserves the honest product identity

### Approach C: Treat math reading as progressive enhancement

Possible confidence tiers:

1. native tagged text with trustworthy order
2. native extracted text with weak structure
3. OCR-derived text
4. math-aware reconstructed fragments only in limited cases

Why this is realistic:

- it matches the evidence from PDF structure and OCR limitations
- it gives the UI a principled way to explain what it can and cannot do

## Approaches To Avoid

### Avoid: “PDF mode is just the canvas with pages dropped in”

Research on Xournal++ supports this warning. That model leads to document-sync and export limitations.

### Avoid: “OCR means the document is now reliable”

Tesseract’s own docs argue against this assumption. OCR quality is conditional and preprocessing-dependent.

### Avoid: “Generic TTS will read math well enough”

That is the weakest assumption in the whole idea. The research contradicts it.

### Avoid: early lock-in to a PDF engine without license review

MuPDF in particular is attractive technically but has licensing implications that should be treated as a first-order decision.

## Risks That Matter For Planning

1. PDF engine choice may become an expensive reversal if annotation/export behavior or licensing turns out wrong.
2. Math-aware reading support may consume far more effort than the rest of the app if not phased carefully.
3. Highlight synchronization may look polished on clean PDFs and break badly on real technical material.
4. OCR fallback may disappoint unless preprocessing and user communication are handled well.
5. Cross-platform stylus behavior may still differ even if pointer pressure is broadly supported in principle.

## Assumptions From IDEA.md To Revisit

1. “Math should be handled as intelligently as possible” should later be rewritten into concrete capability tiers.
2. “Current spoken content is highlighted” needs a confidence model, not a universal promise.
3. “Export recolored PDFs” should later be checked against the selected PDF engine’s capabilities and output quality.
4. “Low resource use” may conflict with heavy OCR and advanced document-understanding passes unless those are designed as optional or on-demand steps.

## Open Questions

1. Which matters more for V1: native editable PDF annotations or fast rasterized visual markup with later export?
2. Is Linux-first acceptable for early releases, especially for stylus and TTS backend consistency?
3. Should V1 support importing PDF pages into the canvas as raster snapshots only, or preserve richer page metadata for recoloring/export later?
4. Which PDF engine best matches the license goals of this project?
5. How much of the math-reading problem should be solved locally in V1 versus explicitly deferred?

## Research Log

- Read local `IDEA.md`, `SPEC.md`, and lightweight repo context.
- Compared the idea against current Rnote and Xournal++ materials.
- Checked current Poppler, MuPDF, `pdfium-render`, `poppler-rs`, and Rust `tts` materials.
- Checked Speech Rule Engine and MathCAT as math-speech candidates.
- Checked W3C reading-order guidance and Tesseract OCR limitations.
- Saved the reusable external notes into `research/source-notes.md`.
