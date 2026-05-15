# Title

Autosave and recovery flow report

# Context

With explicit save/load now in place, `TODO.md` advanced to `add-autosave-and-recovery`. The app had become capable enough that losing an unsaved canvas or PDF session would be more painful than earlier prototype loss, but there was still no recovery path if the window closed unexpectedly or the user forgot to save.

The constraints were:

- build on the new versioned document snapshots instead of inventing a second state format
- keep the recovery flow local and offline-first
- avoid blocking on native filesystem watchers, background daemons, or extra plugins
- keep the UI clear about when recovery is available and when it has been cleared

# Goals

- Primary success criteria:
  - autosave the active workspace regularly
  - expose recovery actions in the app shell
  - restore the latest autosave snapshot into the current mode
  - clear stale recovery state after an explicit manual save or user action

- Secondary success criteria:
  - keep autosave format aligned with the same versioned workspace snapshots used by manual save/load
  - keep the feature mode-aware so Canvas and PDF sessions do not overwrite each other

# Approach

- Chosen approach:
  - Use browser `localStorage` as the first recovery store because it is available inside the Tauri webview, local-only, and good enough for bounded autosave snapshots.
  - Store one autosave record per mode, containing:
    - timestamp
    - exported workspace snapshot
  - Add `Restore autosave` and `Clear recovery` actions to the shared shell header.
  - Trigger autosave on an interval and again on `beforeunload`.

- Rejected options:
  - Writing background recovery files through new Rust commands in this cycle would widen scope before the simpler snapshot path was proven.
  - Relying only on manual save would not satisfy the recovery requirement.
  - Building a hidden autosave system without visible recovery controls would make the trust state worse, not better.

# Implementation

- Architecture / flow:
  - `src/app/shell.ts` now owns:
    - autosave storage keys per mode
    - autosave record parsing
    - interval-based snapshot writes
    - `Restore autosave`
    - `Clear recovery`
  - Autosave records store exported workspace snapshots, so the recovery format matches the real document model rather than UI-local flags.
  - Recovery is mode-aware:
    - Canvas autosave is stored separately from PDF autosave
    - switching modes updates the visible recovery state

- Key files or components:
  - `src/app/shell.ts`
  - `src/styles.css`
  - `TODO.md`

- Example:
  - If the user works in Canvas Mode without manually saving, the shell now writes periodic autosave snapshots into local recovery storage.
  - On the next launch, if a canvas recovery snapshot exists, the header surfaces a recovery action and timestamp so the user can restore it intentionally.

# Results

- Outputs:
  - The shell now autosaves the active workspace every few seconds and again on unload.
  - Recovery buttons now appear only when a mode-specific autosave snapshot exists.
  - Explicit save clears the recovery snapshot for that mode.
  - `TODO.md` now marks `add-autosave-and-recovery` as done and advances the current task to `add-svg-export-eligibility`.

- Metrics or observations:
  - Recovery state is now visible instead of hidden, which matches the project’s trust-surface direction.
  - The implementation reuses the existing workspace export/import path, so autosave does not create a second incompatible state format.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - The first recovery store is `localStorage`, not filesystem-backed autosave files.
  - Assessment:
  - This is the narrowest working recovery layer and keeps the snapshot format local and offline-first while avoiding another backend expansion in the same cycle.

- Fact:
  - Recovery is explicit in the UI through restore/clear actions.
  - Assessment:
  - That is preferable to silent recovery because it preserves user trust and matches the broader product rule against hiding uncertain state.

- Fact:
  - Manual save clears autosave for the same mode.
  - Assessment:
  - This reduces confusion between durable saved files and temporary recovery snapshots.

# Limitations

- Autosave is local to the current Tauri webview storage and is not yet mirrored into explicit recovery files on disk.
- There is no multi-version recovery history yet; only the latest snapshot is kept per mode.
- Recovery was verified by build checks and code-path inspection, not by a scripted crash-and-restore test.
- Autosave cadence is fixed in code and not yet user-configurable.

# Next steps

1. Complete `add-svg-export-eligibility` so the now-more-durable canvas state can leave the app through a trustworthy export path.
2. Complete `add-draw-shapes` so autosaved and persisted canvas work includes more than freehand/image content.
3. Complete `add-tts-and-reliability-pipeline` so PDF Mode reaches the next real product-defining capability rather than staying mostly structural.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Rust backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Inspect the autosave/recovery shell logic:
   - `src/app/shell.ts`
4. Inspect the recovery UI styling:
   - `src/styles.css`
