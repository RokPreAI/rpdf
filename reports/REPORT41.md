# Title

Canvas pressure-sensitivity toggle report

# Context

- Problem:
  - Pressure-aware drawing already existed, but it was always on for pen input and the stroke-width slider did not clearly behave as the maximum width under full pressure.
  - The user wanted direct control in the compact stroke-width overlay: a toggle for pressure sensitivity, with the selected stroke width representing 100 percent width at maximum pressure.
- Constraints:
  - This needed to stay a bounded canvas-input slice.
  - The repo still contains unrelated staged deletions and generated `dist/` churn, so the commit had to stay path-limited.

# Goals

- Primary success criteria:
  - add a pressure-sensitivity toggle beside the stroke-width slider
  - persist that toggle in the existing canvas preferences
  - make disabled pressure produce constant-width strokes
  - make enabled pressure map full pressure to the selected stroke width
- Secondary success criteria:
  - keep selection-style stroke width editing consistent with the existing model
  - avoid widening into retroactive stroke migration or pressure-model redesign

# Approach

- Chosen approach:
  - extend local canvas preferences with a `pressureSensitivityEnabled` flag
  - add a compact checkbox control beside the width slider
  - gate the point-pressure sampling path so disabled pressure records `1` for new stroke points
- Rejected options:
  - rewriting stored stroke semantics for older strokes would have widened the slice unnecessarily
  - introducing a separate stroke-width model for pressure and non-pressure modes would have complicated selection editing without clear benefit

# Implementation

- Task hash:
  - `add-pressure-sensitivity-toggle`
- Matching task file:
  - [todos/add-pressure-sensitivity-toggle](/home/rok/sync/ideas/rpdf2/todos/add-pressure-sensitivity-toggle:1)
- Architecture / flow:
  - Canvas preferences now include `pressureSensitivityEnabled`, defaulting to `true`.
  - The stroke-width control now shows a `Pressure` checkbox inline with the width slider.
  - When pressure sensitivity is disabled, new stroke points record pressure `1`, so rendered width remains constant at the selected stroke width.
  - When pressure sensitivity is enabled, the existing render path still multiplies width by point pressure, so full pressure reaches 100 percent of the selected width.
- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:206)
    - added the inline pressure toggle UI
    - persisted `pressureSensitivityEnabled` in local canvas preferences
    - synchronized the toggle with the existing control setup path
    - updated pointer-pressure sampling so disabled pressure records full-width points
  - [src/styles.css](/home/rok/sync/ideas/rpdf2/src/styles.css:289)
    - added compact inline layout styling for the slider-plus-toggle row
  - [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md:1)
    - marked `add-pressure-sensitivity-toggle` done
  - [todos/add-pressure-sensitivity-toggle](/home/rok/sync/ideas/rpdf2/todos/add-pressure-sensitivity-toggle:1)
    - marked done

# Results

- Outputs:
  - Canvas Mode now exposes a `Pressure` toggle next to the stroke-width slider.
  - Turning pressure off makes new strokes render at a constant selected width.
  - Turning pressure on preserves pressure-responsive width, with full pressure reaching the selected width.
  - The toggle persists through the existing local preference storage.
- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - The toggle changes only new stroke sampling, not the stored pressure data of old strokes.
  - Assessment:
  - That keeps the behavior understandable and matches the task note to avoid retroactive stroke rewriting unless it is truly needed later.

# Limitations

- I did not manually test the live pen experience in the running desktop app in this turn, so the remaining manual confirmation is how the toggle feels with a real pressure-capable device.
- Existing strokes retain their previously stored point pressures; the toggle is not a retroactive reinterpretation switch.

# Next steps

1. Manually verify pen input with pressure on and off, especially that full pressure reaches the selected width and low pressure stays below it.
2. If the user wants pressure behavior to be editable for existing strokes later, add a separate follow-up task instead of widening this slice retroactively.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Rust/Tauri backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch the app, switch to Canvas Mode, draw with `Pressure` on and off, and confirm the selected stroke width acts as the maximum width under full pressure.
