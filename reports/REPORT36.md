# Title

Editable app-config file report

# Context

The selected worker slice was `add-editable-config-file` from [todos/add-editable-config-file](/home/rok/sync/ideas/rpdf2/todos/add-editable-config-file).

This was the right next action because:

- it was the current priority-2 task in [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md)
- the user explicitly asked for a real editable config file covering palette, background, pattern, and shortcuts
- the existing app already had stable CSS-variable and shortcut surfaces, so this could be implemented as a bounded bootstrap/config layer instead of a full settings UI

# Goals

- Primary success criteria:
  - create a real user-editable config file on disk
  - load theme, canvas background pattern, and shortcut settings from that file at app startup
  - fail closed with defaults if the file is missing, unreadable, or invalid

- Secondary success criteria:
  - keep the change narrow and avoid turning it into a full preferences subsystem
  - keep the existing localStorage preferences working for unrelated per-user runtime settings

# Approach

- Chosen approach:
  - add a backend bootstrap-time config loader that resolves a config path, writes a default file if one is missing, and returns a fully defaulted config object to the frontend
  - add a small frontend config module that applies theme variables to `:root`
  - have Canvas Mode read the loaded config for background pattern and shortcut mappings

- Rejected options:
  - a frontend-only file hack would not have given the app a real cross-run config bootstrap path
  - replacing the existing localStorage preference logic entirely would have widened the slice too much

# Implementation

- Task hash:
  - `add-editable-config-file`

- Architecture / flow:
  - Extended backend bootstrap so it now resolves `config.json`, creates it with defaults when absent, parses it with nested defaults, and reports warnings when fallback behavior is used.
  - Extended `AppBootstrap` to carry `appConfig`, `appConfigPath`, and `appConfigWarnings`.
  - Added a frontend app-config module to apply theme variables and expose the loaded config to workspaces.
  - Updated Canvas Mode so background color, grid color, background pattern, and keyboard shortcuts come from the loaded app config instead of only hardcoded values.
  - Kept existing runtime preferences like stroke width and input quality in local storage; this slice only moved the requested stable app-level config into a file.

- Key files or components:
  - [src-tauri/src/contracts/dto.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/contracts/dto.rs)
    - added app-config DTOs and bootstrap fields
  - [src-tauri/src/app/services.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/app/services.rs)
    - added config-path resolution
    - added default-file creation and parse/fallback behavior
  - [src/app/types.ts](/home/rok/sync/ideas/rpdf2/src/app/types.ts)
    - added frontend app-config types
  - [src/app/config.ts](/home/rok/sync/ideas/rpdf2/src/app/config.ts)
    - added active-config storage and theme application
  - [src/app/shell.ts](/home/rok/sync/ideas/rpdf2/src/app/shell.ts)
    - applies loaded config before workspace mount
    - surfaces config warnings in backend status
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts)
    - loads background pattern and shortcut mappings from app config
    - updates visible shortcut labels to match configured keys
  - [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md)
    - marked `add-editable-config-file` done
    - promoted `remove-mode-switch-restore-copy` to current
  - [todos/add-editable-config-file](/home/rok/sync/ideas/rpdf2/todos/add-editable-config-file)
    - marked done
  - [todos/remove-mode-switch-restore-copy](/home/rok/sync/ideas/rpdf2/todos/remove-mode-switch-restore-copy)
    - marked current

# Results

- Outputs:
  - The app now has a real bootstrap config file path on disk, with default creation when the file does not exist.
  - Theme palette values, app background values, canvas background pattern, and canvas shortcut mappings are now file-driven instead of hardcoded-only.
  - Invalid or unreadable config content falls back to defaults and exposes a visible `config fallback` status hint plus detailed warnings in the backend-status tooltip and console.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - The config file is loaded during backend bootstrap and then applied by the shell before workspace mount.
  - Assessment:
  - This ensures the theme and shortcut values are available early enough for workspace initialization without adding a dynamic settings UI.

- Fact:
  - Existing localStorage preferences were not removed.
  - Assessment:
  - This keeps the slice narrow and avoids mixing app-wide config-file work with unrelated runtime preference migration.

# Limitations

- Live end-to-end startup validation of config-file creation and invalid-file fallback was not performed in this turn.
- This first pass only moves the requested stable app-level settings into the config file; it does not yet migrate PDF speech-rate preferences or other local runtime knobs.
- The config path is surfaced through backend status text/title rather than a dedicated settings screen.

# Next steps

1. Implement `remove-mode-switch-restore-copy`.
2. Manually launch the app once to confirm the config file is created where expected and that editing it affects the next startup.
3. If needed later, decide whether additional per-mode preferences should migrate from local storage into the same config file.

# Reproducibility

1. Launch the desktop app so bootstrap runs.
2. Inspect the config file path shown in the backend-status tooltip.
3. Edit the generated config file to change palette colors, canvas background pattern, or shortcut keys.
4. Restart or reload the app and confirm the changes apply.
5. Break the JSON intentionally and confirm the app falls back to defaults while showing `config fallback`.
6. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
