# Title

rpdf save/load, autosave, and recovery report

# Context

This task implemented the current `TODO.md` item `add-save-load-and-recovery`.

The repo already had editable in-memory state for both first-class workflows:

- `CanvasDocument` for Infinite Canvas Mode
- `PdfDocumentSession` for PDF Mode

It also already tracked `AutosaveState`, but that state was only a marker. There was no real save-file format, no load path, no autosave snapshots, and no recovery flow after interruption.

The work had to stay bounded. The goal was to make editable state durable without redesigning the app architecture or expanding into acceptance-check work.

# Goals

- Add explicit editable save/load for both canvas and PDF-study sessions.
- Add internal autosave snapshots and recovery actions.
- Keep recovery state separate from export behavior.
- Keep the persistence format offline and local.
- Verify the change with Rust formatting and tests.

# Approach

The chosen approach was a small JSON-based persistence service behind the existing `AppServices` boundary.

That kept the task local:

- use `serde` and `serde_json` for explicit file-backed save formats
- keep save/load and recovery logic in a persistence service
- expose small UI controls in the canvas and PDF toolbars
- autosave only when a document is dirty and enough time has passed, instead of writing on every frame

Rejected option:

- adding a heavier storage engine or database layer
  - This would have widened scope and was not needed for the current task. The plan only required local durable working-state persistence, not a more complex storage backend yet.

# Implementation

Dependency updates:

- [Cargo.toml](/home/rok/sync/ideas/rpdf/Cargo.toml:1)
  - added `serde`
  - added `serde_json`
- `Cargo.lock`
  - updated after fetching the new crates

Model serialization:

- [src/model/mod.rs](/home/rok/sync/ideas/rpdf/src/model/mod.rs:1)
  - added `Serialize` and `Deserialize` derives across the editable document model so the current working state can be written and restored directly

Persistence service:

- [src/app/services.rs](/home/rok/sync/ideas/rpdf/src/app/services.rs:1)
  - added `PersistenceService`
  - added versioned wrapper formats:
    - `CanvasSaveFile`
    - `PdfSaveFile`
  - added explicit operations:
    - `save_canvas_document(...)`
    - `load_canvas_document(...)`
    - `save_pdf_session(...)`
    - `load_pdf_session(...)`
    - `write_canvas_recovery_snapshot(...)`
    - `write_pdf_recovery_snapshot(...)`
    - `recover_canvas_document(...)`
    - `recover_pdf_session(...)`
  - recovery snapshots are stored under the local state root:
    - `$XDG_STATE_HOME/rpdf/recovery/` when available
    - otherwise `~/.local/state/rpdf/recovery/`
    - otherwise a temp-dir fallback
  - added round-trip tests for both canvas and PDF save files

Autosave and dirty-state flow:

- [src/app/mod.rs](/home/rok/sync/ideas/rpdf/src/app/mod.rs:1)
  - `AppServices` now includes persistence
  - canvas and PDF interaction state now carry:
    - explicit document/session save paths
    - autosave timing fields
    - canvas save status text
  - added:
    - `tick_autosave()`
    - `mark_canvas_dirty()`
    - `mark_pdf_dirty()`
    - `current_unix_ms()`
  - autosave writes recovery snapshots every 2 seconds at most while a document is dirty

Canvas-mode integration:

- [src/app/canvas.rs](/home/rok/sync/ideas/rpdf/src/app/canvas.rs:1)
  - added toolbar controls for:
    - canvas save path
    - `Save canvas`
    - `Load canvas`
    - `Recover autosave`
  - wired document edits to `mark_canvas_dirty()`
  - added:
    - `save_canvas_document()`
    - `load_canvas_document()`
    - `recover_canvas_document()`

PDF-mode integration:

- [src/app/pdf.rs](/home/rok/sync/ideas/rpdf/src/app/pdf.rs:1)
  - added toolbar controls for:
    - session save path
    - `Save session`
    - `Load session`
    - `Recover autosave`
  - wired page navigation, recolor settings, PDF open, and annotation creation to `mark_pdf_dirty()`
  - added:
    - `save_pdf_session()`
    - `load_pdf_session()`
    - `recover_pdf_session()`
  - loading or recovering a PDF session also rebuilds page-count state from the saved PDF source path when that file still exists

Backlog update:

- [TODO.md](/home/rok/sync/ideas/rpdf/TODO.md:1)
  - marked `add-save-load-and-recovery` done
  - promoted `add-offline-acceptance-checks` to current

# Results

Observable outcomes:

- canvas working state can now be saved to and loaded from explicit JSON files
- PDF-study session state can now be saved to and loaded from explicit JSON files
- both modes now write internal autosave snapshots while dirty
- both modes can recover the latest autosave snapshot through explicit toolbar actions
- dirty-state tracking now updates metadata modification timestamps when persistence-relevant edits occur
- the current backlog advanced from persistence to acceptance-check work

Verification:

- Ran `cargo fmt`
- Ran `cargo test`
- Result: passed

Automated test coverage added:

- canvas save-file round trip
- PDF session save-file round trip
- existing OCR/text-quality tests still pass after the persistence changes

Environment note:

- the first `cargo test` attempt needed network access to fetch the new `serde` crates
- after the dependency fetch, the full test run passed locally

# Decisions

- Used plain JSON save files instead of a database.
  - Fact: the persistence layer serializes versioned wrapper structs with `serde_json`.
  - Assessment: this keeps the format simple, inspectable, and easy to evolve while the app is still changing quickly.

- Kept recovery as a separate internal snapshot path.
  - Fact: recovery snapshots live in a local state directory and are distinct from user-chosen save files.
  - Assessment: this preserves the plan’s distinction between editable working-state recovery and user-facing output/export.

- Triggered autosave on dirty state with a 2-second interval.
  - Fact: autosave is rate-limited instead of writing every frame.
  - Assessment: this is a reasonable first durability step without turning viewport or drawing interaction into constant disk churn.

# Limitations

- Save/load currently serializes the existing Rust model directly, so future schema changes will need migration handling beyond the current version check.
- Recovery is user-invoked through toolbar buttons; there is not yet a startup prompt or modal that automatically offers recovery on launch.
- Dirty-state coverage is good for major edits, but not every transient UI or view-only change is intentionally persisted.
- PDF session reload depends on the original PDF path still existing for page-count reconstruction.
- No end-to-end UI automation was added; verification is compile plus unit-test level.
- The worktree still contains unrelated pre-existing deletions of `REPORT1.md` through `REPORT9.md` and untracked planning/skill artifacts, which were left untouched.

# Next steps

1. Complete `add-offline-acceptance-checks`.
   Persistence and OCR fallback now both exist, so the highest-leverage next step is to add repeatable checks for save/load, recovery, weak-PDF fallback, and SVG gating.

2. Add a startup-facing recovery prompt after acceptance checks.
   The core recovery mechanism exists, but the UX is still manual and should become more obvious once verification artifacts are in place.

3. Plan schema migration behavior before broadening the save format.
   The wrappers already carry `schema_version`, so the next storage-related hardening step should define how incompatible versions are handled instead of only rejecting them.

# Reproducibility

1. Work in `/home/rok/sync/ideas/rpdf`.
2. If dependencies are not present yet, fetch them during test/build:

```bash
cargo test
```

3. Reformat and re-run verification:

```bash
cargo fmt
cargo test
```

4. Relevant files changed for this task:

- [Cargo.toml](/home/rok/sync/ideas/rpdf/Cargo.toml:1)
- [TODO.md](/home/rok/sync/ideas/rpdf/TODO.md:1)
- [src/app/mod.rs](/home/rok/sync/ideas/rpdf/src/app/mod.rs:1)
- [src/app/canvas.rs](/home/rok/sync/ideas/rpdf/src/app/canvas.rs:1)
- [src/app/pdf.rs](/home/rok/sync/ideas/rpdf/src/app/pdf.rs:1)
- [src/app/services.rs](/home/rok/sync/ideas/rpdf/src/app/services.rs:1)
- [src/model/mod.rs](/home/rok/sync/ideas/rpdf/src/model/mod.rs:1)
- [REPORT14.md](/home/rok/sync/ideas/rpdf/REPORT14.md:1)
