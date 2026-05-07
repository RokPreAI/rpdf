# Title

rpdf mode-shell state extraction report

# Context

This task implemented the current `TODO.md` item `extract-mode-shells-and-shared-ui-state`.

The previous worker task split the large app file into `src/app/mod.rs`, `src/app/canvas.rs`, `src/app/pdf.rs`, and `src/app/util.rs`, but the top-level app state still treated canvas, PDF, and shared UI concerns as one blended shell. That shape still made the code read like a single mixed workspace even though the files had been separated.

The goal here was to make the mode split real in the in-memory UI state without changing the existing feature set. This was important before moving on to service-boundary work for reading support, export logic, and OCR fallback.

# Goals

- Separate Infinite Canvas Mode and PDF Mode into clearer code-level UI shells.
- Keep shared tool state only in an explicit shared UI container.
- Preserve the current feature band and behavior while changing ownership boundaries.
- Update the backlog so the next current task reflects the new state of the repo.
- Verify the refactor with `cargo check`.

# Approach

The smallest complete slice was to split the top-level shell state into three containers:

- `CanvasModeState`
- `PdfModeState`
- `SharedUiState`

This was chosen instead of building a deeper service layer in the same pass. The task contract was about mode-shell clarity, not yet about adapter or service extraction. By changing the state ownership first, the next task can focus on service boundaries without also having to untangle mixed mode state.

# Implementation

The main structural change is in [src/app/mod.rs](/home/rok/sync/ideas/rpdf/src/app/mod.rs:1):

- `ShellState` now contains:
  - `mode`
  - `canvas_mode`
  - `pdf_mode`
  - `shared_ui`
- `CanvasModeState` now owns:
  - the editable `CanvasDocument`
  - canvas-specific interaction/UI state
- `PdfModeState` now owns:
  - the editable `PdfDocumentSession`
  - PDF-specific interaction/UI state
- `SharedUiState` now owns:
  - shared annotation tool state

Canvas-side code in [src/app/canvas.rs](/home/rok/sync/ideas/rpdf/src/app/canvas.rs:1) was updated to use:

- `self.shell.canvas_mode.document`
- `self.shell.canvas_mode.ui`
- `self.shell.shared_ui.annotation_tools`

PDF-side code in [src/app/pdf.rs](/home/rok/sync/ideas/rpdf/src/app/pdf.rs:1) was updated to use:

- `self.shell.pdf_mode.session`
- `self.shell.pdf_mode.ui`
- `self.shell.shared_ui.annotation_tools`

The backlog in [TODO.md](/home/rok/sync/ideas/rpdf/TODO.md:1) was updated so:

- `extract-mode-shells-and-shared-ui-state` is marked done
- `formalize-reading-and-export-services` is now the current task

# Results

The mode-shell extraction completed successfully.

Observable outcomes:

- canvas state is no longer stored directly as top-level `self.shell.canvas`
- PDF state is no longer stored directly as top-level `self.shell.pdf`
- shared annotation tool state is now explicitly separated from mode-specific state
- the current task in `TODO.md` moved forward to service-boundary work

Verification:

- Ran `cargo check`
- Result: passed

# Decisions

- Kept the split focused on ownership containers rather than building new abstractions for every UI action. This preserved the bounded scope of the task.
- Left rendering and interaction methods in their current files. The state split was enough to satisfy the current task without turning it into a second large file reorganization.
- Kept annotation tool state in `SharedUiState` because it is intentionally reused across both modes today.

# Limitations

- This task did not introduce service traits or adapter layers. Reading support, TTS launching, and export decisions are still implemented directly inside app code.
- The UI code is clearer, but behavior is still method-heavy and not yet split into command/use-case boundaries.
- Verification was compile-level only. No automated runtime UI interaction test was added in this pass.
- The worktree still contains unrelated pre-existing deletions of `REPORT1.md` through `REPORT9.md`, which were left untouched.

# Next steps

1. Complete `formalize-reading-and-export-services`.
   The state boundaries are now explicit enough that reading support and export logic can be pulled behind cleaner interfaces without fighting mixed mode state first.

2. Then implement `add-text-fallback-and-warning-flow`.
   OCR fallback and warning logic should land after service boundaries exist, not as more ad hoc UI-owned logic.

3. After that, complete `add-save-load-and-recovery`.
   Persistence work will benefit from the clearer distinction between canvas-mode UI state, PDF-mode UI state, and shared tool state.

# Reproducibility

Working directory:

- `/home/rok/sync/ideas/rpdf`

Files changed for this task:

- [src/app/mod.rs](/home/rok/sync/ideas/rpdf/src/app/mod.rs:1)
- [src/app/canvas.rs](/home/rok/sync/ideas/rpdf/src/app/canvas.rs:1)
- [src/app/pdf.rs](/home/rok/sync/ideas/rpdf/src/app/pdf.rs:1)
- [TODO.md](/home/rok/sync/ideas/rpdf/TODO.md:1)

Verification command:

```bash
cargo check
```

Expected result:

- the crate compiles successfully in the default development profile
