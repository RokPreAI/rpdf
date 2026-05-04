# Title

Desktop app shell established for rpdf foundation task

# Context

- Problem:
  After the model layer was completed, the next current task in `TODO.md` was `bootstrap-desktop-app-shell`. The project still had no runnable application shell, so later canvas, PDF, annotation, and reading-support work had nowhere stable to attach.
- Constraints:
  The task had to stay narrow. It needed to create a real desktop application shell with separate workspace roots for Infinite Canvas Mode and PDF Mode, but it could not expand into actual canvas interaction, PDF loading, annotation behavior, or persistence. The repository also had no prior GUI dependencies, so verification depended on successfully resolving and compiling a new desktop UI crate.

# Goals

- Primary success criteria:
  Add a minimal desktop application that launches locally, shows a stable shell, and allows switching between Infinite Canvas Mode and PDF Mode.
- Secondary success criteria:
  Keep the shell tied to the existing model layer, preserve offline startup behavior, and leave clear top-level attachment points for later feature tasks.

# Approach

- Chosen approach:
  Added a binary entrypoint and an `eframe`/`egui`-based shell with top-level app state. The shell owns one default canvas document and one default PDF session, exposes a mode switcher, and renders separate placeholder workspaces plus a small session summary.
- Rejected options:
  Did not build a fake CLI-only shell because the task explicitly required a desktop application host. Did not implement any real canvas drawing, PDF loading, or annotation behavior in this step because those belong to later bounded tasks and would widen the change set unnecessarily.

# Implementation

- Architecture / flow:
  `src/main.rs` launches the native desktop app. `src/app.rs` defines `RpdfApp`, `StartupState`, and `ShellState`. `ShellState` holds the current workspace mode plus one default `CanvasDocument` and one default `PdfDocumentSession`, reusing the model types introduced in the previous task. The UI renders a top mode-switch strip, a left summary panel, and a central workspace panel whose content changes with the selected mode.
- Key files or components:
  - `Cargo.toml`: adds the binary target and `eframe` dependency.
  - `src/main.rs`: native app entrypoint and window setup.
  - `src/app.rs`: app shell state, workspace switching, and minimal root UI for both modes.
  - `src/lib.rs`: now exports the new app module.
- Example:
  The shell uses `WorkspaceMode` from the model layer as the single source of truth for whether the central panel renders Infinite Canvas Mode or PDF Mode. That keeps later workspace-specific behavior anchored to the same mode contract used by the domain model.

# Results

- Outputs:
  Added the application shell files:
  - `src/main.rs`
  - `src/app.rs`
  Updated:
  - `Cargo.toml`
  - `src/lib.rs`
  - `Cargo.lock`
  - `TODO.md`
  - `SUBTODO.md`
- Metrics or observations:
  The app now has:
  - a native desktop entrypoint
  - deterministic offline startup state
  - a visible switch between Infinite Canvas Mode and PDF Mode
  - separate root UI surfaces for future canvas and PDF work
  - a side summary panel that exposes current mode-local state without adding broad UI clutter
- Verification:
  Ran `cargo check` successfully after allowing dependency resolution for the new GUI crate. The build finished with `rpdf v0.1.0` compiling successfully.

# Decisions

- Tradeoffs made:
  - Chose a lightweight immediate-mode GUI shell because it provides a real desktop host with minimal ceremony and keeps the current task focused.
  - Reused the existing model-layer types instead of inventing parallel shell-only structs for documents and sessions.
  - Kept both workspaces as placeholders with state summaries rather than adding fake partial features that would be replaced immediately by later tasks.

# Limitations

- Known issues, uncertainties, or risks:
  - The shell compiles, but no interactive canvas, PDF loading, or annotation workflow exists yet.
  - Verification in this task is compile-level only; no automated GUI runtime test was added.
  - The app currently uses default in-memory documents and does not persist state.
  - The dependency tree is now much larger because of the GUI stack, so later tasks should keep compile verification focused.

# Next steps

1. Implement `implement-canvas-pen-and-viewport` because the shell now provides a concrete Infinite Canvas Mode surface where pressure-sensitive drawing and pan/zoom behavior can land.
2. Implement `add-pdf-viewer-and-navigation` after the canvas loop or alongside it in a later cycle, because PDF Mode now has a stable root but still no document behavior.

# Reproducibility

1. From `/home/rok/sync/ideas/rpdf`, inspect the shell files with `sed -n '1,260p' src/app.rs` and `sed -n '1,220p' src/main.rs`.
2. Verify compilation with `cargo check`.
