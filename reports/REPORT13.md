# Title

PDF annotation overlay alignment report

# Context

The selected worker slice was `fix-pdf-mode-annotations`. The user reported that they could not see annotations in PDF Mode even though the annotation surface and pointer handlers already existed in the code.

This was the right next task because:

- it is still a priority-1 backlog item
- it is isolated to the PDF workspace rather than the already-dirty shell and global layout files
- it addresses a core promised PDF-mode behavior instead of secondary polish

The likely failure mode was not missing annotation data structures. The workspace already had:

- a page-scoped `annotationsByPage` store
- pointer handlers on the annotation canvas
- redraw logic
- import/export support for annotation strokes

The weakness was the overlay geometry. The annotation canvas was stretched to the full stage bounds, while the visible PDF page image used `object-fit: contain`. That made the overlay depend on loose stacking and stage sizing instead of being explicitly anchored to the actual displayed page box.

# Goals

- Primary success criteria:
  - make the annotation layer visibly sit over the rendered PDF page
  - keep pointer interaction and redraw behavior intact
  - keep the fix local to PDF Mode

- Secondary success criteria:
  - avoid editing the already-dirty global stylesheet
  - preserve import/export behavior for existing PDF annotation data
  - verify the change with frontend and Rust builds

# Approach

- Chosen approach:
  - Keep the existing annotation stroke model and pointer flow.
  - Move the fix into `src/features/pdf/workspace.ts` by explicitly controlling the annotation canvas geometry and stacking at runtime.
  - Compute the annotation bounds from the displayed image box rather than from the full stage rectangle.
  - Re-sync the overlay when the rendered image loads and when the window resizes.

- Rejected options:
  - A stylesheet-only fix would have required touching `src/styles.css`, which already had unrelated uncommitted changes and would have made a scoped worker commit harder.
  - Rewriting annotation storage or converting the stage to a different rendering model would have widened scope far beyond one bounded slice.
  - Folding this into viewport or mode-persistence work would have mixed separate regressions into one commit.

# Implementation

- Architecture / flow:
  - The PDF page image remains the visual base layer.
  - The annotation canvas is now explicitly treated as a top overlay layer.
  - The workspace computes the actual displayed image bounds from:
    - the stage rectangle
    - the image natural size
    - the contain-scale factor
  - The annotation canvas is then positioned and resized to those bounds instead of blindly filling the whole stage.

- Key files or components:
  - `src/features/pdf/workspace.ts`
    - sets runtime `z-index` values so the canvas is always above the page image
    - adds `currentAnnotationBounds()` to calculate the real visible PDF image box
    - updates `resizeAnnotationLayer()` to:
      - use displayed-image dimensions
      - place the overlay with `left` and `top`
      - hide the annotation layer when there is no active rendered page
    - listens for the page image `load` event so overlay sizing is corrected after the image resource resolves
    - removes that listener during workspace teardown

- Example:
  - If the PDF stage is larger than the visible page because of `object-fit: contain`, the annotation layer no longer spans the whole padded stage area. It now tracks just the actual displayed page region, so drawn strokes appear where the user expects them.

# Results

- Outputs:
  - The PDF annotation layer is now explicitly stacked above the page image.
  - The overlay is now aligned to the displayed page bounds rather than the entire stage shell.
  - The annotation layer is hidden when there is no rendered page instead of remaining a full inactive stage-sized surface.

- Metrics or observations:
  - The fix stayed entirely within the PDF workspace TypeScript module.
  - No Rust, shell, canvas, or global-style changes were needed for this slice.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success
  - Manual runtime drawing validation in the launched app was not performed in this turn.

# Decisions

- Fact:
  - The fix uses runtime geometry calculation in the workspace instead of a stylesheet-only overlay rule.
  - Assessment:
  - This gives the overlay direct knowledge of the rendered image size, which is the part that actually matters for `object-fit: contain`.

- Fact:
  - The global stylesheet was intentionally left untouched.
  - Assessment:
  - That kept the worker commit isolated from unrelated existing edits in `src/styles.css`.

- Fact:
  - Existing annotation persistence was preserved unchanged.
  - Assessment:
  - The bug was in display anchoring, not in the saved data model, so changing persistence would have been unnecessary churn.

# Limitations

- The fix is build-verified, but not yet manually verified by drawing on a live PDF page in the running Tauri app.
- This slice does not address the separate `contain-pdf-within-viewport` or `harden-responsive-layout-under-window-resize` tasks, even though those layout issues may still influence the overall PDF experience.
- Existing saved annotations assume the current page-space coordinate system. This slice improves overlay placement but does not redesign annotation coordinates for future zoom-aware normalization.

# Next steps

1. Implement `preserve-mode-state-across-switches` because mode resets still break the normal two-mode workflow more broadly than any single PDF-view issue.
2. Implement `contain-pdf-within-viewport` because PDF layout containment is still a separate reported regression after annotation visibility.
3. Manually test annotation drawing in the live app to confirm the overlay is now visible and aligned on real pages.

# Reproducibility

1. Inspect the PDF annotation workspace logic:
   - `src/features/pdf/workspace.ts`
2. Build the frontend:
   - `npm run build`
3. Check the Rust side:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
4. Launch the app in development and open a PDF:
   - `npm run tauri dev`
5. Draw on the PDF page stage and verify the strokes appear directly over the displayed page image.
