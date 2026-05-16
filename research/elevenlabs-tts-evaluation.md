# ElevenLabs TTS Evaluation

## Question

Is ElevenLabs a good fit for `rpdf2` as a reading backend for `Read page`, especially if the app wants future follow-along highlighting?

## Current local app state

- The current PDF reading path is local-first.
- Native speech uses `spd-say` through Rust.
- Browser fallback uses `speechSynthesis`.
- The app currently surfaces extraction trust and reading status, but it does not yet implement a real time-synced word-highlighting pipeline.
- Product language in the app already emphasizes trust:
  - local and page-scoped reading
  - weak extraction should not be treated as reliable follow-along

That matters because any cloud backend must preserve this trust model instead of pretending precision it cannot guarantee.

## Relevant ElevenLabs capabilities

### 1. Text to speech with timing

ElevenLabs has a TTS endpoint that returns generated audio together with per-character timing data:

- `POST /v1/text-to-speech/:voice_id/with-timestamps`
- It returns `audio_base64`
- It also returns `alignment` and `normalized_alignment`
- Those include:
  - `characters`
  - `character_start_times_seconds`
  - `character_end_times_seconds`

This is the most relevant endpoint for `rpdf2` because it gives timing data from the same request that generates the audio.

### 2. Streaming text to speech with timing

ElevenLabs also has a streaming variant:

- `POST /v1/text-to-speech/:voice_id/stream/with-timestamps`

That returns a stream of JSON payloads containing audio data plus timing information. This is more complex, but it may reduce perceived latency if the app later wants progressive playback.

### 3. Forced Alignment

ElevenLabs also exposes Forced Alignment:

- `POST /v1/forced-alignment`

That endpoint aligns an existing audio file to supplied text and returns:

- character timings
- word timings
- overall alignment loss

This is useful if audio and text need to be re-aligned after a separate generation step, but it is not the best first integration path for this app because `with-timestamps` already gives timing alongside generation.

## Timing suitability for highlighting

### What is good

- ElevenLabs timing support is real, not hypothetical.
- The `with-timestamps` endpoint gives character-level timing directly from TTS generation.
- Character timings are enough to build word highlighting locally by tokenizing the original text and mapping each word span to the earliest start and latest end character timestamp in that span.

### What is still risky

- The timing is character-level in the TTS endpoint, not explicit word-level spans.
- Word highlighting would have to be derived in app code.
- PDF extraction can be weak, sparse, or OCR-derived. Even perfect TTS timings do not fix bad source text.
- If the app changes text before sending it to TTS, highlight mapping can drift unless normalization is tightly controlled.

## Recommendation

### Default backend decision

No: ElevenLabs should **not** replace the current local-first default reading backend.

Reasons:

- It breaks the current privacy and offline trust model.
- It introduces external API dependency, latency, and cost for a feature that already has a local path.
- The current app language is explicitly careful about reliability. A cloud voice provider with remote text transfer should not silently become the default behavior.

### Optional backend decision

Yes, but only as an **explicit optional cloud backend**.

That means:

- disabled by default
- clearly labeled as cloud TTS
- requires the user to provide an API key
- clearly warns that page text will be sent to ElevenLabs
- only enabled for users who want higher voice quality and accept the tradeoffs

## Privacy and trust implications

If ElevenLabs is used, the extracted page text leaves the device and is sent to a third-party service.

Important consequences:

- not offline
- not suitable as the safest default for sensitive PDFs
- adds key management burden
- changes the app from purely local reading into cloud-assisted reading

The docs also note that zero-retention mode is tied to `enable_logging=false`, and that mode may only be used by enterprise customers. That means many normal users should assume request data is not operating under a simple default zero-retention promise.

## Reliability implications

### Better than local speech in one way

- voice quality is likely better
- timing data is stronger than browser `speechSynthesis`

### Worse than local speech in other ways

- requires network availability
- can fail due to auth, quota, billing, or provider outage
- introduces latency before playback
- may be inappropriate for private or regulated documents

## Cost implications

ElevenLabs TTS is billed per character.

At the time of this evaluation, the public API pricing page lists:

- Flash / Turbo TTS: `$0.05` per `1K` characters
- Multilingual v2 / v3 TTS: `$0.10` per `1K` characters

That makes occasional page reading feasible, but it is still materially different from the current local path, which is effectively no per-page cloud cost.

Forced Alignment pricing is documented as the same rate as the Speech to Text API, so adding a second alignment pass would add further cost and complexity.

## Best minimal integration path

If this is implemented, the first version should be:

1. Keep the current local backend as default.
2. Add a new optional provider mode:
   - `local`
   - `browser`
   - `elevenlabs`
3. For `elevenlabs`, call `POST /v1/text-to-speech/:voice_id/with-timestamps`.
4. Decode the returned audio and play it in the frontend.
5. Convert returned character timings into local word spans for follow-along experiments.
6. Show explicit trust copy:
   - this page text is sent to ElevenLabs
   - highlighting is derived from provider timings
   - weak PDF extraction still weakens trust

## Failure behavior if implemented

The app should fail closed and visibly:

- missing API key:
  - do not try the provider
  - show a clear configuration message
- network/provider failure:
  - show a clear cloud TTS failure
  - offer local fallback when available
- weak extraction:
  - still allow playback if the user explicitly wants it
  - do not claim precise highlighting trust

## What not to do

- Do not make ElevenLabs the default `Read page` backend.
- Do not hide the fact that PDF text is leaving the machine.
- Do not claim precise word highlighting unless the derived timing path is actually validated against real pages.
- Do not start with Forced Alignment as the primary path when the TTS timing endpoint already exists.

## Final answer

ElevenLabs is a **viable optional premium cloud backend**, but **not a good default backend** for `rpdf2`.

If pursued, the first implementation should use the TTS `with-timestamps` endpoint and derive word spans locally from returned character timings. Forced Alignment should remain a secondary tool, not the initial integration path.

## Sources

- ElevenLabs TTS with timestamps:
  - https://elevenlabs.io/docs/api-reference/text-to-speech/convert-with-timestamps
- ElevenLabs streaming TTS with timestamps:
  - https://elevenlabs.io/docs/api-reference/text-to-speech/stream-with-timestamps
- ElevenLabs Forced Alignment API:
  - https://elevenlabs.io/docs/api-reference/forced-alignment/create
- ElevenLabs Forced Alignment overview:
  - https://elevenlabs.io/docs/overview/capabilities/forced-alignment
- ElevenLabs API pricing:
  - https://elevenlabs.io/pricing/api/
