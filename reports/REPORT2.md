# Title

Canvas pressure sensitivity and stroke-width control report

# Context

After the architecture foundation cycle, `TODO.md` selected `add-tablet-pressure-and-stroke-width` as the next current task. The canvas already supported drawing, panning, erasing, paste-image, and zoom, but every stroke used a fixed width and ignored pen pressure.

The constraints were:

- keep the task local to Canvas Mode
- preserve mouse drawing behavior
- avoid widening into save/load, selection, or PDF work
- verify the change with the narrowest useful checks

# Goals

- Primary success criteria:
  - make stroke rendering respond to pointer pressure on supported pen devices
  - allow manual control of base stroke width
  - keep mouse input working with stable non-pressure behavior

- Secondary success criteria:
  - keep the canvas workspace buildable after the earlier shell split
  - avoid backend changes for this frontend-only task

# Approach

- Chosen approach:
  - Extend the canvas stroke model from plain points to pressure-aware stroke points.
  - Store a per-stroke `baseWidth` and apply pressure during rendering on a segment-by-segment basis.
  - Add a lightweight stroke-width range control in the canvas workspace so width can still be controlled when the input device does not provide useful pressure values.

- Rejected options:
  - Keeping a fixed-width stroke and only changing a label or setting would not satisfy the pressure requirement.
  - Rebuilding the stroke renderer around a more advanced brush engine would be too large for this task.
  - Pushing pressure handling into the backend would violate the current browser-side pen interaction design.

# Implementation

- Architecture / flow:
  - `src/features/canvas/workspace.ts` now defines pressure-aware `StrokePoint` samples.
  - Each stroke stores:
    - `points`, where each point includes `x`, `y`, and `pressure`
    - `color`
    - `baseWidth`
  - The draw path now renders stroke segments individually, adjusting line width from the average pressure of adjacent points.
  - Single-point strokes render as filled circles so taps still leave visible marks.

- Key files or components:
  - `src/features/canvas/workspace.ts`
  - `src/styles.css`
  - `TODO.md`

- Example:
  - Pen input now uses `PointerEvent.pressure` when available.
  - Mouse input is normalized to fixed pressure `1`, so it still draws at the selected width without pretending to be a pressure device.
  - The canvas settings panel now exposes a `Stroke width` slider with a live pixel label.

# Results

- Outputs:
  - Canvas strokes now carry pressure samples.
  - Rendering width now changes with pen pressure on supported devices.
  - Users can change base stroke width through the new range control.
  - `TODO.md` now records `add-tablet-pressure-and-stroke-width` as done.

- Metrics or observations:
  - No backend files were changed in this cycle.
  - Rust `cargo check` still succeeds because the change stayed frontend-local.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success, with the same existing dead-code warnings from the architecture-foundation cycle

# Decisions

- Fact:
  - Pressure is clamped and normalized rather than used raw.
  - Assessment:
  - This avoids invisible or unusably thin strokes when devices report very low or zero pressure values during contact changes.

- Fact:
  - Mouse input is treated as fixed pressure `1`.
  - Assessment:
  - This keeps mouse drawing predictable and avoids misrepresenting it as a pressure-capable device.

- Fact:
  - Stroke width is controlled by a simple range input in the canvas UI.
  - Assessment:
  - This is enough to satisfy the current requirement without creating a larger settings subsystem.

# Limitations

- Pressure behavior was verified by build checks, not by testing on a real drawing tablet in this cycle.
- Stroke rendering still uses a simple segment-based approach, not a sophisticated brush/smoothing engine.
- Stroke width is not yet persisted through save/load because save/load is still a separate pending task.

# Next steps

1. Complete `build-pdf-mode-shell` so the PDF workspace becomes more than a contract placeholder and gains real document-opening and navigation scaffolding.
2. Complete `add-save-load-project-files` so the newer canvas stroke model and base-width data are preserved across sessions.
3. Complete `add-selection-and-move-tools` so the canvas can support later export targeting and editing workflows cleanly.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Rust backend still compiles after the frontend-only change:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Inspect the pressure implementation:
   - `src/features/canvas/workspace.ts`
4. Inspect the UI styling for the stroke-width control:
   - `src/styles.css`
