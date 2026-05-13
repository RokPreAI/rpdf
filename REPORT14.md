# Title

Mode-switch workspace preservation report

# Context

The selected worker slice was `preserve-mode-state-across-switches`. The user reported that switching between Canvas Mode and PDF Mode reset both workspaces: the canvas became blank again and the PDF session was lost, forcing the user to retype paths and rebuild context.

This was the highest-leverage next action from the latest report because:

- it was the top priority item in the latest `REPORT13.md` next steps
- it broke the core promise that the app has two usable first-class modes
- both workspace modules already supported `exportDocument()` and `importDocument()`, so the missing piece was shell-level continuity rather than a redesign of canvas or PDF models

Constraints that shaped the fix:

- keep the change local to the app shell and existing workspace contracts
- avoid broad persistence redesign or extra backend work
- do not bundle the unrelated pre-existing shell cleanup edits into the worker commit if they could be staged separately
- preserve the current save/load and autosave model rather than turning a mode switch into an implicit file write

# Goals

- Primary success criteria:
  - switching away from Canvas Mode and back should preserve the current canvas document in memory
  - switching away from PDF Mode and back should preserve the current PDF study session in memory
  - the header save/load path field should preserve its per-mode value across switches

- Secondary success criteria:
  - keep the implementation inside the existing shell/workspace API
  - preserve compatibility with autosave and explicit load flows
  - verify the result with frontend and Rust builds

# Approach

- Chosen approach:
  - Add one in-memory snapshot cache per mode in the shell.
  - Capture the active workspace snapshot before the shell destroys a mode during a switch.
  - Restore the cached snapshot after the target workspace mounts.
  - Track a separate header file-path value for each mode so save/load paths do not get overwritten across switches.
  - Use a render request identifier so an older async restore cannot overwrite a newer mode mount.

- Rejected options:
  - Saving to disk on every mode switch would have been too heavy and would blur the line between temporary workspace continuity and explicit persistence.
  - Rebuilding the state model around a single merged document would conflict with the project’s two-mode architecture.
  - Pushing the fix into the individual workspaces would duplicate shell-level concerns and make mode transitions harder to reason about.

# Implementation

- Architecture / flow:
  - `mountAppShell()` now owns transient cross-mode continuity state.
  - Before remounting a workspace, the shell calls `persistActiveModeState()`:
    - exports the active workspace snapshot
    - stores it under its mode key
    - records the current header file-path input for that mode
  - After mounting a workspace, the shell calls `restoreModeState()`:
    - imports the cached snapshot for that mode if one exists
    - restores the mode-specific header path
    - ignores stale async restores by checking a render request id

- Key files or components:
  - `src/app/shell.ts`
    - adds `modeWorkspaceSnapshots`
    - adds `modeProjectPaths`
    - adds `renderRequestId`
    - introduces `persistActiveModeState()`
    - introduces `restoreModeState()`
    - upgrades `renderMode()` to restore cached state after remount
    - updates load/restore flows so successful imports refresh the in-memory mode cache
    - tracks header file-path input changes per mode

- Example:
  - If the user opens a PDF, annotates a page, then switches to Canvas Mode and back, the shell now remounts PDF Mode and reimports the cached `PdfStudyDocument` instead of showing an empty session again.
  - If the user enters a canvas save path, switches to PDF Mode, and later returns to Canvas Mode, the header path field restores the previous canvas path instead of reusing the PDF session path.

# Results

- Outputs:
  - Mode switching now preserves the active canvas workspace state in memory.
  - Mode switching now preserves the active PDF workspace state in memory.
  - The header save/load path field now keeps one value per mode instead of being overwritten by the last mode used.

- Metrics or observations:
  - The fix required only shell-level TypeScript changes and reused the existing workspace snapshot/import contracts.
  - No Rust-side API changes were needed for this slice.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success
  - Full manual verification of switching modes in the live app was not performed in this turn.

# Decisions

- Fact:
  - Mode continuity is implemented as in-memory workspace snapshots.
  - Assessment:
  - This is the smallest fix that matches the reported workflow problem without introducing file-system side effects on every mode switch.

- Fact:
  - The shell now keeps one project-path input value per mode.
  - Assessment:
  - This directly addresses the user’s complaint about having to re-enter paths after switching contexts.

- Fact:
  - Async restore work is guarded by a render request identifier.
  - Assessment:
  - This prevents stale restores from racing newer mode transitions when mounts become asynchronous.

# Limitations

- The fix is build-verified but not yet manually tested end-to-end in the running Tauri app.
- This slice preserves in-memory mode state across switches, but it does not yet add the separate `add-recent-pdf-quick-open-list` workflow improvement.
- The shell file already had unrelated local edits in the working tree, so `TODO.md` was updated locally but left out of the worker commit to keep the commit scoped and safe.

# Next steps

1. Implement `contain-pdf-within-viewport` because the PDF workspace still has a separate layout containment regression after mode continuity is fixed.
2. Implement `fix-select-tool` because canvas editing reliability is now one of the highest remaining direct interaction issues.
3. Implement `add-recent-pdf-quick-open-list` as the next usability follow-up once PDF session continuity is stable.

# Reproducibility

1. Inspect the shell continuity logic:
   - `src/app/shell.ts`
2. Build the frontend:
   - `npm run build`
3. Check the Rust side:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
4. Launch the app in development:
   - `npm run tauri dev`
5. Open a PDF, switch to Canvas Mode, then switch back to PDF Mode and confirm the study session is still present.
6. Draw on the canvas, switch to PDF Mode, then switch back and confirm the canvas document is still present.
