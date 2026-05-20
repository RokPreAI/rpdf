# Title

Add canvas text-tool keyboard shortcut `T`

# Context

The canvas workspace already had a text tool button, but keyboard tool switching did not include a text shortcut. The task required adding `T`, exposing it in the visible hinting, and preserving the existing focused-input guard so typing into inputs or text editors is not hijacked.

This repo also already had unrelated uncommitted edits in `TODO.md`, `src/features/canvas/workspace.ts`, `src/app/shell.ts`, `src/app/types.ts`, `src-tauri/src/domain/canvas.rs`, and `src/styles.css`. That made a small, isolated task the safest next action.

# Goals

- Pressing `T` switches to the text tool when canvas shortcuts are active.
- The shortcut does not fire while the existing focused-input guards are blocking canvas shortcuts.
- The toolbar hinting and text tool button label path reflect the `T` shortcut.
- The config-backed tool shortcut model knows about the text tool.
- `npm run build` passes.

# Approach

I chose the smallest coherent slice that satisfied both the frontend behavior and the config-model note in the task file.

That meant updating:
- the frontend tool shortcut defaults and hint text in `src/features/canvas/workspace.ts`
- the TypeScript config type in `src/app/types.ts`
- the Rust DTO default in `src-tauri/src/contracts/dto.rs`

I did not take on any broader canvas input cleanup because the repo already had unrelated in-progress edits and the task only asked for the text shortcut.

# Implementation

- Task hash:
  - `add-text-tool-shortcut-t`
- Matching task file:
  - `todos/add-text-tool-shortcut-t`
- Key files:
  - `TODO.md`
    - moved the task into `# Current TODOs` before implementation, then into `# Done TODOs` after verification
  - `src/app/types.ts`
    - added `text: string` to `AppToolShortcutsConfig`
  - `src-tauri/src/contracts/dto.rs`
    - added `text` to `AppToolShortcutsDto`
    - added a dedicated default function so older config files that do not yet contain `text` still deserialize to `"t"` instead of an empty string
  - `src/features/canvas/workspace.ts`
    - added `text` to the configured tool shortcut map with a `"t"` fallback
    - updated the toolbar shortcut summary from `text via toolbar` to an explicit `${configuredToolShortcuts.text.toUpperCase()} text`
    - updated the text tool button `title` and `aria-label` to show the configured shortcut

The existing keyboard handling already routes configured tool shortcuts through `toolShortcutByKey`, and `Tool` already included `"text"`, so no new event-path logic was needed beyond making the text shortcut available in the configured map.

# Results

Observed results:
- `T` is now part of the configured tool shortcut map used by the canvas keydown handler.
- The toolbar shortcut summary now visibly advertises the text shortcut.
- The text tool button tooltip and accessibility label now include the configured shortcut.
- The config schema now includes `text` on both the TypeScript and Rust sides.
- Older config files missing `text` now default to `t` during Rust deserialization.

Verification:
- `npm run build`
- `cargo check --manifest-path src-tauri/Cargo.toml`

Both commands passed.

# Decisions

- Fact:
  - The existing keyboard handler already supports any configured tool shortcut that resolves to a valid `Tool`.
- Assessment:
  - The right fix was to add the missing text shortcut to the configured tool map and hints, not to rewrite keyboard handling.

- Fact:
  - Older config files may not contain a `text` field yet.
- Assessment:
  - A Rust field-level default for `text` was the narrowest way to preserve backward-compatible behavior without a broader config migration task.

# Limitations

- I did not manually runtime-test the shortcut in the live Tauri window; verification here is build- and type-level plus Rust compile validation.
- The repo still contains unrelated uncommitted work outside this task.
- I did not add any config-file documentation update because the task file did not require docs, only model awareness.

# Next steps

1. Implement `todos/fix-hand-tool-drag-pan-scaling` next.
   - Why: it is the next small canvas interaction fix in `TODO.md` and should improve direct manipulation feel.
   - Depends on: current hand-tool behavior remaining otherwise stable.
2. Manually verify `T` in the running app.
   - Why: confirms the focused-input guard still behaves correctly during real text editing and input focus changes.
   - Depends on: launching the app locally.

# Reproducibility

1. Inspect:
   - `src/features/canvas/workspace.ts`
   - `src/app/types.ts`
   - `src-tauri/src/contracts/dto.rs`
   - `TODO.md`
2. Verify:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Manual runtime check:
   - launch the app
   - focus canvas mode
   - press `T` with canvas shortcuts active and confirm the text tool becomes active
   - focus a text input or text editor and confirm typing `t` does not switch tools
