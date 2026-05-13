# Source Notes

Retrieved on 2026-05-10.

This file is not the main research artifact. It is a compact stash of the external material most likely to matter again during planning and implementation.

## Existing tools

- Rnote
  - URL: https://rnote.flxzt.net/
  - Why it matters: close competitor for the infinite-canvas half of the idea.
  - Notes:
    - Stylus-first UI.
    - Pressure-sensitive input.
    - Infinite canvas.
    - PDF, bitmap, and SVG import.
    - Export to SVG/PDF/Xopp.
    - Native `.rnote` format is explicitly unstable.
    - GPL-3.0 project.

- Xournal++
  - URL: https://github.com/xournalpp/xournalpp
  - PDF guide: https://xournalpp.github.io/guide/pdfs/
  - Why it matters: close competitor for the PDF-annotation half of the idea.
  - Notes:
    - Strong pen support and PDF annotation.
    - PDF is treated as a background to a journal, not as a first-class document model.
    - Exported PDF annotations are not editable afterward.
    - Built-in PDF annotations are not preserved on export.
    - GPL-2.0 project.

- Paper2Audio
  - URL: https://www.paper2audio.com/about
  - Why it matters: proof that there is real user value in document-to-audio pipelines that handle math and structure better than plain screen-reader style reading.
  - Notes:
    - Explicitly targets research papers and complex documents.
    - Claims support for figures, tables, equations, and scanned documents.
    - Cloud product, not offline-first.

## PDF engines and bindings

- Poppler
  - URL: https://poppler.freedesktop.org/
  - Why it matters: mature open-source PDF rendering and extraction base.
  - Notes:
    - Latest stable release visible during research: `26.05.0`, released 2026-05-03.
    - Official docs exist for cpp/glib/qt5/qt6 APIs.
    - `pdftotext` remains actively maintained.

- poppler-rs
  - URL: https://docs.rs/poppler-rs/latest/poppler/
  - Why it matters: Rust-side access to Poppler concepts if the app stays Rust-first on the PDF side.
  - Notes:
    - Exposes pages, annotations, structure elements, text spans, signatures, permissions, and related PDF concepts.
    - MIT-licensed bindings, but still tied to Poppler system dependencies.

- MuPDF Core
  - URL: https://mupdf.com/core
  - Licensing: https://artifex.com/licensing
  - Why it matters: strong performance candidate for rendering, extraction, and annotation.
  - Notes:
    - Official positioning emphasizes small size, high-fidelity rendering, extraction, conversion, signing, and annotations.
    - Licensing is dual-track: AGPL or commercial. This is a real product/legal decision, not a footnote.

- pdfium-render
  - URL: https://docs.rs/pdfium-render
  - Why it matters: pragmatic Rust binding option if the project wants a Rust-managed PDF stack with runtime-bundled native libraries.
  - Notes:
    - Supports rendering, text/image extraction, page annotations, forms, links, and document creation.
    - Can bind at runtime to bundled or system Pdfium libraries.
    - Docs explicitly warn that Pdfium itself should be assumed not thread-safe.

## Math speech and reading support

- Speech Rule Engine
  - URL: https://speechruleengine.org/
  - Why it matters: mature math-to-speech logic for MathML/MathJax/LaTeX-adjacent flows.
  - Notes:
    - Browser and Node use are supported.
    - Claims full Mathspeak and Clearspeak rule sets.
    - Supports multiple locales.

- MathCAT
  - URL: https://nsoiffer.github.io/MathCAT/callers.html
  - Why it matters: more navigation-aware math speech/braille engine with explicit API surface.
  - Notes:
    - Takes MathML as input.
    - Returns spoken text and braille.
    - Supports navigation commands and node ids for synchronized highlighting.

## OCR and text reliability

- W3C PDF3 reading order guidance
  - URL: https://www.w3.org/WAI/WCAG21/Techniques/pdf/PDF3
  - Why it matters: confirms that correct reading order depends on tagged structure and often breaks on complex layouts.

- Tesseract quality guidance
  - URL: https://tesseract-ocr.github.io/tessdoc/ImproveQuality.html
  - Why it matters: OCR is viable but fragile; accuracy depends heavily on preprocessing and segmentation.

- Tesseract input-format limits
  - URL: https://tesseract-ocr.github.io/tessdoc/InputFormats.html
  - Why it matters: Tesseract does not read PDF directly, so any OCR fallback needs a PDF-to-image stage first.

## Input and interaction

- PointerEvent pressure
  - URL: https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent/pressure
  - Why it matters: confirms that browser-side pen pressure is broadly available, which supports the current Tauri + web canvas direction for stylus input.
