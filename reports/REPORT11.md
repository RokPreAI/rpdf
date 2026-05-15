# Title

PDF page import and recolor workflow report

# Context

This was the last remaining unfinished task in the requested `0-5` priority band: `add-pdf-page-import-and-recolor`. The earlier cycles had already built the PDF shell, reading pipeline, selection-aware canvas editing, and a small settings path for recolor defaults, but the app still lacked the bridge that makes the two modes cooperate around actual study pages.

This was the correct final bounded worker slice because:

- it was the only remaining unfinished task in the requested band
- it connects PDF Mode and Canvas Mode around a core product behavior rather than adding isolated polish
- the repo already had the right boundary points for page rendering, recolor defaults, and canvas raster placement

# Goals

- Primary success criteria:
  - render real PDF page rasters through the backend
  - add recolor controls that affect the current PDF page view
  - allow the current PDF page to be imported into Canvas Mode as a placed item
  - persist imported PDF page metadata separately from generic pasted images

- Secondary success criteria:
  - keep the first recolor model simple and explicit
  - preserve the source PDF path, page index, and recolor settings used for imported pages
  - keep SVG export honest by treating imported PDF pages as raster content

# Approach

- Chosen approach:
  - Implement page raster generation with local `pdftoppm` in the Rust PDF engine boundary.
  - Keep recolor in the frontend for now by transforming the rendered page image locally.
  - Import the current page into Canvas Mode through an explicit shell event handoff instead of a hidden file-based bridge.
  - Add a dedicated `pdfPages` collection to the canvas document model so imported PDF pages remain distinguishable from ordinary pasted images.

- Rejected options:
  - Flattening imported PDF pages into generic `images` would have lost source-page and recolor metadata.
  - Requiring Pdfium-first rendering before finishing the task would have blocked the final `0-5` slice unnecessarily when the machine already has working local raster tools.
  - Driving page import through save/load files would have added friction and made the workflow feel artificial.

# Implementation

- Backend rendering:
  - `src-tauri/src/infrastructure/pdf_engine/mod.rs`
    - `render_page` now rasterizes the requested page with `pdftoppm`
    - reads the generated PNG back into memory
    - reports real image dimensions
    - base64-encodes the PNG payload for the frontend
  - backend status notes now mention that local page rendering is available through system tools while the Pdfium boundary remains the architectural target

- Canvas persistence and placement:
  - `src/app/types.ts`
    - added `CanvasPdfPagePlacementDocument`
    - extended `CanvasDocument` with `pdfPages`
  - `src-tauri/src/domain/canvas.rs`
    - added `CanvasPdfPagePlacement` and recolor metadata in the Rust persistence model
  - `src/features/canvas/workspace.ts`
    - added runtime support for imported PDF page items
    - imported pages now render, move, erase, select, save, and reload correctly
    - imported PDF pages correctly block full-canvas SVG export like other raster content
    - canvas now listens for shell-dispatched PDF page import events

- Cross-mode import flow:
  - `src/app/shell.ts`
    - now listens for PDF page import requests
    - switches to Canvas Mode when needed
    - forwards the import payload into the mounted canvas workspace

- PDF recolor and import UI:
  - `src/features/pdf/workspace.ts`
    - added recolor controls for enable/disable, foreground, and background
    - applies recolor locally to the rendered page image
    - keeps recolor state in the exported/imported PDF study document
    - added `Import page to canvas`
    - imports carry:
      - source PDF path
      - page index
      - rendered page asset
      - recolor settings used at import time

# Results

- Outputs:
  - PDF Mode now displays real rendered page rasters instead of only a placeholder path.
  - Recoloring works in PDF Mode for the current rendered page.
  - The current page can now be imported into Canvas Mode as a placed PDF page item.
  - Imported PDF page items now persist as a first-class canvas document type with their source metadata and recolor settings.
  - `TODO.md` now marks `add-pdf-page-import-and-recolor` done, which closes the requested `0-5` band.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - Imported PDF pages are stored separately from generic pasted images.
  - Assessment:
  - This preserves the product-specific meaning of imported study pages and keeps future page-specific behavior possible.

- Fact:
  - Recolor is applied locally in the frontend rather than in Rust.
  - Assessment:
  - This keeps the first recolor workflow simple and responsive while avoiding unnecessary backend image-processing expansion in the same cycle.

- Fact:
  - System tools are used for page raster generation right now.
  - Assessment:
  - This is a pragmatic implementation bridge that satisfies the feature while keeping the higher-level Pdfium architectural direction intact.

# Limitations

- Recolor currently uses a simple brightness-based remap rather than a more advanced document-aware palette transform.
- Imported PDF pages preserve the recolor settings used at import time, but there is not yet a dedicated post-import recolor editor inside Canvas Mode.
- Backend page rendering currently depends on local Poppler tools being available on the machine.

# Next steps

1. The requested `0-5` band is now complete.
2. The next natural phase is testing and review of the integrated app flows, especially PDF render/recolor/import/save-load behavior.
3. If later needed, add a canvas-side editor for selected imported PDF page recolor settings instead of only page-time import recolor.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Rust backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Inspect backend page rendering:
   - `src-tauri/src/infrastructure/pdf_engine/mod.rs`
4. Inspect PDF recolor/import UI:
   - `src/features/pdf/workspace.ts`
5. Inspect canvas PDF page placement support:
   - `src/features/canvas/workspace.ts`
6. Inspect the shell import handoff:
   - `src/app/shell.ts`
