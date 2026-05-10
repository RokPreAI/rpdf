# Title

Selection-aware SVG export eligibility report

# Context

After persistence and recovery were in place, `TODO.md` advanced to `add-svg-export-eligibility`. Canvas Mode already had richer editable state, but there was still no way to export vector work honestly. A naive “always export SVG” button would have been misleading because the canvas can also contain raster images.

The constraints were:

- keep the first export path honest about what SVG can and cannot represent
- reuse the new selection model when deciding what can be exported
- stay local to Canvas Mode instead of designing a larger export pipeline
- avoid pretending raster content can round-trip as clean SVG vector output

# Goals

- Primary success criteria:
  - add a visible SVG export control
  - export vector-eligible stroke content as SVG
  - disable export when the chosen content is not honestly representable as SVG

- Secondary success criteria:
  - make eligibility understandable in the UI
  - use selection state when possible so users can still export a vector stroke from an otherwise mixed canvas

# Approach

- Chosen approach:
  - Add a small export panel to the existing canvas settings area.
  - Compute export eligibility from the current canvas state:
    - selected stroke: eligible
    - selected image: not eligible
    - no selection + raster images present: not eligible
    - no selection + vector-only strokes: eligible
  - Generate a bounded SVG from stroke geometry and download it as a file from the browser side.

- Rejected options:
  - Exporting raster images into an “SVG” wrapper would satisfy the file extension but not the product’s honesty requirement.
  - Blocking export entirely whenever any image exists would ignore the new selection model and make mixed canvases unnecessarily rigid.
  - Building PDF/export unification in this cycle would widen scope beyond the current task.

# Implementation

- Architecture / flow:
  - `src/features/canvas/workspace.ts` now computes a live SVG export state from the current canvas document and selection state.
  - The export panel shows:
    - `Export SVG`
    - an eligibility/status message
  - When export is allowed:
    - the selected stroke or the whole vector-only stroke set is converted into SVG path markup
    - a blob URL is generated
    - the browser download path is triggered with a sensible filename

- Key files or components:
  - `src/features/canvas/workspace.ts`
  - `src/styles.css`
  - `TODO.md`

- Example:
  - If the canvas contains both strokes and pasted images, whole-canvas SVG export is disabled with an explicit message.
  - If the user selects one stroke from that same mixed canvas, SVG export becomes available again for that vector selection only.

# Results

- Outputs:
  - Canvas Mode now exposes an SVG export control.
  - Export eligibility is now selection-aware and explicit in the UI.
  - Vector stroke content can now be downloaded as SVG.
  - `TODO.md` now marks `add-svg-export-eligibility` as done and advances the current task to `add-draw-shapes`.

- Metrics or observations:
  - The export path reuses stroke geometry directly, so it does not need a new persistence or backend layer.
  - The export-state message reduces ambiguity by stating why export is currently blocked instead of silently disabling the button.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - The first SVG export path handles strokes only.
  - Assessment:
  - This is the correct honesty boundary because strokes are truly vector data in the current canvas model, while images are not.

- Fact:
  - Selection state now affects export eligibility.
  - Assessment:
  - This turns the earlier selection work into useful export behavior instead of leaving it isolated as an editing-only feature.

- Fact:
  - SVG generation happens in the frontend instead of going through Rust.
  - Assessment:
  - For this bounded task, the browser-side blob path is enough and avoids widening the storage/export backend unnecessarily.

# Limitations

- The exported SVG does not preserve pressure-varying width inside a single stroke; it uses the stored base width.
- Raster images remain non-exportable to SVG in this first pass.
- There is no export dialog or custom filename UI yet beyond the default download name.
- PDF-mode export remains a separate future concern.

# Next steps

1. Complete `add-draw-shapes` so the vector export path can include more than freehand strokes over time.
2. Complete `add-config-and-toolbar-icons` so the growing tool surface stays easier to scan.
3. Complete `add-tts-and-reliability-pipeline` so PDF Mode reaches the next major functional milestone.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Rust backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Inspect the SVG export implementation:
   - `src/features/canvas/workspace.ts`
4. Inspect the export panel styling:
   - `src/styles.css`
