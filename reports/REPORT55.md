# Title

Diagnose Speech Dispatcher backend failure

# Context

- Problem:
  - `Read page` was already surfacing a native local-TTS failure, but the message was still too generic for the current machine.
  - The user supplied real Speech Dispatcher logs showing that the daemon starts, but its actual output modules are broken.
- Constraints:
  - This slice needed to stay diagnostic-only and not broaden into adding a second speech engine yet.
  - The repo still had unrelated local edits in [src/app/shell.ts](/home/rok/sync/ideas/rpdf2/src/app/shell.ts:1) and [src/styles.css](/home/rok/sync/ideas/rpdf2/src/styles.css:1), so the commit had to stay narrow.

# Goals

- Primary success criteria:
  - distinguish “Speech Dispatcher is unreachable” from “Speech Dispatcher is running but has no usable speech backend”
  - surface actionable remediation when the runtime matches the current broken-module environment
- Secondary success criteria:
  - capture the exact observed signatures from `/run/user/1000/speech-dispatcher/log/`
  - add focused Rust coverage for the new diagnosis behavior

# Approach

- Chosen approach:
  - keep the existing `spd-say` probing flow
  - augment error formatting by inspecting the active Speech Dispatcher log directory for known broken-backend signatures
- Why this was the right next action:
  - the supplied logs already showed the real failure mode:
    - `espeak-ng` modules fail because `libespeak-ng.so.1` is missing
    - `festival` cannot connect to a Festival server
    - `openjtalk` is missing its voice data
    - only `dummy` survives, and then audio open fails
  - that meant the missing capability was diagnosis quality, not another retry path
- Rejected option:
  - adding a second local TTS backend in this slice would have widened scope and hidden the current environment problem instead of explaining it

# Implementation

- Task hash:
  - `diagnose-speech-dispatcher-backend-failure`
- Matching task file:
  - [todos/diagnose-speech-dispatcher-backend-failure](/home/rok/sync/ideas/rpdf2/todos/diagnose-speech-dispatcher-backend-failure:1)
- Architecture / flow:
  - I left the `spd-say` command path intact.
  - I changed the local error formatter so it first respects explicit stderr details such as unix-socket connection failures.
  - If stderr is empty or clearly Speech-Dispatcher-related but not actionable enough, the adapter now inspects the current runtime log directory and maps known failure signatures to user-facing remediation text.
- Observed signatures recorded from the real machine:
  - `/run/user/1000/speech-dispatcher/log/espeak-ng.log`
    - `libespeak-ng.so.1: cannot open shared object file`
  - `/run/user/1000/speech-dispatcher/log/festival.log`
    - `festival_client: connect to server failed`
  - `/run/user/1000/speech-dispatcher/log/openjtalk.log`
    - missing `nitech_jp_atr503_m001.htsvoice`
  - `/run/user/1000/speech-dispatcher/log/speech-dispatcher.log`
    - `Opening sound device failed. Reason: server audio is not supported.`
- Key files or components:
  - [src-tauri/src/infrastructure/local_tts.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/infrastructure/local_tts.rs:1)
    - added runtime log-path detection under `XDG_RUNTIME_DIR`
    - added signature-based diagnosis for missing `libespeak-ng.so.1`, dummy-only audio failure, Festival server failure, and OpenJTalk voice-data failure
    - kept direct socket/dispatcher stderr details preferred when they already explain the problem
    - added focused unit tests for the new diagnosis mapping
  - [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md:1)
    - moved the task from current to done
  - [todos/diagnose-speech-dispatcher-backend-failure](/home/rok/sync/ideas/rpdf2/todos/diagnose-speech-dispatcher-backend-failure:1)
    - marked the task file done

# Results

- Outputs:
  - the app can now report that Speech Dispatcher is running but unusable because its real speech backends are broken
  - the message now explicitly points at `libespeak-ng.so.1` when that is the detected blocker and explains that only the dummy backend remains
- Verification:
  - `cargo test --manifest-path src-tauri/Cargo.toml local_tts`
  - `cargo check --manifest-path src-tauri/Cargo.toml`
  - `npm run build`

# Decisions

- Fact:
  - `spd-say` existing on disk does not mean local speech is actually usable
- Assessment:
  - the app now treats “daemon reachable” and “usable output backend available” as separate concerns
- Fact:
  - the current environment has several module-specific failures, but the highest-signal one is missing `libespeak-ng.so.1`
- Assessment:
  - remediation text should name that missing dependency first instead of drowning the user in secondary locale-dictionary noise

# Limitations

- The app still relies on Speech Dispatcher for native local TTS in this path.
- I did not add or test a second local speech engine in this slice.
- The log-based diagnosis depends on the standard per-user Speech Dispatcher log directory being present and readable.

# Next steps

1. Implement [todos/add-secondary-local-tts-backend](/home/rok/sync/ideas/rpdf2/todos/add-secondary-local-tts-backend:1) so local reading still works when Speech Dispatcher is installed but broken.
2. Manually run `Read page` again in the desktop app and confirm the in-app error now mentions the missing `libespeak-ng.so.1` backend problem instead of a generic local-speech failure.

# Reproducibility

1. Inspect:
   - [src-tauri/src/infrastructure/local_tts.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/infrastructure/local_tts.rs:1)
   - `/run/user/1000/speech-dispatcher/log/speech-dispatcher.log`
   - `/run/user/1000/speech-dispatcher/log/espeak-ng.log`
   - `/run/user/1000/speech-dispatcher/log/festival.log`
   - `/run/user/1000/speech-dispatcher/log/openjtalk.log`
2. Run:
   - `cargo test --manifest-path src-tauri/Cargo.toml local_tts`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
   - `npm run build`
