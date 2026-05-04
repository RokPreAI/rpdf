# Model Boundary Notes

This directory defines the editable document and state model for `rpdf`.

The current boundary is:

- editable working state:
  - `CanvasDocument`
  - `PdfDocumentSession`
  - their items, selections, annotations, reading-support state, and view state
- autosave and recovery state:
  - represented only by `AutosaveState`
  - this is internal application state and not a user-facing export format
- user-facing export state:
  - represented by `ExportTarget`
  - includes export scope and SVG eligibility
  - remains separate from autosave/recovery concerns

Important distinctions:

- recoloring can be a temporary view setting or an export choice
- SVG eligibility is a property of the chosen export target, not of the whole project
- imported PDF pages and raster images remain valid editable content even when they make SVG export unavailable for a given target
- reading support reliability is explicit so later TTS and OCR work cannot silently pretend that weak text extraction is trustworthy
