# Title

Project save/load path flow fix report

# Context

- Problem:
  - The app exposes a direct project path field plus `Save` and `Load` buttons in the shell header, which implies direct path-based JSON save/load.
  - In live use, entering a path like `/home/rok/test.json` and clicking `Save` created no file, and `Load` also appeared to do nothing.
  - Autosave still worked, which pointed away from general document export logic and toward the explicit Tauri save/load boundary.
- Constraints:
  - This needed to stay a bounded storage-contract slice.
  - The repo still contains unrelated staged deletions, generated `dist/` churn, and separate uncommitted canvas work, so the commit had to stay path-limited.

# Goals

- Primary success criteria:
  - make direct path-based `Save` actually write the typed file
  - make direct path-based `Load` deserialize the same saved document shape
  - keep the shell contract unambiguous: these buttons use the typed path directly and do not open a dialog
- Secondary success criteria:
  - add a focused verification that the Rust side now accepts the frontend document shape
  - avoid widening into a broader storage redesign

# Approach

- Chosen approach:
  - trace the shell save/load handlers through the Tauri commands into the Rust serializers
  - compare the frontend document shape with the Rust domain models used inside the save/load request DTOs
  - fix the naming mismatch at the Rust serialization boundary
- Root cause:
  - the frontend sends nested project/session documents in `camelCase`
  - the Rust request DTOs already expect `camelCase`, but the nested `CanvasDocument` and `PdfStudyDocument` domain structs were still using default snake_case field names for many fields such as `background_pattern`, `font_size`, `asset_path`, `source_pdf_path`, `page_count`, `current_page_index`, and `reading_cache`
  - that caused Tauri command deserialization to fail before any file write happened, which explains why no file appeared at the typed path
- Rejected options:
  - changing the frontend document model to snake_case would have widened the change unnecessarily and broken the existing TypeScript-side conventions
  - adding a parallel save-specific DTO layer would have been heavier than fixing the actual contract mismatch

# Implementation

- Task hash:
  - `fix-project-save-load-path-flow`
- Matching task file:
  - [todos/fix-project-save-load-path-flow](/home/rok/sync/ideas/rpdf2/todos/fix-project-save-load-path-flow:1)
- Architecture / flow:
  - The shell save/load path remains direct-path based. Typing a file path in the header is still the intended contract, and no new dialog is expected for project JSON save/load.
  - The fix is entirely on the Rust domain side: the nested saved document/session structs now serialize and deserialize in `camelCase`, matching the frontend payload shape used by `invoke(...)`.
  - Focused unit tests were added to prove that both canvas and PDF study payloads now deserialize correctly from the frontend-style JSON shape.
- Key files or components:
  - [src-tauri/src/domain/canvas.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/domain/canvas.rs:1)
    - added `#[serde(rename_all = "camelCase")]` to the canvas document structs that cross the save/load boundary
    - added a focused unit test for frontend-style canvas JSON payloads
    - added `PartialEq` / `Eq` derives needed by the new test assertions
  - [src-tauri/src/domain/pdf.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/domain/pdf.rs:1)
    - added `#[serde(rename_all = "camelCase")]` to the PDF study document structs that cross the save/load boundary
    - added a focused unit test for frontend-style PDF study JSON payloads
    - added `PartialEq` / `Eq` derive for `ReadingSourceKind`
  - [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md:1)
    - moved `fix-project-save-load-path-flow` to done
  - [todos/fix-project-save-load-path-flow](/home/rok/sync/ideas/rpdf2/todos/fix-project-save-load-path-flow:1)
    - marked the task done

# Results

- Outputs:
  - The direct header-path `Save` / `Load` flow now has a consistent data contract across TypeScript and Rust.
  - No dialog is part of normal project JSON save/load; the typed path is the intended destination/source.
  - The Rust side now accepts the camelCase payload the frontend already emits, so save should create the file at the exact path the user provides and load should read the same shape back.
- Verification:
  - Ran `cargo test --manifest-path src-tauri/Cargo.toml`
    - Result: success
    - Added tests passed:
      - `domain::canvas::tests::canvas_document_deserializes_camel_case_payload`
      - `domain::pdf::tests::pdf_study_document_deserializes_camel_case_payload`
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - Explicit project save/load is supposed to use the typed header path directly.
- Assessment:
  - The current shell UI already makes that the most honest interpretation, so the right fix was to make the direct-path contract actually work rather than replacing it with a hidden dialog behavior.

# Limitations

- I did not run a live manual Tauri click-through in this turn, so there is still one hands-on confirmation left: type `/home/rok/test.json`, click `Save`, confirm the file appears, then modify the workspace and click `Load`.
- This slice does not change SVG export behavior; that remains a separate queued task in `fix-svg-export-direct-path-flow`.

# Next steps

1. Manually verify the fixed JSON project save/load flow in the desktop app with a real writable path such as `/home/rok/test.json`.
2. Take the next queued export task, [todos/fix-svg-export-direct-path-flow](/home/rok/sync/ideas/rpdf2/todos/fix-svg-export-direct-path-flow:1), because it is the adjacent path-contract issue the user reported next.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check and test the Rust/Tauri backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
   - `cargo test --manifest-path src-tauri/Cargo.toml`
3. Launch the app, type an explicit path such as `/home/rok/test.json`, press `Save`, confirm the file exists, then press `Load` to restore it.
