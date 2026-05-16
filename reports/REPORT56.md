# Title

Add secondary local TTS backend

# Context

- Problem:
  - the previous slice proved that Speech Dispatcher is installed on this machine but unusable because its real output modules are broken
  - that meant `Read page` still had no working local-native fallback if Speech Dispatcher could not speak
- Constraints:
  - the task needed to stay local and offline-first
  - it had to avoid redesigning the reading flow or introducing a cloud dependency
  - the repo still had unrelated local edits in [src/app/shell.ts](/home/rok/sync/ideas/rpdf2/src/app/shell.ts:1) and [src/styles.css](/home/rok/sync/ideas/rpdf2/src/styles.css:1), so the commit had to stay narrow

# Goals

- Primary success criteria:
  - add at least one local non-Speech-Dispatcher backend path
  - keep the existing `spd-say` path supported rather than replacing it
  - make backend choice visible enough for debugging
- Secondary success criteria:
  - preserve the earlier detailed Speech Dispatcher diagnosis when no direct fallback is available
  - keep verification focused and local

# Approach

- Chosen approach:
  - keep Speech Dispatcher as the preferred backend when healthy
  - add direct CLI fallback support for `espeak-ng`, then `espeak`
  - return the chosen backend to the frontend so the UI can report what actually handled playback
- Why this was the right next action:
  - it fits the app’s existing offline/native design
  - it does not depend on browser speech or cloud APIs
  - it is a small extension of the current Rust command path rather than a new subsystem
- Rejected options:
  - adding a bundled speech engine in this slice would have been much broader
  - treating browser speech as the “secondary local backend” would not satisfy the backend split this task was meant to add

# Implementation

- Task hash:
  - `add-secondary-local-tts-backend`
- Matching task file:
  - [todos/add-secondary-local-tts-backend](/home/rok/sync/ideas/rpdf2/todos/add-secondary-local-tts-backend:1)
- Architecture / flow:
  - local speech backend selection is now explicit:
    1. healthy `spd-say` / Speech Dispatcher
    2. direct `espeak-ng`
    3. direct `espeak`
  - if Speech Dispatcher is broken and neither direct CLI fallback exists, the user now gets one combined actionable error instead of only the dispatcher failure
  - successful native playback now returns backend metadata to the frontend
- Key files or components:
  - [src-tauri/src/infrastructure/local_tts.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/infrastructure/local_tts.rs:1)
    - added explicit backend selection with `Speech Dispatcher`, `eSpeak NG`, and `eSpeak`
    - added direct CLI execution path for `espeak-ng` / `espeak`
    - added a words-per-minute rate mapping for the eSpeak CLI path
    - preserved the earlier Speech Dispatcher diagnosis logic and now appends direct-fallback unavailability when needed
  - [src-tauri/src/contracts/dto.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/contracts/dto.rs:273)
    - added `LocalSpeechBackendDto` so the frontend can see which native backend ran
  - [src-tauri/src/app/services.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/app/services.rs:195)
  - [src-tauri/src/app/commands.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/app/commands.rs:106)
    - changed `speak_text_locally` to return backend metadata instead of bare success
  - [src/features/pdf/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/pdf/workspace.ts:623)
    - native read completion now shows which local backend finished playback

# Results

- Outputs:
  - the app now supports a local non-Speech-Dispatcher native reading path when `espeak-ng` or `espeak` is installed
  - backend choice is visible on successful native playback
  - failure messaging now clearly says when Speech Dispatcher is broken and direct `espeak-ng` / `espeak` fallback is also unavailable
- Verification:
  - `cargo test --manifest-path src-tauri/Cargo.toml local_tts`
  - `cargo check --manifest-path src-tauri/Cargo.toml`
  - `npm run build`

# Decisions

- Fact:
  - this machine still does not have `espeak-ng` or `espeak` installed
- Assessment:
  - the direct fallback path is implemented, but the current environment still needs one of those commands installed before the new fallback can actually be exercised here
- Fact:
  - the frontend had no visibility into which native backend succeeded
- Assessment:
  - returning backend metadata was the smallest clean way to make fallback selection visible without redesigning the playback lifecycle

# Limitations

- I could not live-run the new direct `espeak-ng` / `espeak` path on this machine because neither command is installed here right now.
- Native playback is still a blocking command path, so the frontend only learns the backend name when playback completes.
- `stop_local_speech` is still only meaningful for the Speech Dispatcher path.

# Next steps

1. Install either `espeak-ng` or `espeak` on the target machine and manually verify that `Read page` now uses that direct fallback when Speech Dispatcher remains broken.
2. If native stop/cancel during direct CLI playback becomes important, add a separate task to move native speech onto a managed child-process lifecycle instead of the current blocking call.

# Reproducibility

1. Inspect:
   - [src-tauri/src/infrastructure/local_tts.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/infrastructure/local_tts.rs:1)
   - [src-tauri/src/contracts/dto.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/contracts/dto.rs:273)
   - [src/features/pdf/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/pdf/workspace.ts:623)
2. Run:
   - `cargo test --manifest-path src-tauri/Cargo.toml local_tts`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
   - `npm run build`
