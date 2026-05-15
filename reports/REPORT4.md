# Title

Versioned save/load flows for canvas projects and PDF study sessions report

# Context

`TODO.md` moved from the PDF workspace shell to `add-save-load-project-files`. The app already had versioned domain models and two distinct workspaces, but none of that state could survive a restart. Canvas strokes, pasted images, PDF annotations, and PDF session state all remained in memory only.

The constraints for this cycle were:

- use the versioned internal document models rather than raw UI snapshots
- keep the save/load flow usable without introducing new native file-picker plugins yet
- wire both modes through Rust file I/O so later autosave and recovery can build on the same path
- keep the scope bounded enough to verify with build checks in one pass

# Goals

- Primary success criteria:
  - add explicit save/load controls in the app shell
  - allow Canvas Mode to save and reload a document file
  - allow PDF Mode to save and reload a study-session file
  - serialize and deserialize the versioned internal models through Rust commands

- Secondary success criteria:
  - preserve enough image and annotation state that reloaded documents are meaningfully usable
  - avoid forcing a redesign of the existing workspace split

# Approach

- Chosen approach:
  - Extend the workspace controller contract so each workspace can export a versioned document snapshot and import one back.
  - Add narrow Rust JSON file commands for canvas and PDF session save/load.
  - Add a simple shared path field plus Save/Load actions in the shell header, with the current mode determining which backend command to use.
  - Upgrade the canvas image pipeline to store pasted images as data URLs so reloaded canvas documents can restore them without external asset management.

- Rejected options:
  - Saving arbitrary frontend state blobs would violate the explicit versioned-model requirement.
  - Deferring save/load until after autosave or selection work would keep the new document models effectively untested.
  - Introducing native picker plugins in the same pass would widen scope into dependency and permission work that is not required for the core persistence boundary.

# Implementation

- Architecture / flow:
  - `WorkspaceController` now supports:
    - `exportDocument()`
    - `importDocument()`
  - `src/app/shell.ts` now owns a mode-aware file-path field and Save/Load buttons. It routes the active workspace snapshot through the matching Rust command:
    - `save_canvas_project`
    - `load_canvas_project`
    - `save_pdf_study_session`
    - `load_pdf_study_session`
  - The Rust service layer writes and reads pretty-printed JSON using the existing versioned models as the on-disk contract.

- Key files or components:
  - `src/app/types.ts`
  - `src/app/shell.ts`
  - `src/features/canvas/workspace.ts`
  - `src/features/pdf/workspace.ts`
  - `src-tauri/src/domain/canvas.rs`
  - `src-tauri/src/domain/pdf.rs`
  - `src-tauri/src/contracts/dto.rs`
  - `src-tauri/src/app/services.rs`
  - `src-tauri/src/app/commands.rs`
  - `src-tauri/src/lib.rs`
  - `TODO.md`

- Example:
  - In Canvas Mode, the shell can now save a document to a path such as `/tmp/canvas-project.rpdf.json`, with strokes and pasted images serialized into the internal model.
  - In PDF Mode, the shell can save a study session containing the source PDF path, current page index, annotation layers, and reading-cache state, then reload that session back into the PDF workspace.

# Results

- Outputs:
  - The shell now exposes explicit Save/Load actions with a mode-aware path field.
  - Canvas documents now export/import versioned stroke and image state.
  - PDF workspaces now export/import versioned study-session state.
  - Rust now owns JSON file persistence for both document kinds.
  - `TODO.md` now marks `add-save-load-project-files` as done and advances the current task to `add-selection-and-move-tools`.

- Metrics or observations:
  - Pasted canvas images are now converted to data URLs before insertion, which makes them persistable without a separate asset store.
  - The PDF session model now includes `current_page_index`, which was missing from the earlier domain shape and was necessary for reopening a real study session faithfully.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - Save/load currently uses a manual path field in the shell instead of a native dialog.
  - Assessment:
  - This keeps the storage boundary working immediately and leaves file-picker UX as a later enhancement rather than a blocker.

- Fact:
  - Canvas image persistence stores pasted content inline as data URLs.
  - Assessment:
  - This is the simplest reliable way to make pasted images reopen correctly without designing a separate binary-asset subsystem in the same cycle.

- Fact:
  - The frontend workspace controller contract was extended instead of centralizing all document state in the shell first.
  - Assessment:
  - That was the narrower change because each mode already owns its own data model and interaction logic.

# Limitations

- Save/load was verified by compile/build checks, not by an automated end-to-end desktop interaction test in this cycle.
- The shell does not yet validate file extensions or suggest default save locations.
- Canvas background-pattern loading is not yet user-visible because background selection is still fixed in the current canvas UI.
- PDF session reload still depends on the existing backend’s ability to reopen the source PDF path on the current machine.

# Next steps

1. Complete `add-selection-and-move-tools` so canvas documents become editable after reload instead of only drawable/erasable.
2. Complete `add-autosave-and-recovery` on top of the new file persistence path so interrupted sessions do not depend on manual saves alone.
3. Complete `add-svg-export-eligibility` so the saved canvas model can feed a meaningful export path rather than staying an internal-only format.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Rust backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Inspect the shell save/load routing:
   - `src/app/shell.ts`
4. Inspect the Rust file persistence helpers:
   - `src-tauri/src/app/services.rs`
