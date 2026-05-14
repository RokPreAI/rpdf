# Title

Zoom-scaled stroke rendering report

# Context

The selected worker slice was `fix-zoom-scaled-stroke-rendering` from `TODO.md`.

This was the right next action because:

- it is a small, high-leverage Canvas task with direct user-visible impact
- it stays local to viewport rendering instead of widening into selection or input redesign
- the user had just confirmed resize behavior, so the next clean step was to fix the remaining zoom/render mismatch

# Goals

- Primary success criteria:
  - make freehand strokes visually scale with zoom
  - make shape outlines visually scale with zoom
  - avoid introducing unstable width jumps or exaggerated rendering

- Secondary success criteria:
  - keep the change minimal and local to the render path
  - avoid changing the pressure model, document schema, or interaction model

# Approach

- Chosen approach:
  - inspect how the canvas camera transform and `lineWidth` interact
  - remove the extra `/ camera.scale` compensation from vector rendering where the context is already scaled
  - keep constant-screen-width affordances like selection handles unchanged

- Rejected options:
  - adding new zoom-dependent width formulas would have hidden the real bug
  - changing saved stroke widths or pressure values would have widened scope unnecessarily
  - redesigning zoom behavior globally was not needed once the render-path mismatch was identified

# Implementation

- Architecture / flow:
  - Canvas vector rendering already applies `ctx.scale(camera.scale, camera.scale)`.
  - Stroke segments and shape outlines were still dividing `lineWidth` by `camera.scale`, which cancelled the visual zoom effect.
  - The fix removes that compensation only for actual vector content so on-screen stroke thickness now grows and shrinks with zoom as expected.

- Key files or components:
  - `src/features/canvas/workspace.ts`
    - removed `/ camera.scale` from freehand stroke segment rendering
    - removed `/ camera.scale` from shape outline rendering
  - `TODO.md`
    - marked `fix-zoom-scaled-stroke-rendering` done

# Results

- Outputs:
  - Zooming in now makes strokes and shape outlines appear thicker on screen.
  - Zooming out now makes thin marks appear thinner instead of staying visually bloated.
  - The fix stayed limited to viewport rendering behavior.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - The bug came from double-compensating line width after the camera transform was already applied.
  - Assessment:
  - Removing the compensation is the correct minimal fix because it restores the expected canvas transform behavior without changing stored geometry.

# Limitations

- Manual live-window verification of the zoom feel was not performed in this turn.
- This slice does not change constant-screen-size UI affordances such as selection handles, grid dots, or overlays.

# Next steps

1. Implement `add-multi-select` as the next selection foundation task.
2. Implement `add-marquee-selection` after multi-select is in place.
3. Revisit `fix-pdf-mode-annotations` only with live manual verification, since the code-side fix exists but the backlog still reflects user uncertainty.

# Reproducibility

1. Inspect:
   - `src/features/canvas/workspace.ts`
   - `TODO.md`
2. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch:
   - `npm run tauri dev`
4. In Canvas Mode, draw a thin stroke and a thin shape, zoom in and out with the mouse wheel, and confirm their on-screen thickness scales with the viewport zoom.
