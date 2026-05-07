# Title

rpdf app-module refactor report

# Context

This task implemented the current highest-priority backlog item: `refactor-app-into-modules`.

The project had already completed the priority `0-3` feature band, but most of the desktop behavior still lived in one large file: `src/app.rs`. That shape was directly at odds with the current `PLAN.md`, which now prioritizes long-term maintainability, clearer module ownership, and safer future work on OCR fallback, persistence, and service boundaries.

The work had to stay narrow. The goal was not to add features or redesign behavior, but to preserve the existing working prototype while breaking the flat app file into clearer ownership areas.

# Goals

- Remove `src/app.rs` as the single home for most product behavior.
- Introduce a real `src/app/` module layout that separates canvas logic, PDF logic, and shared helpers.
- Keep the completed priority `0-3` user-visible behaviors intact.
- Leave the codebase in a state where the next workers can split UI shells and service boundaries without first redoing this structural cleanup.
- Verify the refactor with `cargo check`.

# Approach

The safest approach was a behavior-preserving file split:

- keep `RpdfApp`, startup state, shell state, and top-level update flow in `src/app/mod.rs`
- move Infinite Canvas Mode behavior into `src/app/canvas.rs`
- move PDF Mode behavior into `src/app/pdf.rs`
- move shared rendering and utility helpers into `src/app/util.rs`

This was chosen instead of introducing new abstractions or changing the data model because the task was structural, not architectural in the deeper service-boundary sense. The immediate win was to establish real code ownership boundaries without risking unrelated regressions.

# Implementation

The refactor replaced the old flat `src/app.rs` layout with a directory-based module tree:

- `src/app/mod.rs`
  - defines `RpdfApp`
  - keeps the top-level egui update loop
  - owns startup and shell state definitions
  - keeps the shared annotation toolbar and summary panels
- `src/app/canvas.rs`
  - owns Infinite Canvas Mode rendering
  - owns canvas toolbars, drawing, panning, zooming, background rendering, item placement, recolor-on-imported-pages behavior, and SVG export handling
- `src/app/pdf.rs`
  - owns PDF Mode rendering
  - owns PDF toolbars, page navigation, annotation handling, recolor controls, TTS start/stop, and active reading-highlight updates
- `src/app/util.rs`
  - owns shared color conversion, pressure capture, stroke-point filtering, reading-span generation, SVG building, page geometry helpers, and small reusable UI helpers

The task also updated `TODO.md`:

- `refactor-app-into-modules` is now marked done
- `extract-mode-shells-and-shared-ui-state` is now the current task

# Results

The structural split completed successfully.

Observable outcomes:

- `src/app.rs` was removed
- `src/app/` now contains `mod.rs`, `canvas.rs`, `pdf.rs`, and `util.rs`
- the application still compiles after the split
- `TODO.md` now reflects the completed architecture task and the next current task

Verification:

- Ran `cargo check`
- Result: passed

There was one intermediate compile-fix round after the split:

- some helper functions were still being resolved through the old flat-file scope
- note handlers needed broader visibility from child modules back to the app root

Those were corrected and the final compile succeeded.

# Decisions

- Kept `RpdfApp` and state definitions in `src/app/mod.rs` instead of moving everything at once, because the immediate goal was ownership separation, not maximum granularity.
- Did not change the model layer during this task. The model contracts already existed and the task did not require redefining them.
- Did not introduce service traits yet. That remains a separate task so the refactor stays bounded and reviewable.
- Preserved the existing user-visible behavior and command flow instead of using the refactor as a pretext for feature changes.

# Limitations

- This is still not the final target structure from `PLAN.md`. The app now has clearer file boundaries, but not yet the fuller separation into UI shells and service interfaces.
- Canvas and PDF behavior are split into files, but some shared state still lives in broad app-level structs.
- No runtime interaction test was performed beyond successful compilation.
- The worktree still contains unrelated pre-existing deletions of `REPORT1.md` through `REPORT9.md`; those were intentionally left untouched.

# Next steps

1. Complete `extract-mode-shells-and-shared-ui-state`.
   This is the clean next continuation because the new file split makes it possible to separate Canvas Mode and PDF Mode UI ownership without first untangling a monolithic file.

2. Complete `formalize-reading-and-export-services`.
   This matters before OCR fallback and persistence because current reading/export behavior still needs explicit internal boundaries.

3. Then implement `add-text-fallback-and-warning-flow`.
   The codebase is now in a better shape to add OCR fallback without burying more logic directly inside UI control flow.

# Reproducibility

Working directory:

- `/home/rok/sync/ideas/rpdf`

Files changed for this task:

- `src/app/mod.rs`
- `src/app/canvas.rs`
- `src/app/pdf.rs`
- `src/app/util.rs`
- `src/app.rs` removed
- `TODO.md`

Verification command:

```bash
cargo check
```

Expected verification result:

- the crate compiles successfully in the default development profile
