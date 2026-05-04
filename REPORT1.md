# Title

Core document models established for rpdf foundation task

# Context

- Problem:
  `rpdf` had product-definition artifacts (`IDEA.md`, `SPEC.md`, `TODO.md`) but no code. The current selected task in `TODO.md` was `define-document-models`, which required a stable editable representation for canvas content, PDF-session state, annotation state, recoloring state, reading-support state, selection targets, and SVG-export eligibility before UI or rendering work begins.
- Constraints:
  The task had to stay narrow and avoid architecture drift. It needed to preserve the specification boundary between editable working state, autosave/recovery state, and user-facing export behavior. The repository is currently almost empty, so the smallest honest implementation had to introduce the initial Rust crate and only the model layer needed by this task.

# Goals

- Primary success criteria:
  Create a single documented Rust model layer that covers the object boundaries required by `SPEC.md` and the current `TODO.md` task detail.
- Secondary success criteria:
  Make the new model code compile cleanly and leave the repository ready for the next foundation task without forcing a renderer, UI toolkit, or storage engine.

# Approach

- Chosen approach:
  Added a minimal Rust library crate and placed the model definitions under `src/model/`. The types were organized around product behavior rather than implementation technology: canvas documents, PDF sessions, canvas items, annotations, selections, reading-support state, recoloring state, autosave state, and export-target eligibility.
- Rejected options:
  Did not start with UI shell code because that belongs to the next task. Did not add serialization, persistence, renderer bindings, or PDF-engine-specific types because they would over-constrain the project before the app shell and feature slices exist.

# Implementation

- Architecture / flow:
  The new code defines two top-level editable workspace types: `CanvasDocument` for Infinite Canvas Mode and `PdfDocumentSession` for PDF Mode. Shared supporting types model geometry, colors, annotation appearance, selections, imported assets, reading support, recoloring, and export eligibility.
- Key files or components:
  - `Cargo.toml`: creates the initial Rust crate for the project.
  - `src/lib.rs`: exposes the model module.
  - `src/model/mod.rs`: contains the model layer.
  - `src/model/README.md`: explains the boundary between editable state, autosave/recovery state, and user-facing export behavior.
- Example:
  `CanvasItem::svg_compatibility()` classifies strokes and typed text as SVG-compatible and marks raster images or imported PDF pages as incompatible. This gives later export code a direct, model-level rule instead of re-deriving that decision ad hoc.

# Results

- Outputs:
  Added the first implementation files to the repo:
  - `Cargo.toml`
  - `src/lib.rs`
  - `src/model/mod.rs`
  - `src/model/README.md`
  Also updated task-tracking files:
  - `TODO.md`
  - `SUBTODO.md`
- Metrics or observations:
  The model layer now explicitly represents:
  - canvas items for pen strokes, typed text, imported images, and imported PDF pages
  - PDF annotation and reading-support state
  - recoloring settings and annotation palettes for normal vs recolored viewing
  - selection scope and SVG-export eligibility
  - autosave/recovery state separate from export state
- Verification:
  Ran `cargo check` in `/home/rok/sync/ideas/rpdf` and it completed successfully.

# Decisions

- Tradeoffs made:
  - Introduced a Rust crate now because there was no existing code location to hold the model layer.
  - Kept the models plain and renderer-agnostic so later tasks can attach UI, PDF, OCR, and TTS behavior without rewriting the document boundary.
  - Added a small adjacent README for model-boundary notes instead of expanding `SPEC.md` further, because the distinction between editable state and export/recovery state needed to live next to the code.

# Limitations

- Known issues, uncertainties, or risks:
  - The model layer is compile-verified only; no runtime behavior exists yet.
  - No serialization or persistence format has been defined yet.
  - SVG-export eligibility is modeled, but export execution is not implemented.
  - Reading-support state is represented, but no native-text, OCR, or TTS pipeline exists yet.
  - The next UI-shell task may expose small model gaps once real interaction surfaces are added.

# Next steps

1. Implement `bootstrap-desktop-app-shell` because the model layer now exists and the project needs a minimal desktop host with separate canvas and PDF workspaces.
2. Implement `implement-canvas-pen-and-viewport` after the app shell, because the canvas is the fastest high-value workflow to make visible and testable.

# Reproducibility

1. From `/home/rok/sync/ideas/rpdf`, inspect the new model layer with `sed -n '1,260p' src/model/mod.rs`.
2. Verify compilation with `cargo check`.
