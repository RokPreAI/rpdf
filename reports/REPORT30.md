# Title

Input quality control report

# Context

The selected worker slice was `add-input-polling-rate-setting` from [todos/add-input-polling-rate-setting](/home/rok/sync/ideas/rpdf2/todos/add-input-polling-rate-setting).

This was the right next action because:

- it was the current priority-2 canvas task in [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md)
- the user had already identified visible jaggedness in freehand strokes
- it was a bounded drawing-quality improvement that could be implemented locally in the canvas capture path

# Goals

- Primary success criteria:
  - expose a user-adjustable drawing-input quality control
  - make that control change real stroke capture behavior
  - persist the setting so it survives reloads

- Secondary success criteria:
  - preserve existing pen, mouse, and pressure-sensitive drawing
  - avoid widening the task into a full brush-engine redesign

# Approach

- Chosen approach:
  - add a compact `Input quality` range control beside the existing stroke-width control
  - persist the chosen level in the shared canvas preferences store
  - map the level into capture behavior by combining coalesced pointer events with interpolation spacing during pen strokes

- Rejected options:
  - a fake “polling rate” label with no actual capture-path effect would not satisfy the task
  - rewriting freehand rendering into a spline brush system would have widened the scope too much for this worker slice

# Implementation

- Task hash:
  - `add-input-polling-rate-setting`

- Architecture / flow:
  - Expanded the compact canvas control card to include a second persisted range input for `Input quality`.
  - Extended the local preferences schema so the canvas can load and save `defaultInputQuality` alongside stroke width, default shape, and default color.
  - Added stroke-capture helpers that:
    - use coalesced pointer events at medium-to-high quality levels when available
    - interpolate intermediate stroke points based on a quality-dependent spacing target
    - collapse tiny movements by replacing the most recent point instead of always appending a new one
  - Updated the pen pointer-move path to route through the new quality-aware stroke sampling helper.

- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts)
    - added persisted `defaultInputQuality`
    - added the `Input quality` control
    - added coalesced-event and interpolation-based stroke sampling helpers
    - routed pen capture through the quality-aware sampler
  - [src/styles.css](/home/rok/sync/ideas/rpdf2/src/styles.css)
    - added compact field styling for the expanded stroke control card
  - [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md)
    - marked `add-input-polling-rate-setting` done
    - promoted `harden-svg-export-and-save-path` to current
  - [todos/add-input-polling-rate-setting](/home/rok/sync/ideas/rpdf2/todos/add-input-polling-rate-setting)
    - marked done
  - [todos/harden-svg-export-and-save-path](/home/rok/sync/ideas/rpdf2/todos/harden-svg-export-and-save-path)
    - marked current

# Results

- Outputs:
  - Canvas Mode now has a visible `Input quality` slider with a persisted `1/5` to `5/5` level.
  - Higher quality levels can capture smoother strokes by using coalesced input samples and tighter interpolation spacing.
  - Lower quality levels intentionally keep a looser, sparser capture path.
  - The new setting survives reloads because it is stored in the same preference record as the existing canvas defaults.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - The user-facing control is named `Input quality` rather than a literal Hz polling rate.
  - Assessment:
  - This is the more honest label because the implementation improves capture using event coalescing and interpolation quality rather than direct hardware polling control.

- Fact:
  - Quality changes are applied in the stroke-capture path, not only in rendering.
  - Assessment:
  - This ensures the setting has a real effect on stored stroke geometry instead of only changing the visual preview.

# Limitations

- Manual drawing validation across multiple tablets and input devices was not performed in this turn.
- This pass improves point sampling density and interpolation, but it is not a full smoothing/Bezier brush engine.
- The exact feel of each level may still need tuning after hands-on testing.

# Next steps

1. Implement `harden-svg-export-and-save-path` and verify export behavior against the newer selection flows.
2. Revisit keyboard shortcuts after export behavior is stable.
3. If manual drawing still feels rough, tune the quality-level spacing curve or add a second smoothing-specific control rather than overloading this one further.

# Reproducibility

1. Open Canvas Mode.
2. Draw a freehand stroke with `Input quality` at `1/5`.
3. Increase `Input quality` to `5/5`.
4. Draw another freehand stroke and confirm the stored point capture path reacts differently.
5. Reload the app and confirm the chosen quality level persists.
6. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
