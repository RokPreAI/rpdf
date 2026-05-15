# Title

Read-page runtime TTS failure diagnosis and fallback report

# Context

- Problem:
  - The PDF reading pipeline could extract text successfully, but `Read page` still failed at runtime instead of speaking.
  - The user reported that the failure surfaced as an error path rather than usable playback.
- Constraints:
  - This needed to stay a bounded reading-runtime repair slice.
  - The app already had both a native local speech path and a browser/webview fallback path, so the fix needed to diagnose the failing boundary instead of replacing the whole reading architecture.
  - The repo already had unrelated local backlog edits in `TODO.md` and `.gitignore`, so the commit needed to avoid bundling those files.

# Goals

- Primary success criteria:
  - identify why `Read page` fails at runtime after successful text extraction
  - stop treating a broken native speech backend as available
  - show the real local speech failure in the app instead of a vague generic fallback message
  - keep browser speech fallback available when the runtime actually supports it
- Secondary success criteria:
  - add focused regression coverage for the new error formatting path
  - keep the fix local to the reading/TTS boundary

# Approach

- Chosen approach:
  - reproduce the native backend failure outside the UI using the same `spd-say` path the app uses
  - tighten backend availability from “binary exists” to “Speech Dispatcher connection actually works”
  - thread the native error message back into the frontend reading status before deciding whether fallback is possible
- Root cause found during reproduction:
  - `spd-say` is installed on this machine
  - but `spd-say` fails immediately because it cannot connect to Speech Dispatcher
  - the previous code treated `spd-say` existence as availability, so the UI always attempted native speech first and only showed a generic fallback path after failure
- Rejected option:
  - swapping immediately to a cloud TTS provider would have mixed a runtime backend repair with a larger product decision around privacy, timing metadata, and offline behavior

# Implementation

- Task hash:
  - `fix-read-page-runtime-tts-error`
- Matching task file:
  - [todos/fix-read-page-runtime-tts-error](/home/rok/sync/ideas/rpdf2/todos/fix-read-page-runtime-tts-error:1)
- Architecture / flow:
  - The Rust local TTS adapter now validates that `spd-say` can actually talk to Speech Dispatcher by probing `spd-say -O` instead of only checking `command -v spd-say`.
  - Connection failures are normalized into compact actionable messages so the UI can surface a reason such as “could not connect to Speech Dispatcher” instead of leaking a long opaque stderr dump.
  - The PDF workspace now preserves the native error message and only falls back to browser speech when `speechSynthesis` is actually available.
  - If both native and browser speech are unavailable, the reading status now reports both facts directly in-app.
- Key files or components:
  - [src-tauri/src/infrastructure/local_tts.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/infrastructure/local_tts.rs:1)
    - replaced the old existence-only availability check with a real backend-status probe
    - normalized Speech Dispatcher failures into shorter actionable messages
    - added focused unit tests for connection-error and generic-error formatting
  - [src/features/pdf/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/pdf/workspace.ts:579)
    - added browser speech availability and error-description helpers
    - changed `speakCurrentPage()` to surface the native error directly
    - made fallback conditional on actual browser speech support instead of assuming it exists

# Results

- Outputs:
  - The app no longer treats “`spd-say` binary exists” as proof that local speech is usable.
  - A broken Speech Dispatcher connection now produces a direct in-app reason instead of the old vague “native local speech was unavailable” message.
  - Browser/webview fallback still remains available when supported by the runtime.
- Verification:
  - Reproduced the real backend failure directly with:
    - `spd-say --wait --application-name rpdf --rate 0 "rpdf speech test"`
    - observed result: `spd-say` failed because it could not connect to Speech Dispatcher
  - Ran `cargo test --manifest-path src-tauri/Cargo.toml local_tts`
    - result: success
    - added tests passed:
      - `infrastructure::local_tts::tests::formats_dispatcher_connection_errors_compactly`
      - `infrastructure::local_tts::tests::formats_generic_errors_compactly`
  - Ran `npm run build`
    - result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - result: success

# Decisions

- Fact:
  - The immediate runtime problem was not text extraction. It was native speech backend viability detection.
- Assessment:
  - Tightening availability detection and error propagation was the smallest correct fix because it improves both behavior and diagnosability without changing the reading architecture.
- Fact:
  - Browser speech may not exist in every Tauri/WebKitGTK runtime.
- Assessment:
  - Fallback must stay conditional and explicit. The app should not pretend fallback exists if the runtime does not expose it.

# Limitations

- I did not perform live audible playback verification in the desktop app in this turn, so the remaining hands-on question is whether your actual user session can either:
  - speak through browser fallback, or
  - connect successfully to Speech Dispatcher after this clearer failure handling
- The current fix improves native backend detection and UI feedback, but it does not add a second local CLI speech backend such as `espeak-ng`.
- `TODO.md` was not updated in this commit because it already contained unrelated local backlog edits that did not belong to this worker slice.

# Next steps

1. Run the desktop app and press `Read page` on a page with extracted text.
   - This confirms whether browser fallback actually speaks in your runtime or whether the app now shows the clearer Speech Dispatcher reason.
2. If the app still cannot speak, capture the exact new in-app reading status message.
   - That will tell us whether the next slice is a Speech Dispatcher/session fix or a browser fallback/runtime support issue.
3. If you want a stronger local fallback after this, add a separate task for a second local speech backend.
   - That should stay separate from the current runtime-diagnosis repair.

# Reproducibility

1. Reproduce the native speech backend failure directly:
   - `spd-say --wait --application-name rpdf --rate 0 "rpdf speech test"`
2. Run the focused Rust verification:
   - `cargo test --manifest-path src-tauri/Cargo.toml local_tts`
3. Build the frontend:
   - `npm run build`
4. Check the Tauri/Rust backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
5. Launch the app, open a readable PDF page, and press `Read page` to verify the new status behavior in the real desktop runtime.
