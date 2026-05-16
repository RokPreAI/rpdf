# Title

Project save-file size first-pass optimization report

# Context

- Problem:
  - The user reported that a project file containing only two strokes was already around `46K`, which suggested too much fixed serialization overhead for small canvases.
  - The same report said adding two images barely changed the total size, which pointed more toward verbose stroke/save metadata than toward image payload growth in that specific case.
- Constraints:
  - This needed to stay a bounded first-pass storage optimization, not a full save-format redesign or compression system.
  - The repo still has unrelated local changes in `src/app/shell.ts`, `src/styles.css`, `TODO.md`, `.gitignore`, and generated `dist/` files, so the commit needed to stay scoped to document serialization only.

# Goals

- Primary success criteria:
  - reduce JSON project save size materially without breaking load compatibility
  - remove obviously redundant payload rather than inventing a new storage layer
  - keep save/load round-trip behavior intact
- Secondary success criteria:
  - add focused verification that the optimized canvas document still serializes and deserializes correctly

# Approach

- Chosen approach:
  - stop pretty-printing saved project documents and write compact JSON instead
  - omit stroke ids in saved canvas files because the loader already regenerates them
  - omit stroke order when it matches the natural default reconstruction path
  - omit per-point pressure when it is the default value `1.0`
- Why this was the right first pass:
  - it removes repeated overhead from every save immediately
  - it does not require a new file version, migration framework, or binary codec
  - it preserves the existing load path by relying on defaults the loader already understands
- Rejected option:
  - adding compression infrastructure or a new packed geometry format would likely save more space, but it would be a larger contract change than this first-pass task called for

# Implementation

- Task hash:
  - `optimize-project-save-file-size`
- Matching task file:
  - [todos/optimize-project-save-file-size](/home/rok/sync/ideas/rpdf2/todos/optimize-project-save-file-size:1)
- Architecture / flow:
  - Canvas export now saves strokes without persisted ids, and it only persists stroke order when the order differs from the natural default that import already knows how to reconstruct.
  - The Rust canvas domain now treats point pressure as defaulting to `1.0` and skips serializing that field when the value is the default.
  - Rust document saving now writes compact JSON bytes instead of pretty-printed JSON text, so the whole saved project loses whitespace-heavy fixed overhead.
- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:3803)
    - stops persisting stroke ids
    - omits default stroke order during canvas export
  - [src-tauri/src/domain/canvas.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/domain/canvas.rs:26)
    - marks optional stroke metadata so `None` fields are not serialized
    - adds default-pressure semantics so `pressure: 1.0` is omitted from saved JSON
    - adds a focused serialization test to prove those fields are actually omitted
  - [src-tauri/src/app/services.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/app/services.rs:339)
    - changes project document writing from `serde_json::to_string_pretty(...)` to compact `serde_json::to_vec(...)`

# Results

- Outputs:
  - Saved project files are now more compact in three ways:
    - no pretty-print whitespace
    - no serialized stroke ids
    - no serialized default stroke pressure or default stroke order
  - Load compatibility is preserved because the import path already regenerates missing stroke ids and already falls back to natural stroke order when order is absent.
- Verification:
  - Ran `cargo test --manifest-path src-tauri/Cargo.toml canvas_document_`
    - result: success
    - verified:
      - `canvas_document_deserializes_camel_case_payload`
      - `canvas_document_serialization_omits_default_pressure_and_empty_stroke_metadata`
  - Ran `npm run build`
    - result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - result: success

# Decisions

- Fact:
  - The existing loader already supports missing stroke ids and missing stroke order through fallback logic.
- Assessment:
  - That made stroke metadata elision a safe optimization target for the first pass.
- Fact:
  - Compact JSON alone removes fixed whitespace overhead across every saved file.
- Assessment:
  - This is the lowest-risk way to shrink all project saves immediately, even before any deeper geometry packing work.

# Limitations

- I did not recreate and remeasure the user's exact `46K` two-stroke save file inside this turn, so I cannot claim a numeric delta against that exact live sample yet.
- This slice focuses on canvas project saves. It does not redesign PDF session save payloads.
- Pasted images and imported PDF page placements still save their `assetPath` values as before, so any future large-image optimization would need a separate task.
- `TODO.md` was not updated in this commit because it already contains unrelated local backlog edits.

# Next steps

1. Manually save the same small canvas again and compare the resulting file size against the earlier baseline.
   - This confirms the practical impact on the exact user-observed case.
2. If save files are still too large, take a second-pass storage task focused on geometry packing or raster asset handling.
   - The current slice removed obvious redundancy first.
3. If you want to continue interaction work instead, [todos/add-fit-view-space-shortcut](/home/rok/sync/ideas/rpdf2/todos/add-fit-view-space-shortcut:1) is the next bounded canvas task in the visible queue.

# Reproducibility

1. Run the focused Rust serialization checks:
   - `cargo test --manifest-path src-tauri/Cargo.toml canvas_document_`
2. Build the frontend:
   - `npm run build`
3. Check the Tauri/Rust backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
4. Launch the app, save a small canvas project, and compare the file size with an earlier save from before this change.
