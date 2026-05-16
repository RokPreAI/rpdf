# Title

Canvas text item re-edit support report

# Context

- Problem:
  - The canvas text tool could create persistent text items, but existing text could not be edited after placement.
  - The backlog required reopening an editor prefilled with the current text so users could modify, append, or delete text in place.
- Constraints:
  - This needed to reuse the explicit editor lifecycle from the previous `fix-text-tool-commit-control` slice instead of creating a second inconsistent editing path.
  - The repo still has unrelated local changes in `src/app/shell.ts`, `src/styles.css`, `TODO.md`, `.gitignore`, and generated `dist/` churn, so the commit needed to stay path-limited.

# Goals

- Primary success criteria:
  - allow an existing text item to re-enter edit mode
  - prefill the editor with the current text content
  - update the original item in place on accept instead of creating a duplicate
  - define empty-save behavior without corrupting canvas state
- Secondary success criteria:
  - preserve selection and positioning behavior
  - keep the implementation local to the canvas workspace

# Approach

- Chosen approach:
  - add a dedicated re-edit entry gesture in `Select` mode: double-click a text item
  - extend the active text editor session so it can either create a new text item or update an existing one
  - reuse the same inline textarea, commit rules, and placement coordinates for both new text and edited text
- Rejected option:
  - introducing a separate side-panel or modal text editor would have widened the scope and broken the direct on-canvas editing model already established

# Implementation

- Task hash:
  - `add-text-item-reedit`
- Matching task file:
  - [todos/add-text-item-reedit](/home/rok/sync/ideas/rpdf2/todos/add-text-item-reedit:1)
- Architecture / flow:
  - `TextEditorSession` now tracks an optional `existingTextId`.
  - New text placement still opens the inline editor in the same way as before.
  - In `Select` mode, double-clicking a text item now reopens that same editor with the current text prefilled.
  - On commit:
    - if the session is editing an existing item and the value is non-empty, the original text item is updated in place
    - if the session is editing an existing item and the value is empty, the original item is removed
    - if the session is creating a new item, the existing create-new flow still runs
- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:136)
    - extended `TextEditorSession` with `existingTextId`
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:2931)
    - updated `commitTextEditor()` so it can update or remove an existing text item instead of always creating a new one
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:2980)
    - generalized `beginTextEditor()` so it can open for either a new placement or an existing item
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:3032)
    - added a helper to reopen the editor for a selected text target
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:3392)
    - added double-click handling in `Select` mode to enter text re-edit

# Results

- Outputs:
  - Existing text items can now be edited in place.
  - Re-edit uses the same inline canvas textarea as new text creation, so the interaction stays consistent.
  - Emptying an existing text item and accepting removes it cleanly instead of leaving a broken blank item.
- Verification:
  - Ran `npm run build`
    - result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - result: success

# Decisions

- Fact:
  - The last text slice already established explicit editor commit rules and stable focus behavior.
- Assessment:
  - Reusing that exact editor path was safer than layering a second edit UI on top of text items.
- Fact:
  - The task file did not prescribe a specific gesture for re-edit entry.
- Assessment:
  - Double-click in `Select` mode is a bounded, discoverable choice that does not interfere with normal drag-to-move behavior.

# Limitations

- I did not manually test the live desktop interaction in this turn, so the remaining hands-on confirmation is whether double-click timing feels natural in the actual Tauri app.
- This slice does not add a visible hint that double-click edits text; that can be a later UX refinement if needed.
- `TODO.md` was not updated in this commit because it already contains unrelated local backlog edits.

# Next steps

1. Manually test text re-edit in the running app.
   - Double-click a text item in `Select` mode, edit it, then accept and confirm the original item updates in place.
2. If the gesture feels good, take the next general canvas task, [todos/extend-eraser-to-images](/home/rok/sync/ideas/rpdf2/todos/extend-eraser-to-images:1), or whichever task you want to prioritize next from the queue.
3. If users do not discover double-click editing easily, add a small follow-up task for a visible affordance or shortcut hint.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Tauri/Rust backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch the app, switch to Canvas Mode, create a text item, then verify:
   - double-clicking the text in `Select` mode reopens the editor
   - the previous text is already filled in
   - accepting updates the existing item in place
   - accepting empty text removes the item cleanly
