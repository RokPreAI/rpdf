# Title

SVG export hardening report

# Context

The selected worker slice was `harden-svg-export-and-save-path` from [todos/harden-svg-export-and-save-path](/home/rok/sync/ideas/rpdf2/todos/harden-svg-export-and-save-path).

This was the right next action because:

- it was the current priority-2 export task in [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md)
- the user had explicitly asked for SVG export testing and a selectable save destination
- the old browser-style download flow did not let the user intentionally choose where the SVG file would go

# Goals

- Primary success criteria:
  - keep the existing SVG eligibility checks honest
  - replace the implicit browser download with an explicit save-path choice
  - preserve clear failure and cancel behavior

- Secondary success criteria:
  - keep the export entrypoint in Canvas Mode narrow
  - avoid widening the task into a generic export framework

# Approach

- Chosen approach:
  - keep SVG markup creation in the canvas workspace
  - move the actual file save step into a new native backend command
  - use a save dialog with an SVG filter and suggested filename so the user explicitly chooses the destination

- Rejected options:
  - keeping the `<a download>` blob path would still leave destination choice ambiguous and browser-managed
  - introducing a full export manager or shell-level modal would have widened the task too much

# Implementation

- Task hash:
  - `harden-svg-export-and-save-path`

- Architecture / flow:
  - Added a new backend request DTO and command for SVG export saving.
  - Implemented a native save-file path using `rfd::FileDialog`, defaulting the file name to `.svg` and writing the chosen path on the Rust side.
  - Updated the canvas export function to call the backend command instead of triggering a browser download object URL.
  - Preserved the current export eligibility rules for raster and mixed-content cases, but now report success, cancellation, or failure back through the existing export-state message path.

- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts)
    - replaced blob download export with backend save invocation
    - added export status messaging for success, cancellation, and failure
  - [src-tauri/src/contracts/dto.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/contracts/dto.rs)
    - added `SaveSvgExportRequestDto`
  - [src-tauri/src/app/commands.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/app/commands.rs)
    - added `save_svg_export` command
  - [src-tauri/src/app/services.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/app/services.rs)
    - added native save-dialog + write flow for SVG export
  - [src-tauri/src/lib.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/lib.rs)
    - registered the new command
  - [src-tauri/Cargo.toml](/home/rok/sync/ideas/rpdf2/src-tauri/Cargo.toml)
    - added `rfd` for the native save-file dialog path
  - [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md)
    - marked `harden-svg-export-and-save-path` done
    - promoted `add-excalidraw-style-tool-and-color-shortcuts` to current
  - [todos/harden-svg-export-and-save-path](/home/rok/sync/ideas/rpdf2/todos/harden-svg-export-and-save-path)
    - marked done
  - [todos/add-excalidraw-style-tool-and-color-shortcuts](/home/rok/sync/ideas/rpdf2/todos/add-excalidraw-style-tool-and-color-shortcuts)
    - marked current

# Results

- Outputs:
  - Exporting SVG now opens a native save dialog with an SVG filter and suggested filename.
  - The chosen export destination is written intentionally by the backend instead of relying on browser download behavior.
  - Cancelled exports remain non-destructive and return a clear status message.
  - Existing ineligible export cases still stay honest for raster or mixed-content selections.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - The save path is now chosen through a native dialog in the backend.
  - Assessment:
  - This directly satisfies the task requirement without redesigning the shell or adding a generic file picker abstraction.

- Fact:
  - SVG markup creation stayed in the canvas workspace and only file selection/writing moved to Rust.
  - Assessment:
  - This keeps the change local and preserves the current export eligibility logic.

# Limitations

- Manual end-to-end runtime export validation through the live desktop dialog was not performed in this turn.
- The status message currently feeds back through the existing export-state message channel rather than a dedicated export toast or shell notification.
- This slice only hardens SVG export, not other export formats.

# Next steps

1. Implement `add-excalidraw-style-tool-and-color-shortcuts`.
2. If manual testing shows the export status feedback is too subtle, add a dedicated shell-visible success/error surface later instead of widening this slice now.
3. Re-test SVG export manually against full-canvas vector export, selected-vector export, and ineligible mixed-content selections.

# Reproducibility

1. Open Canvas Mode.
2. Draw at least one vector stroke or shape.
3. Click `Export SVG`.
4. Confirm a native save dialog appears with an SVG filename suggestion.
5. Save the file and confirm the resulting path is chosen intentionally.
6. Try again with an ineligible mixed-content selection and confirm export remains disabled with an honest message.
7. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
