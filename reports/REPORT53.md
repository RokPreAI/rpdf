# Title

ElevenLabs reading-backend evaluation report

# Context

- Problem:
  - The current `Read page` feature is local-first and now has clearer runtime failure reporting, but the user asked whether ElevenLabs voice generation could be a better speech backend.
  - The key unresolved question was not only voice quality. It was whether ElevenLabs can provide timing data strong enough for future follow-along highlighting without breaking the app’s trust model.
- Constraints:
  - This was explicitly an evaluation task, not provider implementation.
  - The repo still has unrelated local changes in `TODO.md`, `.gitignore`, `src/app/shell.ts`, `src/styles.css`, generated `dist/` files, and a stray root `REPORT1.md`, so this worker slice needed to stay documentation-only and avoid mixing those unrelated edits into the commit.

# Goals

- Primary success criteria:
  - produce a clear yes/no recommendation on whether ElevenLabs is suitable for this app’s reading mode
  - explicitly address timing data for future follow-along highlighting
  - document privacy, latency, reliability, and cost tradeoffs in app terms
- Secondary success criteria:
  - define the smallest viable future integration path if the provider is kept as an option

# Approach

- Chosen approach:
  - inspect the current local reading pipeline in the repo
  - verify current ElevenLabs capabilities and constraints against official docs
  - write a durable repo-local evaluation artifact with a concrete recommendation and next-step integration shape
- Why this was the right next action:
  - it is the only remaining open backlog item that was both real and bounded
  - it avoids inventing new code work when the task itself is a design/evaluation task
- Rejected option:
  - implementing a provider speculatively without first resolving timing, privacy, and trust implications would have been the wrong sequence

# Implementation

- Task hash:
  - `evaluate-elevenlabs-tts-fallback-and-timing`
- Matching task file:
  - [todos/evaluate-elevenlabs-tts-fallback-and-timing](/home/rok/sync/ideas/rpdf2/todos/evaluate-elevenlabs-tts-fallback-and-timing:1)
- Architecture / flow:
  - I inspected the current PDF reading path and confirmed that the app currently has local/native and browser fallback speech, extraction trust messaging, and reading status updates, but not a real time-synced word-highlighting implementation yet.
  - I then checked the current ElevenLabs docs for:
    - text-to-speech with timestamps
    - streaming text-to-speech with timestamps
    - forced alignment
    - pricing and retention constraints
  - I captured the recommendation and integration path in a dedicated evaluation note.
- Key files or components:
  - [src/features/pdf/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/pdf/workspace.ts:623)
    - confirmed the current local/browser speech behavior and trust messaging
  - [src-tauri/src/infrastructure/local_tts.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/infrastructure/local_tts.rs:1)
    - confirmed the current native path is local `spd-say`, not cloud-backed
  - [research/elevenlabs-tts-evaluation.md](/home/rok/sync/ideas/rpdf2/research/elevenlabs-tts-evaluation.md:1)
    - added the actual evaluation artifact with the final recommendation and future integration path

# Results

- Outputs:
  - Recommendation:
    - ElevenLabs is not suitable as the default reading backend for this app.
    - ElevenLabs is viable only as an explicit optional cloud backend.
  - Timing conclusion:
    - ElevenLabs does provide timing data that is relevant enough for future follow-along work.
    - The best first path is `text-to-speech with timestamps`, which returns character-level alignment that can be converted into word spans locally.
    - Forced Alignment is useful, but not as the primary first integration path.
  - Product/trust conclusion:
    - Because the app is currently local-first and page-scoped, a cloud backend must stay opt-in, visibly labeled, and disabled by default.
- Verification:
  - Verified the local app reading path by inspecting:
    - [src/features/pdf/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/pdf/workspace.ts:623)
    - [src-tauri/src/infrastructure/local_tts.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/infrastructure/local_tts.rs:1)
  - Verified current ElevenLabs capabilities against official docs:
    - `Create speech with timing`
    - `Stream speech with timing`
    - `Create Forced Alignment`
    - `Forced Alignment` overview
    - `API Pricing`

# Decisions

- Fact:
  - ElevenLabs exposes a `with-timestamps` TTS endpoint that returns audio plus character timing metadata.
- Assessment:
  - That is good enough to justify optional-provider exploration because word spans can be derived locally from the character timings.
- Fact:
  - The current app does not yet have a finished highlight engine, only trust messaging around weak follow-along reliability.
- Assessment:
  - This lowers migration pressure. The task is to choose the right provider contract first, not preserve an already-existing fine-grained synchronizer.
- Fact:
  - ElevenLabs cloud use changes the privacy and reliability model, and zero-retention mode is not a normal default path for all users.
- Assessment:
  - That makes ElevenLabs incompatible with the app’s safest default behavior.

# Limitations

- This slice does not implement the ElevenLabs provider.
- I did not add config-file fields, API-key storage, playback code, or highlight mapping code in this turn.
- `TODO.md` was not updated in this commit because it still contains unrelated local backlog edits.

# Next steps

1. If you want to build this, add a new task for an explicit optional `elevenlabs` reading backend.
   - Scope it around provider config, API-key handling, `with-timestamps` playback, and visible cloud-trust messaging.
2. If follow-along highlighting becomes a concrete feature, add a separate task for character-to-word timing mapping and validation against real extracted PDF pages.
3. If you want to stay local-first, stop here and keep ElevenLabs as a documented non-default option rather than productizing it now.

# Reproducibility

1. Inspect the current local reading path:
   - [src/features/pdf/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/pdf/workspace.ts:623)
   - [src-tauri/src/infrastructure/local_tts.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/infrastructure/local_tts.rs:1)
2. Read the evaluation artifact:
   - [research/elevenlabs-tts-evaluation.md](/home/rok/sync/ideas/rpdf2/research/elevenlabs-tts-evaluation.md:1)
3. Cross-check the provider docs linked in that evaluation note.
