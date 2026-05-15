# Title

Read page native TTS fallback report

# Context

The current priority-1 backlog item was `fix-read-page-action`. The user reported that pressing `Read page` in PDF Mode appeared to do nothing.

The existing implementation depended entirely on browser `speechSynthesis` from inside the Tauri webview. That is a weak fit for a desktop app because webview speech support varies by runtime and platform. In this repo, the reading pipeline already existed for text extraction and OCR fallback, but the final text-to-speech step was still webview-only.

Constraints that shaped the fix:

- keep the change local to reading support instead of mixing it with the separate PDF viewport, annotation, or mode-persistence regressions
- preserve the existing Tauri command and service boundary instead of adding ad hoc shell logic in the frontend
- avoid introducing a new Cargo dependency when the machine already provides a working local speech command
- keep browser speech as a fallback so the app does not become Linux-only just because this slice uses `spd-say`

# Goals

- Primary success criteria:
  - make `Read page` trigger an observable speech path instead of a silent no-op
  - route speech through a native local backend when available
  - keep `Stop` meaningful for the native path

- Secondary success criteria:
  - keep the browser speech path as a fallback instead of deleting it
  - surface a clear status message when neither native nor browser speech is available
  - update the backlog so the `fix-read-page-action` task is recorded as done

# Approach

- Chosen approach:
  - Add two new Tauri commands: one to speak text locally and one to stop local speech.
  - Back those commands with a small infrastructure adapter that invokes `spd-say`.
  - Update the PDF workspace so `Read page` prefers native local speech first, then falls back to browser speech only if native speech fails or is unavailable.
  - Track an active speech request identifier in the frontend so stale completions from a cancelled request cannot overwrite the current reading state.

- Rejected options:
  - Leaving the button on browser `speechSynthesis` only would not address the original failure mode in a Tauri runtime.
  - Introducing a larger Rust TTS dependency would have widened scope unnecessarily for a bounded worker slice.
  - Implementing this together with mode persistence, annotation visibility, or PDF layout fixes would have mixed unrelated regressions into one hard-to-review change set.

# Implementation

- Architecture / flow:
  - The PDF workspace still owns text selection, reliability state, and read/stop button behavior.
  - A new native speech path now sits behind the Tauri invoke boundary.
  - The runtime order for `Read page` is now:
    1. gather the currently readable page text from native extraction or OCR fallback
    2. try `speak_text_locally` through Tauri
    3. if native speech fails, fall back to browser `speechSynthesis`
    4. if neither path is available, show a clear status message

- Key files or components:
  - `src-tauri/src/infrastructure/local_tts.rs`
    - new native speech adapter built on `spd-say`
    - exposes `speak_text()` and `stop_speaking()`
  - `src-tauri/src/contracts/dto.rs`
    - adds `SpeakTextRequestDto` for text + rate transport
  - `src-tauri/src/app/services.rs`
    - exposes the local speech operations through the application service layer
  - `src-tauri/src/app/commands.rs`
    - adds `speak_text_locally` and `stop_local_speech` Tauri commands
  - `src-tauri/src/lib.rs`
    - registers the new commands with the Tauri invoke handler
  - `src/features/pdf/workspace.ts`
    - now prefers native speech
    - keeps browser speech as fallback
    - adds request-id tracking so stop/cancel cannot be overwritten by a stale completion
  - `TODO.md`
    - marks `fix-read-page-action` as done

- Example:
  - If the page text exists and `spd-say` is available, clicking `Read page` now starts local system speech and updates the status line to show that reading started.
  - If native speech is unavailable but browser speech works, the app continues with the webview path instead of failing immediately.

# Results

- Outputs:
  - `Read page` now has a native local speech path instead of depending only on the webview.
  - `Stop` now explicitly stops the native speech path through `spd-say --stop` when native speech is active.
  - The reading panel now reports a clear availability/fallback message instead of looking inert when local speech is missing.
  - `TODO.md` now records `fix-read-page-action` as completed.

- Metrics or observations:
  - The machine used for this slice has `/usr/bin/spd-say` available, which made a real native fix possible without adding new crates.
  - The change stayed within the reading/TTS boundary and did not modify the separate canvas, layout, or PDF rendering tasks.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success
  - Ran `spd-say --help`
    - Result: confirmed the local backend supports `--wait` and `--stop`, which the new command path relies on
  - Full end-to-end audible playback through the desktop app was not verified in this turn.

# Decisions

- Fact:
  - Native speech is implemented with the system `spd-say` command.
  - Assessment:
  - This is the smallest real desktop-native fix for the reported regression on this machine.

- Fact:
  - Browser speech remains as a fallback instead of being removed.
  - Assessment:
  - Keeping the old path preserves some cross-runtime resilience while still preferring the more reliable desktop path.

- Fact:
  - Stop/cancel is coordinated in the frontend with an incrementing request identifier.
  - Assessment:
  - This prevents an older speech request from incorrectly setting the UI back to `Reading finished.` after the user already stopped it.

# Limitations

- The native implementation currently assumes `spd-say` as the local backend. Other machines may still rely on browser fallback until a broader TTS abstraction is added.
- Full audible runtime verification inside the launched Tauri app was not performed in this turn, so the command path is build-verified rather than manually playback-verified.
- The PDF workspace still has separate open backlog items for annotation visibility, viewport containment, recolor-control layout, and mode-state persistence. This slice intentionally did not address them.

# Next steps

1. Implement `preserve-mode-state-across-switches` because it is the highest-priority workflow break still open and blocks normal use of both modes together.
2. Implement `fix-pdf-mode-annotations` because missing visible annotations undermine one of the core PDF-mode promises even when reading now works better.
3. Manually test `Read page` and `Stop` in the real Tauri app and capture any runtime-specific failures before expanding TTS behavior further.

# Reproducibility

1. Confirm the local speech backend exists:
   - `command -v spd-say`
2. Inspect the native speech adapter:
   - `src-tauri/src/infrastructure/local_tts.rs`
3. Inspect the Tauri command wiring:
   - `src-tauri/src/app/commands.rs`
   - `src-tauri/src/app/services.rs`
   - `src-tauri/src/lib.rs`
4. Inspect the PDF workspace read/stop logic:
   - `src/features/pdf/workspace.ts`
5. Rebuild the frontend:
   - `npm run build`
6. Recheck the Rust side:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
