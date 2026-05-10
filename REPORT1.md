# Title

Architecture foundation, versioned models, and app shell split report

# Context

`rpdf` started this cycle as a single-file TypeScript canvas prototype with a placeholder Rust `greet` command. The current `TODO.md` selected `architecture-foundation-and-pdfium-gate` as the highest-priority task, and that task explicitly carried part of the older `Split main.ts into multiple files` work with it.

The main constraints were:

- keep the product aligned with the chosen Tauri + Rust platform direction
- preserve the robust architecture choice from `DECISION.md`
- honor the early Pdfium-style PDF-engine commitment instead of reopening the engine decision
- keep scope narrow enough to finish one bounded cycle with verification and a commit

# Goals

- Primary success criteria:
  - Replace the placeholder backend command layout with explicit app, contracts, domain, and infrastructure boundaries.
  - Introduce a Pdfium-named Rust adapter boundary so later PDF work no longer depends on a future engine-selection task.
  - Split the monolithic frontend into an app shell with separate Canvas Mode and PDF Mode entry points.
  - Add versioned internal document/session schemas for canvas and PDF study state.

- Secondary success criteria:
  - Keep the current canvas behavior working after the split.
  - Expose backend bootstrap state to the frontend through a typed IPC surface.

# Approach

- Chosen approach:
  - Use the current task as the architecture boundary pass and fold in the tightly coupled `versioned-project-and-session-model` and `split-main-into-shell-and-modes` work because they are direct consequences of the same structural change.
  - Keep the Rust PDF layer deliberately thin: create a concrete `PdfiumEngineAdapter` boundary and typed commands now, but leave page rendering and text extraction behavior intentionally unimplemented behind that boundary.
  - Preserve the working canvas behavior by moving it into a dedicated canvas workspace module instead of rewriting the drawing model at the same time.

- Rejected options:
  - Leaving the frontend in `src/main.ts` and only adding Rust modules would have kept the old architectural drift alive.
  - Fully implementing PDF rendering in this cycle would have widened scope into `build-pdf-mode-shell` and `add-tts-and-reliability-pipeline`.
  - Deferring the versioned model work would have left the new backend boundaries without stable document/session shapes.

# Implementation

- Architecture / flow:
  - The frontend now boots through `src/main.ts` into `src/app/shell.ts`.
  - `src/app/shell.ts` owns app mode state, mode switching, backend bootstrap loading, and workspace mounting.
  - `src/features/canvas/workspace.ts` contains the moved canvas interaction code and preserves the existing drawing, panning, erasing, paste-image, and zoom behavior.
  - `src/features/pdf/workspace.ts` provides a distinct PDF workspace shell that displays the prepared backend contract and trust-state scaffolding.
  - The Rust side now has explicit layers:
    - `src-tauri/src/app/` for commands and service orchestration
    - `src-tauri/src/contracts/` for IPC DTOs
    - `src-tauri/src/domain/` for versioned canvas/PDF/reading models
    - `src-tauri/src/infrastructure/pdf_engine/` for the Pdfium adapter boundary

- Key files or components:
  - Frontend:
    - `src/app/types.ts`
    - `src/app/state.ts`
    - `src/app/shell.ts`
    - `src/features/canvas/workspace.ts`
    - `src/features/pdf/workspace.ts`
    - `src/styles.css`
    - `index.html`
  - Backend:
    - `src-tauri/src/lib.rs`
    - `src-tauri/src/app/commands.rs`
    - `src-tauri/src/app/services.rs`
    - `src-tauri/src/contracts/dto.rs`
    - `src-tauri/src/domain/canvas.rs`
    - `src-tauri/src/domain/pdf.rs`
    - `src-tauri/src/domain/reading.rs`
    - `src-tauri/src/infrastructure/pdf_engine/mod.rs`
    - `src-tauri/Cargo.toml`

- Example:
  - The frontend now calls `get_app_bootstrap` through Tauri IPC to learn:
    - supported modes
    - active PDF backend status
    - known reading reliability states
  - The PDF workspace shell renders that bootstrap information directly, which proves the mode split and backend contract without pretending PDF rendering already exists.

# Results

- Outputs:
  - The repo now has explicit frontend and backend boundaries instead of a monolithic entrypoint on both sides.
  - The backend exposes typed commands:
    - `get_app_bootstrap`
    - `get_pdf_backend_status`
    - `render_pdf_page`
    - `extract_pdf_page_text`
  - The backend contains versioned `CanvasDocument` and `PdfStudyDocument` schemas plus reading/cache-related types.
  - `TODO.md` was updated to mark these completed tasks:
    - `architecture-foundation-and-pdfium-gate`
    - `versioned-project-and-session-model`
    - `split-main-into-shell-and-modes`

- Metrics or observations:
  - The frontend build succeeded after the split.
  - Rust `cargo check` succeeded with the new module layout and the added optional `pdfium-render` dependency.
  - Rust emitted dead-code warnings for the new domain models, which is expected at this stage because the schemas are now defined before save/load and PDF features consume them.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success with dead-code warnings for currently unconsumed domain structs and enums

# Decisions

- Fact:
  - The frontend was split into an app shell plus workspace modules in the same cycle as the architecture foundation work.
  - Assessment:
  - This was the right tradeoff because the selected architecture task explicitly subsumed part of the `main.ts` split, and keeping the split separate would have created artificial duplication.

- Fact:
  - A concrete `PdfiumEngineAdapter` boundary was added before real page rendering.
  - Assessment:
  - This keeps the engine decision settled without falsely claiming PDF functionality is already implemented.

- Fact:
  - Versioned domain models were created before save/load behavior.
  - Assessment:
  - This matches the robust foundation choice and avoids later persistence design drift.

# Limitations

- The `PdfiumEngineAdapter` is a structural boundary only. It does not yet render pages or extract text.
- The PDF workspace is still a shell. It does not yet open documents, navigate pages, or place annotation overlays.
- The new versioned document/session models are not yet wired into save/load commands.
- Rust currently reports dead-code warnings because the new domain models are defined before the dependent features are implemented.

# Next steps

1. Complete `add-tablet-pressure-and-stroke-width` so Canvas Mode uses real pointer pressure and controllable stroke thickness on top of the new app shell.
2. Complete `build-pdf-mode-shell` properly by wiring PDF open/navigation scaffolding through the new backend contracts instead of keeping the PDF workspace as a placeholder.
3. Complete `add-save-load-project-files` so the new versioned canvas/PDF models are exercised by real persistence flows rather than only existing as schemas.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Rust backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Inspect the key architecture outputs:
   - frontend shell: `src/app/shell.ts`
   - frontend workspaces: `src/features/canvas/workspace.ts`, `src/features/pdf/workspace.ts`
   - backend commands: `src-tauri/src/app/commands.rs`
   - backend PDF adapter boundary: `src-tauri/src/infrastructure/pdf_engine/mod.rs`
