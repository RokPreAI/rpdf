# Title

PDF viewport containment report

# Context

The selected worker slice was `contain-pdf-within-viewport`. The user reported that PDF Mode could escape the visible application viewport, especially when the window was reduced in size.

This was the right next action because:

- it was the top unfinished priority-1 PDF task in `TODO.md`
- `REPORT14.md` called it out as the next PDF follow-up after mode-switch preservation
- it was a bounded layout regression that could be fixed locally without reopening PDF rendering, reading, or canvas interaction logic

Constraints for this slice:

- keep the change narrow and layout-focused
- do not mix in the separate `fix-recolor-controls-layout-and-state` task
- do not claim full completion of the broader human-verified responsive-layout task from static inspection alone

# Goals

- Primary success criteria:
  - keep the PDF viewer surface inside the visible workspace bounds
  - stop the PDF stage from forcing the whole layout taller than the available viewport
  - preserve predictable internal scrolling for long sidebar content

- Secondary success criteria:
  - keep the fix CSS-only if possible
  - preserve current PDF rendering and annotation logic
  - verify the change with the normal frontend and Rust build checks

# Approach

- Chosen approach:
  - tighten the PDF workspace sizing chain with explicit `min-width: 0` and `min-height: 0` on the containing layout nodes
  - move overflow responsibility inward so the PDF sidebar can scroll instead of forcing the whole app surface to overflow
  - remove the hard 420px stage minimum so the viewer can shrink with the window
  - add height-based padding reductions for shorter windows so cards and previews do not consume the whole viewport before the viewer area is measured

- Rejected options:
  - changing PDF render dimensions or adding zoom behavior would have widened scope far beyond containment
  - patching the problem in TypeScript would not address the actual CSS sizing chain that caused the overflow
  - treating this as the full responsive-layout task would be dishonest because the user already marked that broader task as needing human review

# Implementation

- Architecture / flow:
  - The workspace root now clips overflow instead of allowing PDF Mode to spill past the shell bounds.
  - The PDF layout grid now has explicit `min-height: 0` and `overflow: hidden`, which lets the viewer and sidebar measure against the real available height.
  - The PDF sidebar now scrolls internally when its cards exceed the available height.
  - The PDF stage no longer enforces a fixed 420px minimum height, so the viewer can shrink instead of pushing the whole PDF workspace out of view.
  - Shorter windows now get reduced padding and smaller text-preview height so the viewer keeps usable space.

- Key files or components:
  - `src/styles.css`
    - tightened `.workspace-root`
    - tightened `.pdf-workspace`, `.pdf-layout`, `.pdf-sidebar`, and `.pdf-viewer-panel`
    - removed the hard minimum stage height from `.pdf-stage`
    - added height-based media queries for shorter windows

# Results

- Outputs:
  - PDF Mode now keeps overflow inside the workspace instead of letting the whole layout escape the app viewport.
  - The sidebar can scroll independently when there is not enough vertical space.
  - The stage can shrink with the window instead of forcing a too-tall viewer area.

- Metrics or observations:
  - The fix stayed CSS-only.
  - No PDF rendering, annotation, or backend code paths changed in this slice.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success
  - Manual runtime verification in a live resized Tauri window was not performed in this turn, so the task remains partially human-verified by design.

# Decisions

- Fact:
  - The hard fixed stage minimum was removed.
  - Assessment:
  - That minimum was the main reason the viewer could demand more height than a small window could provide.

- Fact:
  - Sidebar overflow is now local to the sidebar instead of the whole PDF workspace.
  - Assessment:
  - This keeps the document viewer contained while still allowing access to the controls in shorter windows.

- Fact:
  - The fix stayed in CSS.
  - Assessment:
  - The underlying regression was layout containment, not broken PDF state or rendering logic.

# Limitations

- This slice does not complete the broader `harden-responsive-layout-under-window-resize` task. It only fixes the direct PDF viewport containment issue.
- Full human runtime confirmation at multiple window sizes is still needed.
- The separate recolor-controls layout bug remains open and was intentionally not mixed into this task.

# Next steps

1. Implement `fix-select-tool` because it is the other remaining priority-1 interaction issue.
2. Update `TODO.md` to mark `fix-pdf-mode-annotations` done if manual runtime testing confirms the earlier overlay fix is now behaving correctly.
3. Implement `fix-recolor-controls-layout-and-state` as the next tight PDF/UX cleanup slice.

# Reproducibility

1. Inspect the containment changes:
   - `src/styles.css`
2. Build the frontend:
   - `npm run build`
3. Check the Rust side:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
4. Launch the app:
   - `npm run tauri dev`
5. Open a PDF, reduce the window height and width, and confirm the viewer stays inside the app while the sidebar scrolls internally.
