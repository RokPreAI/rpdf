# Title

rpdf offline acceptance checks and validation handoff report

# Context

This task implemented the current `TODO.md` item `add-offline-acceptance-checks`.

At this point the repo already had working slices for:

- canvas drawing and mixed content
- PDF navigation and annotation
- SVG export gating
- OCR fallback for weak PDFs
- save/load, autosave, and recovery

What was still missing was an executable validation layer that told a later worker exactly:

- what can already be checked automatically
- what still requires the GUI, real PDFs, or a drawing tablet
- how those checks map back to the project plan and specification

The task needed to stay bounded. The goal was validation and acceptance coverage, not another feature pass.

# Goals

- Add a runnable offline verification entrypoint for the currently automatable checks.
- Expand automated coverage for acceptance-relevant service behavior.
- Add a spec-mapped acceptance document that makes manual-only validation explicit.
- Advance the backlog to the final UX-hardening phase.

# Approach

The chosen approach was to combine:

- one small script for the automated offline subset
- a root acceptance-check document for the full matrix
- a few additional service tests that cover real contract edges the script should enforce

This avoided two common failure modes:

- a “validation task” that is only prose and cannot be rerun
- an “acceptance suite” that claims to automate GUI/tablet behavior it does not actually control

Rejected option:

- trying to add a full GUI automation harness in this pass
  - That would have widened scope sharply and still would not have covered the real drawing-tablet and study-session workflow checks the plan cares about.

# Implementation

Automated entrypoint:

- [scripts/run_acceptance_checks.sh](/home/rok/sync/ideas/rpdf/scripts/run_acceptance_checks.sh:1)
  - added a small offline script that:
    - runs `cargo test`
    - prints the next manual follow-up steps

Acceptance matrix:

- [ACCEPTANCE_CHECKS.md](/home/rok/sync/ideas/rpdf/ACCEPTANCE_CHECKS.md:1)
  - maps the current validation surface across:
    - app startup and offline behavior
    - canvas mode and pen workflow
    - SVG export gating
    - PDF navigation and annotation
    - reading-support fallback and warnings
    - save/load, autosave, and recovery
    - recolor behavior
  - explicitly separates:
    - automated checks
    - manual checks
    - current gaps
  - documents the minimum local asset set needed for meaningful manual validation

Additional automated tests:

- [src/app/services.rs](/home/rok/sync/ideas/rpdf/src/app/services.rs:1)
  - added coverage for:
    - canvas recovery snapshot round trip
    - PDF recovery snapshot round trip
    - SVG export acceptance for vector-only targets
    - SVG export refusal for raster and imported-PDF targets
  - also hardened recovery-root selection so tests and real runs choose the first writable state location instead of assuming the home-state path is writable

Backlog update:

- [TODO.md](/home/rok/sync/ideas/rpdf/TODO.md:1)
  - marked `add-offline-acceptance-checks` done
  - promoted `run-study-session-ux-hardening` to current

# Results

Observable outcomes:

- the repo now has a single scripted acceptance entrypoint:

```bash
./scripts/run_acceptance_checks.sh
```

- automated coverage now includes:
  - OCR/text-quality heuristics
  - canvas save/load round trip
  - PDF save/load round trip
  - canvas recovery snapshot round trip
  - PDF recovery snapshot round trip
  - SVG export compatibility/refusal rules

- the acceptance document now makes manual-only validation explicit instead of leaving it implicit or pretending it is already automated
- the backlog advanced to the final real-use UX-hardening task

Verification:

- Ran `cargo fmt`
- Ran `./scripts/run_acceptance_checks.sh`
- Result: passed

Observed intermediate issue:

- the first scripted run exposed a real environment problem
  - recovery snapshots initially targeted a read-only state path
  - the fix was to choose the first writable recovery root among `XDG_STATE_HOME`, `HOME/.local/state`, and `tmp`

# Decisions

- Kept the acceptance entrypoint as a shell script that delegates to `cargo test`.
  - Fact: the automated acceptance path is intentionally small and transparent.
  - Assessment: this is easier to trust and rerun than a heavier wrapper with hidden behavior.

- Documented local sample-asset requirements instead of committing binary PDF fixtures right now.
  - Fact: `ACCEPTANCE_CHECKS.md` names the minimum asset categories needed for manual validation.
  - Assessment: this keeps the repo lightweight while still making the manual workflow reproducible.

- Treated writable recovery-root selection as part of validation hardening.
  - Fact: the validation pass found a real filesystem assumption bug.
  - Assessment: fixing that during the acceptance task was correct because it directly affected whether the acceptance checks were truthfully rerunnable.

# Limitations

- GUI rendering, tablet pressure feel, recolor legibility, and end-to-end TTS playback still require manual validation.
- The scripted acceptance path currently runs only service-level and model-level Rust tests; it does not drive the live GUI.
- The acceptance document references local sample assets but does not ship binary PDF fixtures in the repo.
- The worktree still contains unrelated pre-existing deletions of `REPORT1.md` through `REPORT9.md` plus untracked planning/skill artifacts, which were left untouched.

# Next steps

1. Complete `run-study-session-ux-hardening`.
   The automated acceptance surface is now in place, so the highest-leverage remaining task is a real study-session pass focused on friction, warnings, pen feel, and mode transitions.

2. Capture one concrete manual validation run using real local assets.
   The acceptance document now defines the process; the next UX task should record at least one real pass against that checklist.

3. Decide later whether to add committed PDF fixtures.
   That may be useful if the repo grows a larger automated validation harness, but it is not yet necessary for the current scope.

# Reproducibility

1. Work in `/home/rok/sync/ideas/rpdf`.
2. Re-run formatting:

```bash
cargo fmt
```

3. Re-run the automated offline acceptance subset:

```bash
./scripts/run_acceptance_checks.sh
```

4. Then follow the manual matrix in [ACCEPTANCE_CHECKS.md](/home/rok/sync/ideas/rpdf/ACCEPTANCE_CHECKS.md:1) for:

- tablet drawing validation
- GUI workflow validation
- weak-PDF OCR fallback confirmation with real files
- recolor and annotation visibility checks
