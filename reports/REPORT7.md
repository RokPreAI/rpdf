# Title

Selection-aware SVG export added to the rpdf canvas

# Context

- Problem:
  The current task in `TODO.md` was `add-selection-aware-svg-export`. The canvas already had mixed content and real annotation items, but there was no way to export vector-compatible work or to distinguish whole-canvas export from vector-only selection export.
- Constraints:
  The task had to stay inside the canvas export problem. It could not widen into persistence, PDF export, or general item editing. It also needed to honor the specification rule that incompatible targets such as imported images or imported PDF pages must clearly refuse SVG export instead of silently degrading.

# Goals

- Primary success criteria:
  Support SVG export for compatible canvas targets, default to whole-canvas export when nothing is selected, and switch to selection-based export when specific items are selected.
- Secondary success criteria:
  Make export eligibility visible in the UI and keep the selection state grounded in the existing model layer.

# Approach

- Chosen approach:
  Added item-selection controls to the canvas toolbar, tracked selection through `SelectionTarget`, and implemented a direct SVG writer for vector-compatible canvas items. The export path field and status message stay in canvas interaction state so later workers can extend the behavior without redefining export state.
- Rejected options:
  Did not add a general item-manipulation system in this task because export only needs target selection, not full editing. Did not attempt partial export of incompatible item types because the specification explicitly calls for clear refusal instead of misleading partial SVG output.

# Implementation

- Architecture / flow:
  The canvas toolbar now lets the user choose whole-canvas export or item-based export. `selected_canvas_items` resolves the current target, `export_canvas_svg` enforces compatibility checks, and `build_svg_document` serializes strokes and text items into an SVG file when the target is eligible.
- Key files or components:
  - `src/app.rs`: added selection controls, export-path UI, export status handling, compatibility checks, and the SVG writer.
  - `TODO.md`: advanced task state after completion.
  - `SUBTODO.md`: cleared after the task finished.
- Example:
  If the current target contains an imported image or imported PDF page, `export_canvas_svg` sets a clear in-UI failure message instead of writing a misleading SVG file.

# Results

- Outputs:
  Updated:
  - `src/app.rs`
  - `TODO.md`
  - `SUBTODO.md`
- Metrics or observations:
  The canvas now supports:
  - explicit whole-canvas export targeting
  - item-based selection targeting
  - SVG export for stroke/text-only targets
  - clear refusal messages for incompatible targets
  - writing SVG files to a user-specified path
- Verification:
  Ran `cargo check` successfully after the export changes.

# Decisions

- Tradeoffs made:
  - Kept selection simple and checkbox-based because the task needed export targeting, not a full spatial selection system.
  - Limited SVG output to strokes and text because those are the model-defined vector-compatible canvas items.
  - Stored export status in the canvas interaction state so the failure reason stays visible at the point of action.

# Limitations

- Known issues, uncertainties, or risks:
  - SVG output uses a simple fixed world view box rather than a tight content-derived bounding box.
  - Spatial click-selection of items is still not implemented.
  - Export verification in this task is compile-level only; an example SVG file was not generated in this run.
  - Imported images and PDF pages are deliberately ineligible for SVG export rather than converted.

# Next steps

1. Implement `add-pdf-recolor-and-annotation-visibility` because PDF Mode now has enough annotation behavior for recoloring and palette control to be meaningful.
2. Implement `add-pdf-tts-and-highlight-modes` after recoloring so the PDF workspace can begin to exercise reading-support state instead of only document/navigation state.

# Reproducibility

1. From `/home/rok/sync/ideas/rpdf`, inspect `src/app.rs` to review selection targeting and SVG export logic.
2. Verify compilation with `cargo check`.
