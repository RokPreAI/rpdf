# Title

SVG export direct-path flow repair report

# Context

- Problem:
  - Canvas SVG export did not honor the path visible in the shared header field.
  - The frontend exposed a path input that already drives project save/load, so users reasonably expected `Export SVG` to write to that same explicit destination when they provided one.
  - The actual backend behavior always opened a native save dialog and only used a suggested filename, which made the visible contract misleading.
- Constraints:
  - This needed to stay a bounded export-contract slice.
  - The repo still has unrelated local edits in `TODO.md`, `.gitignore`, and generated `dist/` churn, so the commit has to stay path-limited.

# Goals

- Primary success criteria:
  - make `Export SVG` save directly to the header path when the user provides an explicit `.svg` destination
  - keep dialog-based export available when no path is provided
  - surface the real saved path or the real failure in visible UI, not only in button hover state
- Secondary success criteria:
  - keep the export change local to the existing shell/canvas/backend boundary
  - avoid breaking project JSON save/load behavior

# Approach

- Chosen approach:
  - thread the header path from the shell into the canvas export request
  - extend the backend export DTO so it can accept an optional explicit file path
  - use a direct write path when that explicit path exists, and fall back to the native save dialog only when it does not
  - reject non-`.svg` explicit paths early with a clear visible message so the contract stays unambiguous
- Rejected option:
  - silently appending `.svg` to arbitrary header paths would still leave the contract fuzzy, especially because that same field is also used for `.json` project files

# Implementation

- Task hash:
  - `fix-svg-export-direct-path-flow`
- Matching task file:
  - [todos/fix-svg-export-direct-path-flow](/home/rok/sync/ideas/rpdf2/todos/fix-svg-export-direct-path-flow:1)
- Architecture / flow:
  - The shell now sends the current header path with the `rpdf:request-canvas-svg-export` event.
  - The canvas workspace forwards that optional explicit path to the backend `save_svg_export` command.
  - The backend now has two export modes:
    - explicit path present and ending in `.svg`: write directly there
    - no explicit path: open the native save dialog as before
  - Export result messages are now dispatched back to the shell and shown in the visible backend-status overlay. Successful exports also write the final saved SVG path back into the header field.
- Key files or components:
  - [src/app/shell.ts](/home/rok/sync/ideas/rpdf2/src/app/shell.ts:387)
    - passes the header path into the canvas export request
    - blocks obvious invalid direct export attempts when the explicit path is not a `.svg` destination
    - shows visible export success/failure messages in the backend-status overlay
    - syncs the header field to the saved SVG path after success
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:1948)
    - accepts export requests with an optional direct path
    - emits result events so shell-level status becomes visible
  - [src-tauri/src/contracts/dto.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/contracts/dto.rs:233)
    - extends `SaveSvgExportRequestDto` with optional `filePath`
  - [src-tauri/src/app/services.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/app/services.rs:130)
    - writes SVG directly to an explicit `.svg` path when one is provided
    - preserves dialog export when no path is provided
    - factors shared file-writing into a reusable helper

# Results

- Outputs:
  - SVG export now respects the visible header path when the user provides an explicit `.svg` destination.
  - If the field is empty, export still falls back to the native save dialog.
  - Success and failure messages are now visible in the shell status overlay instead of being hidden in a button title only.
  - Invalid explicit export destinations now fail clearly with a message telling the user to provide a `.svg` path or clear the field to choose a save location.
- Verification:
  - Ran `npm run build`
    - result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - result: success

# Decisions

- Fact:
  - The header path field is already a core file-path contract in this app.
- Assessment:
  - Reusing it for explicit SVG export is the least surprising behavior, but only when the destination is clearly an SVG path.
- Fact:
  - The same field is also used for JSON project files.
- Assessment:
  - Failing clearly on non-`.svg` explicit paths is safer than trying to guess whether the user intended JSON save/load or SVG export.

# Limitations

- I did not run a live desktop click-through in this turn, so the remaining hands-on confirmation is:
  - enter a real `.svg` path in Canvas Mode and verify the file is written there
  - clear the field and verify dialog fallback still works
- `TODO.md` was not updated in this commit because it already contains unrelated local backlog edits from earlier turns.
- This slice does not change SVG eligibility rules; raster images and imported PDF pages are still intentionally excluded from SVG export.

# Next steps

1. Manually verify direct SVG export with a real path such as `/home/rok/test.svg`.
   - This confirms the exact user-facing contract that motivated the task.
2. Take the next text-entry task, [todos/fix-text-tool-commit-control](/home/rok/sync/ideas/rpdf2/todos/fix-text-tool-commit-control:1).
   - That is the next queued priority-1 task and it addresses a visible editing bug.
3. If export path behavior still feels ambiguous after hands-on use, add a small follow-up task around header copy or button labeling.
   - The implementation is now explicit, but the text affordance can still be tightened if needed.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Tauri/Rust backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch the app in Canvas Mode.
4. Test direct export:
   - type a `.svg` path into the header field
   - press `Export SVG`
   - confirm the file appears at that exact path
5. Test chooser fallback:
   - clear the header field
   - press `Export SVG`
   - confirm the native save dialog appears
