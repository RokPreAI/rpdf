# Title

PDF annotation page-frame visibility report

# Context

The selected worker slice was `fix-pdf-mode-annotations` from `TODO.md`.

This was the right next action because:

- it is still the highest-priority remaining backlog item
- it stays local to PDF Mode and avoids the already-dirty canvas file
- the earlier overlay alignment fix improved geometry, but the stage still depended on a loose full-area overlay instead of a stable page box

# Goals

- Primary success criteria:
  - make the annotation layer visibly and structurally sit on the rendered PDF page
  - avoid the overlay depending on incidental full-stage sizing
  - keep pointer drawing and page rendering intact

- Secondary success criteria:
  - keep the fix bounded to the PDF workspace and its styles
  - avoid widening into persistence or annotation-model redesign

# Approach

- Chosen approach:
  - introduce an explicit page frame inside the PDF stage
  - size and position that frame from the actual rendered page box
  - move both the PDF image and the annotation canvas into that frame so they share the same geometry

- Rejected options:
  - another overlay-only tweak on the full stage would have left the annotation layer structurally detached from the visible page
  - changing the saved PDF annotation schema would have widened the slice beyond a visibility/anchoring fix
  - touching the dirty canvas file to work around the issue would have broken commit isolation

# Implementation

- Architecture / flow:
  - The PDF stage now contains a dedicated `page frame` wrapper instead of placing the image and annotation canvas directly against the whole stage.
  - The workspace computes the displayed page bounds from the stage size and the rendered page dimensions.
  - That box is applied to the page frame, and the annotation canvas is then sized from the same frame dimensions.
  - The stage shell now also has a real minimum height, so the page area does not depend on absolutely positioned children to exist.

- Key files or components:
  - `src/features/pdf/workspace.ts`
    - added `pdf-page-frame`
    - switched annotation sizing to drive the page frame and canvas together
    - used rendered page dimensions as a fallback when the image has not finished loading yet
  - `src/styles.css`
    - added page-frame styling
    - gave the PDF stage shell a stable minimum height
    - changed the page image to fill the explicit frame instead of using whole-stage `object-fit: contain`

# Results

- Outputs:
  - The PDF page and annotation canvas now share one explicit positioned frame.
  - Annotation visibility is no longer tied to a full-stage overlay that can drift or collapse independently.
  - The viewer has a more stable page area even before the image fully loads.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - The fix introduces a page-frame container instead of continuing to rely on a stage-wide overlay.
  - Assessment:
  - This is the more correct structural model because annotations belong to the rendered page box, not to the whole viewer panel.

- Fact:
  - The page frame uses rendered-page dimensions as a fallback before image natural size is available.
  - Assessment:
  - This keeps the annotation surface from collapsing during the load boundary.

# Limitations

- Manual live-window verification of drawing on top of a real PDF page was not performed in this turn.
- This slice does not redesign annotation coordinates for cross-resize normalization.
- `TODO.md` was left out of the commit because it already contains unrelated local backlog edits.

# Next steps

1. Manually test PDF annotation drawing in the live app to confirm this closes the user-reported visibility issue.
2. Implement `add-multi-select` as the next major canvas interaction foundation.
3. Implement `add-marquee-selection` after multi-select is in place.

# Reproducibility

1. Inspect:
   - `src/features/pdf/workspace.ts`
   - `src/styles.css`
2. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch:
   - `npm run tauri dev`
4. In PDF Mode, open a PDF and draw on the page stage. Confirm the strokes now appear inside the same positioned page frame as the rendered PDF image.
