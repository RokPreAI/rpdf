# Title

rpdf GUI simplification and tool-grouping report

# Context

The active task in `TODO.md` was `simplify-gui-and-tool-grouping`. The application already had working canvas editing, PDF reading support controls, save/recovery, selection, eraser support, and clipboard-image paste, but those controls had accumulated as several dense blocks with too much always-visible detail.

Two constraints shaped this pass:

- this needed to stay a bounded UI simplification slice rather than expanding into the separate `add-keyboard-shortcuts-and-discoverability` task
- save/recovery state, reading-support warnings, and export guidance still needed to remain visible enough to support real study use

# Goals

Primary success criteria:

- make the main canvas and PDF control surfaces easier to scan
- give tools, session actions, content actions, and reading actions clearer homes
- reduce low-value always-visible detail without removing key warnings or recovery feedback

Secondary success criteria:

- keep the refactor local to the existing `egui` shell instead of widening into architecture work
- advance `TODO.md` cleanly so the next worker task becomes keyboard shortcuts and discoverability

# Approach

The chosen approach was to simplify the current shell through layout and grouping rather than by removing capabilities:

- add a shared section-card helper in the app shell so the main control groups read as a small number of named blocks
- reduce summary-panel noise by showing only the most useful mode state
- keep common actions visible while moving lower-frequency inputs behind collapsible subsections

Rejected option:

- redesigning the entire visual language or introducing a new design system
  - Assessment: that would have widened scope beyond the concrete backlog item and risked mixing style experimentation with the still-pending shortcut work.

# Implementation

Architecture and flow:

- [src/app/mod.rs](/home/rok/sync/ideas/rpdf/src/app/mod.rs:1)
  - simplified the top app chrome so mode, active tool, and offline state are visible in one wrapped row
  - reduced the left sidebar from a verbose session dump to compact workspace summaries
  - added shared helpers for section-card rendering, compact key/value summary rows, current tool labeling, current mode labeling, and canvas selection summaries
  - converted the shared annotation toolbar into a concise `Tools` section with a short current-tool explanation and a collapsed `Quick note` row

- [src/app/canvas.rs](/home/rok/sync/ideas/rpdf/src/app/canvas.rs:1)
  - reworked the canvas controls into three clearer sections:
    - `Session`
    - `Add content`
    - `Selection and export`
  - kept background selection visible, but folded text/image/PDF import inputs into smaller collapsible groups
  - kept export guidance visible while moving manual item targeting and PDF-page recolor actions behind collapses
  - added a direct current-target summary so the user can tell quickly whether export or recolor actions apply to the whole canvas or a narrower selection

- [src/app/pdf.rs](/home/rok/sync/ideas/rpdf/src/app/pdf.rs:1)
  - reworked the PDF controls into three clearer sections:
    - `Session`
    - `Navigation and view`
    - `Reading support`
  - kept page movement and reading support visible
  - folded recolor settings and annotation palettes into collapsible subgroups so lower-frequency appearance controls no longer compete with the main reading flow

- [TODO.md](/home/rok/sync/ideas/rpdf/TODO.md:1)
  - marked `simplify-gui-and-tool-grouping` done
  - promoted `add-keyboard-shortcuts-and-discoverability` to the current task

Example:

- in Infinite Canvas Mode, the user now sees one top-level `Add content` section instead of three always-open input rows; text notes, image import, and PDF-page import stay available, but only expand when needed

# Results

Outputs:

- both mode shells now present the main study controls as a small number of named sections instead of one dense mixed toolbar block
- the sidebar now emphasizes current workspace state rather than raw internal debug-like details
- the next report was created at [reports/REPORT22.md](/home/rok/sync/ideas/rpdf/reports/REPORT22.md:1)

Metrics and observations:

- the first verification pass caught one real regression: the new canvas selection summary initially missed the existing `SelectionTarget::PdfPageSelection(...)` variant
- that exhaustiveness issue was fixed immediately, then the full verification set passed

Verification:

- ran `cargo fmt`
- ran `cargo test`
- ran `./scripts/run_acceptance_checks.sh`
- all passed after the single match-exhaustiveness fix

# Decisions

Fact: the simplification pass kept recovery banners, export guidance, and reading-support banners visible.
Assessment: those surfaces are part of the product’s trust model, so hiding them to make the UI look cleaner would have been the wrong tradeoff.

Fact: lower-frequency controls were folded with `egui` collapses instead of being removed.
Assessment: this reduces clutter without changing the current feature set or forcing users into hidden-only workflows.

Fact: the left sidebar now acts as a compact workspace summary rather than a verbose state dump.
Assessment: the previous panel exposed too many low-value implementation details for the current product stage.

# Limitations

- This pass improves layout and grouping only; it does not yet add the broader keyboard shortcut map the user requested.
- The simplified GUI was verified through build/tests and the acceptance script in this headless environment, not through a live graphical study session here.
- The placeholder-rendering limitation for imported images and imported PDF visuals remains unchanged.
- Unrelated user-side report moves and planning artifacts are still present in the worktree and were intentionally left untouched.

# Next steps

1. Implement `add-keyboard-shortcuts-and-discoverability`.
   The control layout is now stable enough to attach a coherent shortcut map without documenting shifting UI group names.

2. Run a live graphical pass for the new grouped layout.
   The current environment is still headless, so a local desktop check is the right way to confirm the simplified sections feel faster to scan in practice.

# Reproducibility

1. Work in `/home/rok/sync/ideas/rpdf`.
2. Format the repo:

```bash
cargo fmt
```

3. Run the test suite:

```bash
cargo test
```

4. Run the automated acceptance checks:

```bash
./scripts/run_acceptance_checks.sh
```

5. In a graphical local session, confirm that:

- Infinite Canvas Mode shows compact `Session`, `Add content`, and `Selection and export` sections
- PDF Mode shows compact `Session`, `Navigation and view`, and `Reading support` sections
- the sidebar shows concise workspace summaries rather than the older verbose detail list
