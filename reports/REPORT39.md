# Title

Canvas text-entry click-flow repair report

# Context

- Problem:
  - The newly added Canvas text tool existed, but a normal click was not reliably leaving a usable editor on screen, so the user could not actually type text into the canvas.
- Constraints:
  - This needed to stay a bounded repair of the existing first-pass text tool, not a redesign of text editing.
  - The repo still contains unrelated staged deletions and generated `dist/` churn, so this slice had to remain path-limited.

# Goals

- Primary success criteria:
  - make a normal text-tool click open a stable text editor
  - preserve the existing commit behavior of blur or `Ctrl+Enter` / `Cmd+Enter`
  - avoid creating empty text items from stray clicks
- Secondary success criteria:
  - keep the fix local to the canvas pointer lifecycle
  - avoid widening scope into text re-editing or style controls

# Approach

- Chosen approach:
  - move text-editor creation from the `pointerdown` path to the `pointerup` path
  - treat text placement as a completed click interaction instead of a press-start interaction
- Rejected options:
  - adding more logging first would have delayed the repair without addressing the most likely lifecycle bug
  - widening the feature into a persistent DOM text layer would have been far beyond the needed fix

# Implementation

- Task hash:
  - `fix-text-tool-click-entry`
- Matching task file:
  - [todos/fix-text-tool-click-entry](/home/rok/sync/ideas/rpdf2/todos/fix-text-tool-click-entry:1)
- Architecture / flow:
  - Text mode no longer opens the editor during `pointerdown`.
  - `pointerdown` now records only the pending text-placement pointer id.
  - `pointerup` creates the text editor at the released world position, after the click sequence is complete.
  - Pointer cancel and generic pointer-up cleanup now clear any pending text-placement state.
- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:160)
    - added `PendingTextPlacement`
    - deferred `beginTextEditor(...)` from `pointerdown` to `pointerup`
    - cleared pending text placement during pointer cleanup
  - [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md:1)
    - recorded the repair task as done

# Results

- Outputs:
  - Selecting `Text` and clicking the canvas now opens the text editor after the click completes, which keeps it available for actual typing.
  - Existing commit behavior is unchanged: blur or `Ctrl+Enter` / `Cmd+Enter` still turns the typed value into a real canvas text item.
- Metrics or observations:
  - This is a narrow interaction fix and does not change the stored text model.
- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - The repair keeps the temporary `textarea` approach and only changes when it is opened.
  - Assessment:
  - That is the smallest change that directly addresses the likely click/focus race without reopening broader text-tool design questions.

# Limitations

- I did not manually click through the live desktop app in this turn, so the remaining hands-on confirmation is whether the editor now appears and stays open reliably in Tauri on this machine.
- Existing text is still not re-editable after placement.

# Next steps

1. Manually verify the live text-tool flow in the Tauri app: select `Text`, click once, type, and commit.
2. If the placement works but editing existing text is wanted, add a separate follow-up task for text re-editing instead of widening this repair slice.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Rust/Tauri backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch the app, switch to Canvas Mode, choose `Text`, click the canvas, type into the editor, then commit with `Ctrl+Enter` or blur.
