# Title

Canvas text-entry commit control repair report

# Context

- Problem:
  - The canvas text tool could open a text editor correctly, but it finalized unpredictably while the user was still typing.
  - The task spec suspected an overly eager accept path rather than a placement problem.
- Constraints:
  - This needed to stay a narrow text-entry lifecycle fix, not a broader re-edit or rich-text redesign.
  - The repo already had unrelated uncommitted UI work in `src/app/shell.ts`, `src/styles.css`, `TODO.md`, and generated `dist/` files, so the commit needed to stay scoped to the text-entry slice.

# Goals

- Primary success criteria:
  - stop text from being committed just because focus changes or typing pauses
  - make text placement happen only under explicit user control
  - support `Esc` as the explicit accept key requested by the user
- Secondary success criteria:
  - keep empty text sessions fail-closed
  - preserve the existing `Ctrl+Enter` explicit accept path

# Approach

- Chosen approach:
  - inspect the current text-editor lifecycle and remove implicit commit on `blur`
  - keep the editor alive across accidental focus loss by refocusing it unless the session is explicitly completing
  - treat `Esc` as the primary explicit commit action, while leaving `Ctrl+Enter` as an additional explicit confirm path
- Root cause:
  - the editor was created correctly, but it had a one-shot `blur` listener that immediately called `commitTextEditor()`
  - in a desktop-webview drawing workflow, small focus transitions are common, so blur-based commit made the tool feel random

# Implementation

- Task hash:
  - `fix-text-tool-commit-control`
- Matching task file:
  - [todos/fix-text-tool-commit-control](/home/rok/sync/ideas/rpdf2/todos/fix-text-tool-commit-control:1)
- Architecture / flow:
  - `TextEditorSession` now tracks a small completion-intent state so blur handling can distinguish between a real explicit finish and an accidental focus change.
  - `commitTextEditor()` marks the session as committing before teardown.
  - `cancelTextEditor()` marks the session as canceling before teardown.
  - `beginTextEditor()` no longer commits on plain blur. Instead, if blur happens without an explicit completion intent, the editor is refocused on the next tick so the editing session stays active.
  - `Esc` now explicitly commits the current text item, matching the requested interaction model.
- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:2927)
    - added `completionIntent` to the active text editor session
    - changed `Esc` from cancel to explicit commit
    - replaced blur-driven commit with blur refocus unless the editor is intentionally completing

# Results

- Outputs:
  - Text entry no longer finalizes just because the textarea briefly loses focus.
  - `Esc` now commits the current text to the canvas.
  - `Ctrl+Enter` still commits explicitly.
  - Empty text sessions still create no item.
- Verification:
  - Ran `npm run build`
    - result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - result: success

# Decisions

- Fact:
  - The old failure mode came from blur semantics, not text rendering or canvas placement.
- Assessment:
  - Replacing implicit blur commit with explicit accept is the smallest correct fix because it changes only the editor lifecycle and not the text model.
- Fact:
  - The user explicitly asked for `Esc` to be the key that accepts text.
- Assessment:
  - Matching that request directly is better than inventing a separate hidden accept/cancel split for this slice.

# Limitations

- I did not run live manual interaction testing in the desktop app in this turn, so the remaining hands-on confirmation is whether the refocus behavior feels right while clicking around the canvas and toolbar.
- This slice does not yet add re-editing of existing text items. That remains the separate queued task [todos/add-text-item-reedit](/home/rok/sync/ideas/rpdf2/todos/add-text-item-reedit:1).
- `TODO.md` was not updated in this commit because it already contains unrelated local backlog edits.

# Next steps

1. Manually test the text tool in the running app.
   - Place text, pause typing, click around, and verify it does not finalize until `Esc` or `Ctrl+Enter`.
2. Take the follow-up text task, [todos/add-text-item-reedit](/home/rok/sync/ideas/rpdf2/todos/add-text-item-reedit:1).
   - The new explicit editor lifecycle is the right base for reopening existing text items.
3. If the forced refocus feels too rigid, add a tiny follow-up task for an explicit cancel gesture.
   - That should be decided from live feel rather than guessed in this slice.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Tauri/Rust backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch the app, switch to Canvas Mode, select the `Text` tool, click on the canvas, type, then verify:
   - pausing does not commit
   - blur does not silently commit
   - `Esc` commits
   - `Ctrl+Enter` commits
