# Title

Replace stroke-only history with action history

# Context

- Problem:
  - Canvas Mode only had a stroke-oriented `Ctrl+Z` path, so undo worked for the simplest drawing case but failed as a real editing history system.
  - The user explicitly called out broken undo for delete/restore and asked for redo plus broader action coverage.
- Constraints:
  - this slice needed to stay inside Canvas Mode
  - the repo already had unrelated local edits in [src/app/shell.ts](/home/rok/sync/ideas/rpdf2/src/app/shell.ts:1), [src/styles.css](/home/rok/sync/ideas/rpdf2/src/styles.css:1), and an untracked sync-conflict copy of `TODO.md`, so the commit had to avoid those

# Goals

- Primary success criteria:
  - replace the old stroke-only undo behavior with one coherent history model
  - add redo support
  - make history cover draw, delete, recolor, paste-image, resize, and text-content edits
- Secondary success criteria:
  - preserve current save/load behavior
  - keep the implementation simple enough to reason about and verify

# Approach

- Chosen approach:
  - switch the canvas workspace to action entries built from `before` / `after` document snapshots
  - restore snapshots through the existing canvas document import path
- Why this was the right next action:
  - it avoids one-off per-feature undo hacks
  - it naturally covers mixed item types
  - it reuses the existing document model and import/export logic instead of inventing a second partial state system
- Rejected option:
  - extending the old “remove last vector item” path would still have been fundamentally wrong for delete, resize, image paste, and text edits

# Implementation

- Task hash:
  - `replace-stroke-only-history-with-action-history`
- Matching task file:
  - [todos/replace-stroke-only-history-with-action-history](/home/rok/sync/ideas/rpdf2/todos/replace-stroke-only-history-with-action-history:1)
- Architecture / flow:
  - the canvas workspace now records `HistoryEntry` objects with `before` and `after` `WorkspaceDocumentSnapshot`s
  - undo restores the `before` snapshot and pushes the entry onto the redo stack
  - redo restores the `after` snapshot and pushes the entry back onto the undo stack
  - snapshot restore goes through a shared `applyCanvasSnapshot(...)` path, which is also now used by normal document import
- Action coverage added in this slice:
  - draw stroke
  - draw shape
  - move selection
  - resize selection
  - erase items
  - delete selection
  - recolor selected vector items
  - change selected vector stroke width
  - insert text
  - edit existing text
  - paste image
  - import PDF page onto canvas
- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:223)
    - toolbar shortcut hint now advertises redo
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:2256)
    - added snapshot cloning/signature helpers and history entry recording
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:4042)
    - added shared `applyCanvasSnapshot(...)` restore logic
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:4124)
    - added async `undoLastAction()` and `redoLastAction()`
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:3800)
    - replaced the old `Ctrl+Z` “remove last vector item” behavior with real undo/redo shortcuts

# Results

- Outputs:
  - Canvas Mode now has real undo/redo stacks instead of a stroke-pop shortcut
  - deleting an item can now be undone and redone
  - non-stroke edits such as text changes, image paste, recolor, resize, and move are now captured by history
- Verification:
  - `npm run build`
  - `cargo check --manifest-path src-tauri/Cargo.toml`

# Decisions

- Fact:
  - the task allowed either action-based history or an equivalent snapshot-based model
- Assessment:
  - per-action snapshot entries were the fastest correct route here because they cover every canvas item type with one consistent mechanism
- Fact:
  - the existing canvas import/export path already knew how to rebuild the full workspace state
- Assessment:
  - making history restore go through that path reduced duplicated state-rebuild logic

# Limitations

- I did not run a manual live-app interaction matrix for every covered action in this turn.
- The current history model is intentionally coarse: one committed edit action becomes one snapshot entry. It is not yet optimized for very long sessions or memory usage.
- Direct text-editor transient state while the editor is still open is not itself undoable until the text edit is committed into the canvas document.

# Next steps

1. Manually verify the target action set in the live app:
   - draw
   - delete selection
   - recolor selected vector item
   - paste image
   - resize item
   - edit text
   - undo and redo each one
2. Implement [todos/add-text-font-size-controls](/home/rok/sync/ideas/rpdf2/todos/add-text-font-size-controls:1) next, since text editing is now sitting on a broader history foundation that can track those edits correctly.

# Reproducibility

1. Inspect:
   - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:2256)
   - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:4042)
   - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:3800)
2. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
