# Research: rpdf

## Research Scope

This research pass evaluates the current `IDEA.md` against reality.

The current idea is a personal desktop study tool with two core modes:

1. an infinite canvas for visual notes
2. a PDF reading and annotation mode

Key claims and assumptions tested in this pass:

- whether the feature combination is already well served by existing tools
- whether drawing-tablet-first desktop interaction is realistic
- whether PDF recoloring is a meaningful and validated feature
- whether SVG export from vector-only canvas content is a reasonable product rule
- whether strong PDF text-to-speech, especially for math-heavy material, is realistic
- which technical and product risks are likely to matter later

## Research Questions

1. Do strong existing tools already solve this exact workflow well enough that building `rpdf` would be redundant?
2. Which parts of the idea are already validated by existing products?
3. Which parts appear difficult or risky, especially PDF TTS and math-heavy reading?
4. Which rendering and document APIs look realistic for later implementation work?
5. Which assumptions from `IDEA.md` look solid, and which ones need caution?

## Summary

The idea appears worth building.

The strongest conclusion from this pass is that the feature set is not imaginary or incoherent. Most individual parts are already validated by existing tools:

- stylus-first infinite-canvas note taking exists
- PDF annotation exists
- PDF recoloring exists
- SVG export for vector-based note content exists
- PDF text extraction and annotation APIs exist
- math speech technology exists

What does **not** appear to be well served by one existing desktop tool is the specific combination you want:

- desktop-first
- drawing-tablet-first
- two clear modes instead of one awkward hybrid
- lightweight/minimal
- strong PDF reading support
- configurable follow-along highlighting during TTS
- explicit attention to math-heavy documents

That means the idea is not invalidated by the current landscape. It is also not starting from zero. Many pieces are already proven separately.

The main risk is not "can this exist at all?" The main risk is that the PDF reading/TTS/math side is substantially harder than the canvas/annotation side, especially if the product wants reliable spoken reading order and meaningful math handling across messy real-world PDFs.

## Key Findings

### 1. The canvas-and-stylus part is strongly validated

Rnote is the clearest existing proof that a modern stylus-first desktop note tool is viable. Its public feature list already includes:

- adaptive UI focused on stylus input
- pressure-sensitive stylus input
- infinite canvas layouts
- customizable background colors, patterns, and sizes
- PDF, bitmap, and SVG import
- document, page, and selection export including SVG

This matters because several of your desired features are already validated in one real product:

- drawing-tablet-first interaction
- patterned backgrounds
- mixed-content canvas
- selection-based export

Research implication:

The infinite-canvas side of `rpdf` is realistic and not unusual. It is a credible foundation, not a speculative one.

### 2. Existing PDF annotation tools still leave a gap

Xournal++ proves that PDF annotation on desktop with pen input is useful and mature, but its model also reveals an important limitation: it stores a separate journal file tied to a PDF background and documents export limitations explicitly.

Its docs highlight that:

- the PDF is treated as a background
- the journal file is separate from the PDF
- export to PDF flattens annotations in ways that are not re-editable
- built-in PDF annotations/forms are not fully preserved on export

Research implication:

This validates the demand for desktop PDF markup, but also warns that "annotation on top of a PDF" often leads to awkward data-model compromises. Your idea should expect this area to be tricky and should not assume that native PDF annotation, overlay-style annotation, and reversible export all align cleanly.

### 3. Recoloring is clearly a real user feature, not a niche idea

Zathura documents a built-in recolor command (`Ctrl+R`) for grayscale/inverted reading. That is enough to validate the basic user need: many readers want visual comfort controls that reduce bright white pages.

Research implication:

Recoloring belongs in the idea. It is not cosmetic. It is a recognized document-reader feature.

However, existing evidence mostly validates recoloring as a viewing mode, not as a fully standardized PDF-editing operation. This supports your current product framing:

- recolor for viewing first
- optionally support recolored output/export later

### 4. SVG export rules are product-coherent

Rnote explicitly supports SVG export for document, page, and selection content. That does not prove your exact export rule, but it strongly supports the idea that vector-first note content and selection-based export are practical and user-meaningful.

Research implication:

Your proposed rule is good product design:

- whole canvas export by default
- selection-aware export when items are selected
- SVG available only when the selected content is compatible

This is both understandable and aligned with real tool behavior.

### 5. Desktop tablet input is realistic

Qt's `QTabletEvent` API explicitly exposes:

- pressure
- rotation
- tangential pressure
- tilt
- device identity

This is not a product comparison but an implementation-feasibility signal. It confirms that mainstream desktop UI stacks do support the kind of stylus input your idea depends on.

Research implication:

The drawing-tablet requirement is realistic on desktop. It should be treated as a first-class input model, not as a weird edge case.

### 6. PDF TTS is valuable, but PDF structure quality is a serious constraint

W3C's PDF accessibility guidance makes the main problem very clear:

- reading order depends heavily on correct tagging and structure
- complex layouts can break logical reading order
- multi-column documents can be read incorrectly if they are not properly tagged

Research implication:

This is the biggest reality check in the project.

`rpdf` can plausibly provide useful TTS, highlighting, and navigation for many text-based PDFs. But it should not assume that arbitrary technical PDFs will always yield reliable sentence order, paragraph order, or math segmentation.

This does **not** kill the idea. It just means the product promise should remain grounded:

- excellent support for well-structured text PDFs
- useful best-effort support for harder documents
- explicit handling of failure cases later

### 7. Math-aware speech is plausible, but PDF math remains the hardest part

MathJax's accessibility components connect to the Speech Rule Engine and can generate speech strings for mathematics. MathJax also has an explorer mode for interactive math exploration.

This validates an important part of the vision:

- there is real, modern infrastructure for math speech
- math can be spoken more meaningfully than raw symbol-by-symbol fallback

But there is an important limitation:

MathJax/SRE work well when you have actual mathematical structure to work from, such as MathML or semantically enriched math. PDFs often do not give you that structure cleanly.

Harvard's accessibility guidance states this very directly: PDFs are unreliable for accessible math, and alternative formats like HTML with MathML or Office formats are preferred.

Research implication:

Math-aware TTS is plausible as a product differentiator, but only if the idea stays honest about the input problem. The hard part is not only speech generation. The hard part is recovering enough structure from technical PDFs to speak math well.

### 8. There is evidence of demand for better academic-document TTS

Paper2Audio positions itself around accurate narration for research papers and claims specific handling for figures, tables, and math rather than simply reading everything literally.

Research implication:

This supports the demand side of your idea. People do want better document listening for technical material. But current visible solutions are more web/mobile and document-consumption-oriented than desktop pen-first study-workspace tools.

That makes your concept more differentiated, not less.

### 9. Mature PDF engines exist, but their tradeoffs matter

MuPDF presents a compelling technical profile for later planning:

- fast rendering
- structured text extraction
- annotation support
- conversion to and from formats including SVG
- low-level PDF access

Its `StructuredText` API groups extracted text into blocks, lines, and spans, which is directly relevant to highlighting and TTS alignment work.

However, MuPDF's license is AGPL unless you buy a commercial license.

Research implication:

MuPDF looks technically attractive for this project, especially if lightweight rendering and structured extraction remain priorities. But its licensing model is a real planning constraint.

## Existing Tools And Similar Projects

### Rnote

What it covers well:

- stylus-first note taking
- pressure-sensitive drawing
- infinite canvas
- background patterns
- image/PDF/SVG import
- SVG export

Why it matters:

It is the strongest proof that your canvas mode is realistic.

What it does not appear to center:

- advanced PDF TTS
- math-aware reading support
- a dedicated PDF reading mode with follow-along speech highlighting

### Xournal++

What it covers well:

- desktop handwritten notes
- PDF annotation
- pen-first workflows
- LaTeX support

Why it matters:

It validates the usefulness of desktop PDF markup.

What it reveals:

- separate journal/background models create complexity
- export/editability tradeoffs are real

### Zathura

What it covers well:

- lightweight PDF viewing
- keyboard-driven reading
- recoloring

Why it matters:

It validates recoloring and minimal document-viewer expectations.

What it does not provide:

- visual notes workspace
- stylus-first annotation environment
- strong TTS workflow

### Paper2Audio

What it covers well:

- academic-document listening
- handling figures/tables/math differently from normal prose

Why it matters:

It validates the need for better complex-document TTS.

What it does not cover:

- desktop pen workflow
- local note-taking workspace
- PDF annotation as a first-class experience

## Libraries, APIs, And Architectures

This section is not an implementation plan. It only records which directions look realistic enough to consider later.

### PDF engines

#### MuPDF

Pros:

- fast and lightweight positioning
- rendering, extraction, annotation, and conversion in one stack
- structured text API useful for search/highlighting/TTS alignment

Cons:

- AGPL/commercial licensing constraint

Why it stands out:

It is the clearest single-stack candidate if later planning values performance, extraction, and a compact rendering core.

#### Poppler

Pros:

- mature and widely used
- multiple APIs including Qt bindings
- explicit support for text search and annotation classes

Cons:

- the research gathered here does not yet show the same compact, unified positioning around structured extraction and conversion that MuPDF advertises

Why it matters:

It is a realistic alternative and should remain in scope for later technical comparison.

### Desktop tablet input

#### Qt tablet APIs

Qt explicitly supports pressure, tilt, rotation, and related tablet-event data. This makes it a realistic UI/toolkit direction if tablet-first desktop interaction remains central.

### Math speech

#### MathJax + Speech Rule Engine

Pros:

- real math speech generation stack
- existing accessibility tooling
- configurable speech domains/locales

Cons:

- assumes usable mathematical structure exists
- PDF extraction may not provide that structure reliably

Why it matters:

This is a realistic piece of a later math-speech pipeline, but not a complete solution by itself.

## Useful Sources

- Rnote features: https://rnote.flxzt.net/
- Xournal++ PDF guide: https://xournalpp.github.io/guide/pdfs/
- Zathura documentation: https://pwmt.org/projects/zathura/documentation/
- Qt tablet events: https://doc.qt.io/qt-6/qtabletevent.html
- W3C PDF reading order guidance: https://www.w3.org/WAI/WCAG22/Techniques/pdf/PDF3
- MathJax accessibility components: https://docs.mathjax.org/en/stable/web/components/accessibility.html
- MathJax and SRE integration note: https://www.mathjax.org/MathJax-v3.2.1-available/
- MuPDF product overview: https://mupdf.com/
- MuPDF structured text docs: https://mupdf.readthedocs.io/en/latest/reference/javascript/types/StructuredText.html
- MuPDF licensing: https://mupdf.readthedocs.io/en/1.26.3/license.html
- Artifex licensing overview: https://artifex.com/licensing
- Poppler project homepage: https://poppler.freedesktop.org/
- Poppler Qt6 annotation docs: https://poppler.freedesktop.org/api/qt6/classPoppler_1_1Annotation.html
- Poppler Qt6 page search docs: https://poppler.freedesktop.org/api/qt6/classPoppler_1_1Page.html
- Harvard accessible math note: https://accessibility.huit.harvard.edu/accessible-math
- Paper2Audio: https://www.paper2audio.com/

## Saved Material

No extra raw artifacts were saved in this pass.

Reason:

- the main sources are stable official docs or product pages
- the current value is in synthesis, not in archiving screenshots or copies
- later technical passes may justify saving API docs, sample PDFs, or comparison notes

## What This Changes About The Idea

### Validated strongly

- two-mode product shape is reasonable
- tablet-first desktop workflow is realistic
- patterned canvas backgrounds are normal and useful
- SVG export from vector-only content is a sound product idea
- recoloring is a real study/reading feature
- math-aware speech is directionally valid

### Strengthened

The idea is more justified as a personal tool than as a generic market thesis. Existing tools validate the parts, but none of the researched tools clearly combine them the way you want.

### Weakened or constrained

The biggest weak point is still PDF TTS quality on messy technical documents. The idea is not weakened in purpose, but it is constrained in what it can promise honestly.

## Realistic Approaches

- Treat the infinite canvas and PDF mode as distinct first-class modes.
- Keep the product promise strongest for well-structured text PDFs.
- Treat math-aware TTS as a differentiator, but assume imperfect extraction from real PDFs.
- Prefer an architecture where text extraction, highlighting geometry, and spoken output can be kept aligned explicitly.
- Keep export rules understandable: portable formats when possible, internal format only where necessary.

## Approaches To Avoid

- Avoid promising perfect TTS for arbitrary scanned or badly tagged PDFs.
- Avoid collapsing canvas mode and PDF mode into one blurry interaction model.
- Avoid making recoloring depend on destructive document editing only.
- Avoid assuming that "PDF annotations" and "editable annotations after export" are naturally the same thing.
- Avoid choosing a document engine later without considering licensing implications early.

## Risks That Matter For Planning

### 1. PDF reading order and structure

This is the main product-quality risk for TTS and guided highlighting.

### 2. Math extraction quality

Speaking math well depends on recovering structure, which PDFs often do poorly.

### 3. Annotation model complexity

There is a real risk of ending up with confusing differences between:

- temporary visual overlays
- editable note objects
- native PDF annotations
- flattened exported output

### 4. Licensing constraints

Some technically attractive stacks, especially MuPDF, impose licensing constraints that matter early.

### 5. Scope creep

The two-mode product is coherent, but it can still expand too far if it tries to become:

- a full whiteboard
- a full PDF office suite
- a complete accessibility platform

## Assumptions From IDEA.md To Revisit

- "math-heavy PDFs" should probably be framed as a priority workload, not a guaranteed perfect experience
- "minimal" should continue to mean low clutter and low resource use, not artificially few features
- "desktop-first" looks correct and well supported by the landscape
- "portable outputs" looks like a strong product principle worth keeping

## Open Questions

1. Should the product promise explicitly say "best on text-based PDFs" in a future `IDEA.md` revision?
2. Do you want the project to aim for native PDF annotations where possible, or is an overlay-based model acceptable if the user experience is better?
3. In the long run, is math TTS supposed to read equations literally, summarize them, or support both modes?
4. Is the infinite canvas expected to become a long-lived document format of its own, or mainly a workspace/export surface?
5. How important is fully offline/local TTS compared with using an external speech service later?

## Research Log

- Reviewed the current `IDEA.md` to extract the actual project claims.
- Surveyed existing desktop note/PDF tools to test whether the concept is redundant.
- Collected official documentation on PDF structure, stylus input, and math accessibility.
- Compared the idea against existing products that each cover only part of the workflow.
- Identified the main technical constraint as PDF structure quality, not canvas feasibility.
