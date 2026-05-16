# Title

Fix canvas text editor autosave commit

# Context

- Problem:
  - The text tool still appeared to auto-confirm text after a few seconds even though the earlier blur-based commit logic had been changed.
  - The new backlog task traced that symptom to the snapshot path rather than the editor blur path.
- Constraints:
  - The fix needed to stay narrow and avoid reopening the broader text-tool workflow.
  - The repo still had unrelated local changes in `.gitignore`, `src/app/shell.ts`, `src/styles.css`, generated `dist/`, and a stray root `REPORT1.md`, so this slice needed to commit only the task files and the real implementation fix.

# Goals

- Primary success criteria:
  - stop autosave and other snapshot/export flows from force-committing the active text editor
  - keep explicit commit actions like `Esc` and `Ctrl+Enter` working
- Secondary success criteria:
  - keep the fix architectural and local instead of adding more timing or blur exceptions

# Approach

- Chosen approach:
  - inspect the call path from autosave to workspace snapshot export
  - remove the side effect from canvas snapshot export so serialization is read-only
- Why this was the right next action:
  - the task file already identified the likely culprit: shell autosave calls `activeWorkspace.exportDocument()`, and canvas export was still calling `commitTextEditor()`
  - that explained the "about after three seconds" symptom far better than the blur logic did
- Rejected option:
  - adding more editor-side timers, focus guards, or blur exceptions would have treated the symptom instead of the real cause

# Implementation

- Task hash:
  - `fix-text-editor-autosave-commit`
- Matching task file:
  - [todos/fix-text-editor-autosave-commit](/home/rok/sync/ideas/rpdf2/todos/fix-text-editor-autosave-commit:1)
- Architecture / flow:
  - I confirmed that shell autosave snapshots call `activeWorkspace.exportDocument()` on an interval.
  - I confirmed that Canvas Mode `exportDocument()` still unconditionally called `commitTextEditor()`.
  - I removed that call so snapshot/export is side-effect free and no longer converts transient editing into committed canvas content.
- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:3900)
    - removed the unconditional `commitTextEditor()` from `exportDocument()`
  - [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md:1)
    - moved the task from current to done
  - [todos/fix-text-editor-autosave-commit](/home/rok/sync/ideas/rpdf2/todos/fix-text-editor-autosave-commit:1)
    - marked the task file done

# Results

- Outputs:
  - autosave and other snapshot/export callers no longer force-confirm the current text editor
  - explicit editor commit behavior remains in the editor key handling and outside-click pointer path
- Verification:
  - `npm run build`
  - `cargo check --manifest-path src-tauri/Cargo.toml`

# Decisions

- Fact:
  - the root cause was not a three-second idle timeout inside the text editor
- Assessment:
  - it was a snapshot side effect: shell autosave or any snapshot/export flow could finalize the editor by calling canvas `exportDocument()`
- Fact:
  - snapshot/export code should be serialization-only
- Assessment:
  - removing `commitTextEditor()` from export is the narrowest fix and avoids future hidden editor state changes from autosave or persistence paths

# Limitations

- I did not runtime-test the live desktop app with a stopwatch and wait for the autosave interval in this turn.
- In-progress text editing is still transient state and is not serialized into snapshots; this fix prevents implicit commit, but it does not add persistence for an open editor.

# Next steps

1. Manually verify the text tool in the running app:
   - open a text editor
   - type some text
   - wait longer than the autosave interval
   - confirm the editor stays open and the text is not placed until explicit accept or outside-click commit
2. If the user wants open editors to survive autosave or mode switches, add a separate task for serializing transient text-editor state instead of committed text items.

# Reproducibility

1. Inspect the autosave call path in [src/app/shell.ts](/home/rok/sync/ideas/rpdf2/src/app/shell.ts:232) and [src/app/shell.ts](/home/rok/sync/ideas/rpdf2/src/app/shell.ts:494).
2. Inspect the canvas export path in [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:3900).
3. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
