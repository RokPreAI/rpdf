# Title

Canvas shape tools and vector persistence report

# Context

After selection, autosave, and SVG export were in place, the next bounded `0-5` task in `TODO.md` was `add-draw-shapes`. Canvas Mode could already handle freehand strokes and pasted images, but it still lacked any geometric authoring even though selection and vector export were now mature enough to support it.

This was the right next slice because:

- the task was explicitly marked as the current TODO
- it stayed local to Canvas Mode and the existing versioned document model
- it could reuse the recent selection and SVG-export work instead of creating a parallel editing path
- it unlocked a broader class of vector canvas content without widening into the PDF reading pipeline

# Goals

- Primary success criteria:
  - add a minimal but useful set of shape tools
  - persist shapes inside saved canvas documents
  - make shapes participate in selection, movement, erase, and SVG export

- Secondary success criteria:
  - keep the first shape set bounded
  - preserve honest vector-export behavior when mixed with raster images
  - avoid breaking older saved canvas documents that do not yet contain shapes

# Approach

- Chosen approach:
  - Add a dedicated `shape` tool to Canvas Mode and keep the first shape set minimal: rectangle, ellipse, and line.
  - Extend the canvas document model with explicit shape records instead of trying to encode shapes as fake strokes.
  - Preserve vector drawing order with per-item `order` metadata so strokes and shapes export and render in the same sequence they were created.
  - Reuse the existing selection and export surfaces rather than introducing a separate shape subsystem.

- Rejected options:
  - Converting shapes to dense freehand stroke point lists would have made persistence and selection less honest, not simpler.
  - Adding many shape variations in this cycle would have widened scope beyond the stated “basic shapes” goal.
  - Keeping shapes frontend-only would have broken save/load integrity because the Rust persistence schema would not match the new document structure.

# Implementation

- Architecture / flow:
  - `src/features/canvas/workspace.ts`
    - added the `shape` tool and a shape-kind selector
    - introduced runtime `Shape` records for `line`, `rectangle`, and `ellipse`
    - added live shape preview during drag creation
    - extended selection, move, erase, undo, clear, and SVG export flows to include shapes
    - added vector ordering so mixed strokes and shapes render/export consistently
  - `src/app/types.ts`
    - added `CanvasShapeDocument` and `CanvasShapeKindDocument`
    - extended `CanvasDocument` with a `shapes` collection
    - added optional `order` metadata for persisted vector items
  - `src-tauri/src/domain/canvas.rs`
    - extended the Rust canvas schema with `CanvasShape`
    - added `serde(default)` compatibility for `shapes` and `order` so older files still deserialize cleanly
  - `src/styles.css`
    - added styling for the shape selector and disabled export button state

- Behavior:
  - Users can switch to the `H` shape tool, choose rectangle/ellipse/line, and drag to place a shape.
  - Shapes can be selected and moved with the existing selection tool.
  - Shapes are now saved and restored with canvas project files.
  - SVG export now accepts shape selections and full vector canvases that include shapes.
  - Raster images still block full-canvas SVG export unless a vector item is selected, which preserves the existing honesty rule.

# Results

- Outputs:
  - Canvas Mode now supports three basic geometric shapes.
  - The canvas persistence model now carries shapes explicitly.
  - Shapes now participate in selection and vector export eligibility checks.
  - `TODO.md` now marks `add-draw-shapes` done and advances the current task to `add-tts-and-reliability-pipeline`.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - Shapes were added as first-class document items instead of encoded stroke approximations.
  - Assessment:
  - This keeps persistence, selection, and later export behavior honest and easier to extend.

- Fact:
  - The first shape set is limited to line, rectangle, and ellipse.
  - Assessment:
  - This satisfies the task requirement without turning the cycle into a broad drawing-suite redesign.

- Fact:
  - Rust persistence was updated in the same cycle.
  - Assessment:
  - This avoids a dangerous frontend/backend schema mismatch for saved canvas documents.

# Limitations

- There are still no resize handles or post-creation shape editing controls beyond moving the whole shape.
- Erasing shapes removes them as whole vector items rather than partially trimming them.
- Shape SVG export uses geometric primitives and fixed width, not pressure-like variable width.
- The broader reading pipeline task for PDF Mode remains open and is still the highest-priority unfinished item in the requested band.

# Next steps

1. Complete `add-tts-and-reliability-pipeline` because it is the only remaining priority-2 task and the next major functional gap in PDF Mode.
2. Complete `add-pdf-page-import-and-recolor` so canvas and PDF workflows start to share page-level study artifacts.
3. Complete `add-config-and-toolbar-icons` so the growing Canvas/PDF tool surfaces remain easier to scan.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Rust backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Inspect the canvas implementation:
   - `src/features/canvas/workspace.ts`
4. Inspect the shared canvas types:
   - `src/app/types.ts`
5. Inspect the Rust canvas persistence model:
   - `src-tauri/src/domain/canvas.rs`
