# Title

Canvas controls compaction and header-export report

# Context

The selected worker slice was the pending canvas UI compaction work already present in the working tree: remove the oversized canvas settings block, move SVG export into the shell header, and make each shape its own direct tool button.

This was the right next action because:

- the user explicitly requested this UI change sequence
- the changes already formed one coherent bounded slice in the working tree
- committing them cleanly now avoids mixing these frontend edits into a later unrelated worker task

Constraints for this slice:

- keep the work local to canvas UI and shell wiring
- do not widen into resize handles, shortcuts, or deeper export backend changes
- preserve the existing buildable app behavior while changing only how the controls are surfaced

# Goals

- Primary success criteria:
  - remove the large `canvas-settings` panel
  - keep only a compact stroke-width slider overlay on the canvas
  - move `Export SVG` into the shared header
  - split shape drawing into separate rectangle, ellipse, and line buttons inside `.canvas-pickers`

- Secondary success criteria:
  - keep SVG export availability synced with Canvas Mode state
  - preserve existing shape drawing behavior with the new picker model
  - verify the result with the standard frontend and Rust checks

# Approach

- Chosen approach:
  - replace the canvas settings panel with a standalone `stroke-width-control`
  - route SVG export through shell-level events so the header owns the button while the canvas workspace still owns export eligibility and execution
  - expand the `Tool` union so each supported shape is its own explicit tool instead of relying on a second shape-type control

- Rejected options:
  - keeping a reduced settings card would still waste overlay space compared with a single slider control
  - moving export entirely into the shell business logic would over-couple the shell to canvas export details
  - keeping one generic shape tool plus another selector would preserve the extra step the user wanted removed

# Implementation

- Architecture / flow:
  - Canvas Mode now renders only the stroke-width slider as its top-left overlay control.
  - The shell header now includes a Canvas-only `Export SVG` button.
  - The canvas workspace publishes export eligibility through `rpdf:canvas-svg-export-state`.
  - The shell listens for that state and enables/disables the header export button accordingly.
  - Clicking the header button dispatches `rpdf:request-canvas-svg-export`, which the canvas workspace handles by running the existing export logic.
  - Shape tools are now explicit tool modes: `rectangle`, `ellipse`, and `line`.

- Key files or components:
  - `src/features/canvas/workspace.ts`
    - removed the settings panel markup
    - removed the shape dropdown
    - removed the local SVG export button and status surface
    - added separate shape picker buttons
    - expanded the tool model to direct shape tools
    - added shell-facing SVG export state and request events
  - `src/app/shell.ts`
    - added the Canvas-only header `Export SVG` button
    - added Canvas Mode visibility and enabled-state wiring for that button
  - `src/styles.css`
    - removed `.canvas-settings`, shape dropdown, and local SVG export panel styles
    - restyled the stroke-width slider as a compact standalone overlay

# Results

- Outputs:
  - The bulky canvas settings block is gone.
  - The canvas keeps only a small stroke-width slider overlay.
  - `Export SVG` now lives in the shared header and appears only in Canvas Mode.
  - Rectangle, ellipse, and line are now direct picker buttons instead of a second-step shape selection flow.

- Metrics or observations:
  - The slice stayed entirely in frontend TypeScript and CSS.
  - Existing SVG export behavior was preserved; only the control surface moved.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success
  - Manual runtime verification of the new header export button and direct shape-button flow was not performed in this turn.

# Decisions

- Fact:
  - SVG export remains owned by the canvas workspace even though the button moved into the shell header.
  - Assessment:
  - This keeps the export logic local to the canvas feature while still giving the user the header-level control surface they asked for.

- Fact:
  - Each shape is now represented as its own tool mode.
  - Assessment:
  - This removes the extra shape-selection step and makes the picker behavior more direct.

- Fact:
  - The old canvas settings panel was fully removed instead of partially reduced.
  - Assessment:
  - This matches the user’s goal of reclaiming overlay space rather than cosmetically shrinking redundant UI.

# Limitations

- This slice does not change the actual SVG export file-target behavior; it only relocates the export control.
- The picker column is now longer because shapes each have their own button, so further density adjustments may still be useful later.
- Manual runtime validation is still needed for header export enable/disable behavior after selection changes and mode switches.

# Next steps

1. Implement `fix-recolor-controls-layout-and-state` as the next tight PDF/UX cleanup slice.
2. Implement `add-element-resize` now that selection and direct tool selection are in better shape.
3. Revisit `harden-svg-export-and-save-path` if the user wants export to choose an explicit filesystem save target instead of the current browser-style download flow.

# Reproducibility

1. Inspect the UI changes:
   - `src/features/canvas/workspace.ts`
   - `src/app/shell.ts`
   - `src/styles.css`
2. Build the frontend:
   - `npm run build`
3. Check the Rust side:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
4. Launch the app:
   - `npm run tauri dev`
5. In Canvas Mode, verify that only the stroke-width slider remains as an overlay, that shape buttons are direct tools, and that `Export SVG` appears in the header only for Canvas Mode.
