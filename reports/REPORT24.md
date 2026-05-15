# Title

Responsive layout hardening report

# Context

The selected worker slice was `harden-responsive-layout-under-window-resize` from `TODO.md`.

There is no `todos/` directory in this repo yet, so this turn used `TODO.md` plus the latest reports as the task source of truth.

This was the right next action because:

- it is a current priority-2 task with direct user-reported layout pain behind it
- it can be completed cleanly without touching the already-dirty canvas implementation file
- the issue is primarily structural CSS behavior under smaller windows, so a bounded stylesheet pass has high leverage

# Goals

- Primary success criteria:
  - keep the shell header from collapsing into an awkward stacked mess on smaller windows
  - keep canvas overlays usable when width and height are reduced
  - keep PDF Mode readable and contained at narrower viewport sizes

- Secondary success criteria:
  - keep the change CSS-only
  - avoid widening into behavior or state-management changes

# Approach

- Chosen approach:
  - harden the existing breakpoints instead of inventing a new layout system
  - add more explicit small-window behavior for header actions, canvas overlays, and PDF cards
  - turn cramped horizontal control groups into responsive grids or wrapped rows where needed

- Rejected options:
  - changing shell or workspace TypeScript would have widened a layout task into behavior changes
  - trying to solve this with one global scaling rule would have left the real collisions in place
  - touching the dirty `TODO.md` file for task-state cleanup would have broken commit isolation for this slice

# Implementation

- Task hash:
  - `harden-responsive-layout-under-window-resize`

- Architecture / flow:
  - The existing `max-width: 920px` rules were tightened so the workspace header fully stretches and the action area behaves like a stacked responsive control block rather than a half-flexed desktop row.
  - New `max-width: 760px` rules convert header action rows into a 2-column grid, move canvas overlays into safer positions, and let the canvas picker become a horizontal wrapped tool strip.
  - New `max-width: 560px` rules collapse action rows to one column, simplify the rail branding, and make the smallest layout more deliberate instead of just compressed.
  - PDF Mode now also gets smaller-window spacing reductions and a shorter page-stage minimum height.

- Key files or components:
  - `src/styles.css`
    - strengthened header and action-row responsiveness
    - added narrow-window canvas overlay rules
    - added smaller-window PDF layout and stage sizing adjustments

# Results

- Outputs:
  - The shell header now behaves more predictably under narrow widths.
  - Canvas Mode no longer relies on the right-edge vertical picker layout at small widths; it switches to a wrapped horizontal strip.
  - The stroke-width control and bottom toolbar get repositioned to avoid competing for the same corners.
  - PDF Mode gets tighter padding and a smaller page-stage floor for reduced windows.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - This slice stays CSS-only.
  - Assessment:
  - The reported issue is layout collapse, so CSS is the correct narrow fix surface.

- Fact:
  - Canvas controls now intentionally reflow instead of staying in their desktop positions.
  - Assessment:
  - Reflow is preferable to preserving the desktop arrangement when that arrangement becomes cramped and overlap-prone.

# Limitations

- Manual live-window resize verification in the running Tauri app was not performed in this turn.
- This slice does not solve deeper canvas interaction issues such as multi-select or shortcut behavior.
- `TODO.md` was not included in the commit because it already contains unrelated local edits, so the task index was not updated in this commit.

# Next steps

1. Manually resize the live app to confirm the new breakpoints feel correct in both Canvas and PDF Mode.
2. Implement `add-multi-select` as the next major canvas interaction foundation.
3. Implement `add-marquee-selection` after multi-select is in place.

# Reproducibility

1. Inspect:
   - `src/styles.css`
2. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch:
   - `npm run tauri dev`
4. Resize the window through narrow desktop and near-tablet widths in both modes and confirm the header, canvas overlays, and PDF layout stay usable without human rescue.
