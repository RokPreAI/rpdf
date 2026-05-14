# Title

Mode-switch restore copy removal report

# Context

- Problem:
  - The shell showed `Restored canvas workspace state after mode switch` and the matching PDF variant every time the user switched modes.
  - The user explicitly wanted that copy removed while keeping the actual workspace-state restore behavior.
- Constraints:
  - This needed to stay a narrow UX cleanup, not a broader mode-switch rewrite.
  - The repo already had unrelated staged deletions and generated build-output churn, so this worker slice had to stay path-limited.

# Goals

- Primary success criteria:
  - stop emitting the mode-switch restore status message
  - keep mode-switch state restoration working
  - keep real error and recovery messages visible
- Secondary success criteria:
  - avoid touching unrelated shell status surfaces
  - preserve the existing worker-loop backlog flow by promoting the next task cleanly

# Approach

- Chosen approach:
  - remove only the success-status write from `restoreModeState()` and keep the import path intact
  - clean up the now-unused render-request bookkeeping so the shell stays minimal
- Rejected options:
  - suppressing all backend-status updates would have hidden real save/load/recovery feedback
  - rewriting the mode-switch flow would have widened this slice unnecessarily

# Implementation

- Task hash:
  - `remove-mode-switch-restore-copy`
- Matching task file:
  - [todos/remove-mode-switch-restore-copy](/home/rok/sync/ideas/rpdf2/todos/remove-mode-switch-restore-copy:1)
- Architecture / flow:
  - `renderMode()` still snapshots the active workspace before switching and still mounts/imports the target workspace snapshot after the mode change.
  - `restoreModeState()` now performs only the import of the cached snapshot for the target mode.
  - The previous post-import shell status assignment was removed, so successful mode switches stay visually quiet.
- Key files or components:
  - [src/app/shell.ts](/home/rok/sync/ideas/rpdf2/src/app/shell.ts:144)
    - removed the restore-success status copy
    - removed the now-unused render request id bookkeeping
  - [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md:1)
    - marked `remove-mode-switch-restore-copy` done
    - promoted `add-text-tool` to current
  - [todos/remove-mode-switch-restore-copy](/home/rok/sync/ideas/rpdf2/todos/remove-mode-switch-restore-copy:1)
    - marked done
  - [todos/add-text-tool](/home/rok/sync/ideas/rpdf2/todos/add-text-tool:1)
    - marked current

# Results

- Outputs:
  - Switching between Canvas and PDF mode no longer writes the restore-status message into the shell.
  - The actual mode-state import path remains in place.
- Metrics or observations:
  - This was a shell-only implementation slice with no backend changes.
  - The only functional code change was in the mode-switch restore path.
- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - The success message was removed without changing autosave restore, save/load failure, or backend bootstrap status handling.
  - Assessment:
  - This preserves useful trust surfaces while removing the user-reported noise.
- Fact:
  - The now-unused render request id bookkeeping was removed instead of being left as dead state.
  - Assessment:
  - This keeps the shell implementation honest and avoids carrying inert coordination code after the UX change.

# Limitations

- I did not manually click through live mode switches in the desktop app during this turn, so the remaining runtime confirmation is human verification that state still restores cleanly in both directions.
- This slice intentionally does not change any other shell copy or backend-status behavior beyond the mode-switch restore success message.

# Next steps

1. Implement `add-text-tool` because it is now the current backlog item and is the next user-visible Canvas capability.
2. Manually switch between Canvas and PDF mode once to confirm the restore behavior still works and that no restore-copy message appears.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Rust/Tauri backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch the app and switch between Canvas and PDF modes.
4. Confirm the workspace state still restores but the shell no longer shows `Restored ... workspace state after mode switch`.
