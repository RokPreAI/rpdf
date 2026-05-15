# Title

Backend status workspace-overlay report

# Context

- Problem:
  - The shared backend-status element was sitting inside the shell header and taking horizontal space away from the main controls.
  - The user explicitly wanted it moved to the bottom-right of the workspace, similar to the canvas toolbar living in the lower workspace corner.
- Constraints:
  - This needed to stay a narrow layout slice.
  - The repo still contains unrelated staged deletions and generated `dist/` churn, so the commit had to remain path-limited.

# Goals

- Primary success criteria:
  - move backend status out of the header
  - anchor it to the bottom-right of the workspace
  - keep it non-interactive so it does not block canvas or PDF interactions
- Secondary success criteria:
  - preserve readable behavior on smaller windows
  - avoid changing backend-status logic or text content

# Approach

- Chosen approach:
  - move the `#backend-status` node into `#workspace-root`
  - style it as an absolute-positioned overlay badge inside the workspace container
- Rejected options:
  - leaving the element in the header and trying to fake the position with margins would still make it part of header layout
  - duplicating separate status elements per mode would have widened the slice unnecessarily

# Implementation

- Task hash:
  - `move-backend-status-overlay`
- Matching task file:
  - [todos/move-backend-status-overlay](/home/rok/sync/ideas/rpdf2/todos/move-backend-status-overlay:1)
- Architecture / flow:
  - The shell now mounts `#backend-status` inside `#workspace-root` instead of inside `.header-actions`.
  - The status uses absolute positioning against the already-relative workspace root.
  - On smaller windows, the overlay stretches across the bottom for readability instead of staying pinned to a narrow right column.
- Key files or components:
  - [src/app/shell.ts](/home/rok/sync/ideas/rpdf2/src/app/shell.ts:56)
    - moved `#backend-status` from the header action group into the workspace section
  - [src/styles.css](/home/rok/sync/ideas/rpdf2/src/styles.css:132)
    - turned `.backend-status` into a bottom-right overlay badge
    - kept it non-interactive with `pointer-events: none`
    - added narrow-screen overlay positioning

# Results

- Outputs:
  - Backend status no longer consumes header layout space.
  - The status now appears in the bottom-right of the workspace on larger layouts.
  - On narrower screens, the status expands across the bottom edge for readability.
- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - The status remains a shared shell-level element rather than becoming mode-specific.
  - Assessment:
  - That keeps the behavior consistent across Canvas and PDF while still solving the layout problem the user reported.

# Limitations

- I did not manually inspect the live desktop window in this turn, so the remaining check is hands-on spacing against the canvas toolbar and PDF footer in the running app.
- This slice does not change status wording or update timing.

# Next steps

1. Manually verify the overlay position in both Canvas Mode and PDF Mode.
2. If the status visually collides with other lower-corner controls in a specific mode, add a follow-up task for per-mode offset tuning instead of widening this slice now.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Rust/Tauri backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch the app, switch between Canvas and PDF, and confirm the backend status sits at the workspace bottom-right instead of in the header.
