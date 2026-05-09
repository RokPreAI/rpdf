# Title

rpdf keyboard shortcuts and discoverability report

# Context

The active task in `TODO.md` was `add-keyboard-shortcuts-and-discoverability`. The previous UI simplification pass stabilized the visible control groups, but most actions still required clicking through the interface or relying on undocumented existing behavior such as canvas paste and space-pan.

Two constraints shaped this pass:

- shortcut handling needed to stay out of text-entry fields so typing notes and paths would not break
- this needed to remain a bounded first-pass shortcut layer, not an attempt to bind every possible action in both modes

# Goals

Primary success criteria:

- add working keyboard shortcuts for the most common study and editing actions
- keep the shortcuts coherent across both modes
- expose the bindings in at least one visible in-app location

Secondary success criteria:

- update a durable project doc so the shortcut map is not discoverable only inside the UI
- finish the current backlog item cleanly and advance `TODO.md`

# Approach

The chosen approach was to centralize shortcuts in the shared app shell and keep mode-specific actions behind their existing mode methods:

- add one shortcut dispatcher in `src/app/mod.rs` that runs once per frame
- gate the dispatcher behind `ctx.wants_keyboard_input()` so text fields keep normal behavior
- use a small, stable shortcut set for mode switching, tool switching, save/load/recover, PDF open/navigation/TTS, and canvas export
- render a compact shortcut reference in the left sidebar so the bindings stay visible during normal use

Rejected option:

- binding every exposed button immediately
  - Assessment: that would have widened scope and created a noisier shortcut surface before the first pass could be validated.

# Implementation

Architecture and flow:

- [src/app/mod.rs](/home/rok/sync/ideas/rpdf/src/app/mod.rs:1)
  - added `handle_global_shortcuts(...)`
  - added mode-specific helpers:
    - `handle_canvas_mode_shortcuts(...)`
    - `handle_pdf_mode_shortcuts(...)`
  - added shared shortcut helpers for `Cmd/Ctrl` and `Cmd/Ctrl+Shift` combinations
  - added `toggle_workspace_mode(...)`
  - added `visible_shortcuts(...)` and `render_shortcuts_summary(...)` so the current mode exposes a compact in-app cheat sheet in the sidebar

- [src/app/canvas.rs](/home/rok/sync/ideas/rpdf/src/app/canvas.rs:1)
  - widened the existing canvas action entry points to `pub(super)` so the shared shell can trigger them safely:
    - `export_canvas_svg(...)`
    - `save_canvas_document(...)`
    - `load_canvas_document(...)`
    - `recover_canvas_document(...)`

- [src/app/pdf.rs](/home/rok/sync/ideas/rpdf/src/app/pdf.rs:1)
  - widened the existing PDF action entry points to `pub(super)` for the same reason:
    - `open_pdf_document(...)`
    - `step_pdf_page(...)`
    - `start_pdf_tts(...)`
    - `stop_pdf_tts(...)`
    - `save_pdf_session(...)`
    - `load_pdf_session(...)`
    - `recover_pdf_session(...)`

- [README.md](/home/rok/sync/ideas/rpdf/README.md:1)
  - added a shortcut section documenting the current first-pass bindings

- [TODO.md](/home/rok/sync/ideas/rpdf/TODO.md:1)
  - marked `add-keyboard-shortcuts-and-discoverability` done
  - left the current and active sections empty because no other backlog item remained open in this file

Shortcut map implemented in this pass:

- Global:
  - `Tab`: switch workspace mode
  - `B`, `H`, `V`, `E`: select ink, highlighter, selection, eraser
  - `Cmd/Ctrl+S`: save current mode
  - `Cmd/Ctrl+L`: load current mode
  - `Cmd/Ctrl+R`: recover latest autosave for current mode
- Infinite Canvas Mode:
  - `Cmd/Ctrl+V`: paste clipboard image
  - `Cmd/Ctrl+Shift+E`: export SVG
  - hold `Space` and drag: pan
  - double-tap `Space`: fit content
- PDF Mode:
  - `Cmd/Ctrl+O`: open PDF
  - `Left` / `Right`: previous or next page
  - `T`: start or stop TTS

# Results

Outputs:

- the app now has a shared first-pass shortcut dispatcher covering the main editing and study actions
- the sidebar now shows a visible mode-aware shortcut reference
- `README.md` now documents the current shortcut map
- the next report was created at [reports/REPORT23.md](/home/rok/sync/ideas/rpdf/reports/REPORT23.md:1)

Metrics and observations:

- the first verification pass caught one real issue: the shared dispatcher could not call several existing mode actions because they were still private to their mode modules
- that was fixed by widening only the shortcut-triggered action entry points to `pub(super)`, after which the full verification set passed

Verification:

- ran `cargo fmt`
- ran `cargo test`
- ran `./scripts/run_acceptance_checks.sh`
- all passed after the method-visibility fix

# Decisions

Fact: shortcut handling is skipped when `egui` wants keyboard input.
Assessment: preserving normal note/path typing is more important than capturing every possible shortcut press.

Fact: the first pass uses one coherent shortcut set instead of trying to bind every button.
Assessment: this keeps the bindings easier to learn and reduces accidental conflicts while the app is still evolving.

Fact: the in-app shortcut surface is mode-aware.
Assessment: showing only the relevant mode-specific bindings keeps the cheat sheet smaller and easier to scan.

# Limitations

- This pass does not add shortcuts for every content-creation action such as text insertion or PDF-page import; it focuses on the highest-frequency study and editing flows first.
- The bindings were verified through build/tests and the acceptance script in a headless environment, not through a live desktop keypress session here.
- The README still contains older spelling and wording issues outside the newly added shortcut section.
- Unrelated user-side report moves and planning artifacts are still present in the worktree and were intentionally left untouched.

# Next steps

1. Run a live graphical shortcut pass on a desktop session.
   The code-level verification passed, but a real keypress pass is still the best way to confirm there are no ergonomic surprises in `egui` focus behavior.

2. Build a new backlog from observed UX gaps rather than extending this shortcut set blindly.
   `TODO.md` is now closed again, so the next implementation slice should be driven by concrete follow-up requirements or manual test findings.

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

5. In a graphical local session, verify that:

- `Tab` switches modes when no text field is focused
- `B`, `H`, `V`, `E` switch tools in both modes
- `Cmd/Ctrl+S`, `Cmd/Ctrl+L`, and `Cmd/Ctrl+R` trigger the current mode’s persistence actions
- PDF Mode responds to `Cmd/Ctrl+O`, `Left`, `Right`, and `T`
- Infinite Canvas Mode responds to `Cmd/Ctrl+V`, `Cmd/Ctrl+Shift+E`, hold-`Space` pan, and double-`Space` fit-to-content
