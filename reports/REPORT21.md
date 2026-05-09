# Title

rpdf clipboard image paste support report

# Context

The active task in `TODO.md` was `add-clipboard-image-paste-support`. Infinite Canvas Mode already supported file-path image import, but it still forced the user through a manual path entry workflow even when the desired image was already in the system clipboard.

Two constraints shaped the implementation:

- native `eframe`/`egui` on desktop only forwards text clipboard contents through `egui::Event::Paste(...)`, so image paste could not rely on the built-in paste event alone
- this needed to stay a bounded image-only clipboard slice, not expand into generic clipboard handling or the broader shortcut/discoverability backlog item

# Goals

Primary success criteria:

- allow a user to paste an image from the system clipboard directly into Infinite Canvas Mode
- expose the action through at least one normal UI path and one keyboard path
- report a bounded failure state when the clipboard does not contain a usable image

Secondary success criteria:

- keep the imported clipboard image compatible with the app's existing save/load model
- preserve sensible default placement and sizing without widening the renderer architecture

# Approach

The chosen approach was to add a small clipboard import service backed by `arboard`, then keep the canvas-side behavior simple:

- read clipboard image metadata directly from the native clipboard
- represent pasted images as a serializable `ImportedAssetSource::ClipboardImage { ... }`
- reuse the existing imported-image canvas item path, with aspect-ratio-preserving default bounds
- expose paste through both a visible `Paste clipboard image` button and `Cmd/Ctrl+V` when the canvas is active and a text field is not focused

Rejected option:

- trying to route clipboard images through `egui::Event::Paste(...)`
  - Fact: the local `eframe 0.31` native integration only injects text paste events.
  - Assessment: image paste needed an explicit clipboard read path instead of pretending the text-paste event would carry image data.

# Implementation

Architecture and flow:

- [Cargo.toml](/home/rok/sync/ideas/rpdf/Cargo.toml:1)
  - added a direct dependency on `arboard = "3.6.1"` so the app can read native clipboard image data deliberately instead of depending on transitive internals

- [src/app/services.rs](/home/rok/sync/ideas/rpdf/src/app/services.rs:1)
  - added `ClipboardImportService` to `AppServices`
  - added `ClipboardImageImport { width, height }`
  - added `read_clipboard_image(...)`:
    - opens the system clipboard
    - reads image contents through `arboard`
    - rejects empty, unsupported, or zero-sized clipboard image states with bounded error strings
  - added `canvas_image_size(...)` and a private scaling helper so pasted images get sane default bounds while preserving aspect ratio
  - added focused tests for clipboard-image sizing behavior

- [src/model/mod.rs](/home/rok/sync/ideas/rpdf/src/model/mod.rs:1)
  - extended `ImportedAssetSource` with:
    - `ClipboardImage { width, height, pasted_unix_ms }`
  - this keeps clipboard-imported images serializable and recoverable through the existing document model

- [src/app/canvas.rs](/home/rok/sync/ideas/rpdf/src/app/canvas.rs:1)
  - added a visible `Paste clipboard image` button alongside the file-path image import controls
  - added `handle_canvas_clipboard_shortcuts(...)` to consume `Cmd/Ctrl+V` only when the UI is not currently focused on text entry
  - added `paste_canvas_image_from_clipboard(...)`:
    - requests clipboard image metadata from the new service
    - creates a new canvas image item with aspect-ratio-preserving bounds
    - selects the pasted item immediately
    - reports success or bounded failure through the existing canvas status surface
  - updated imported-image placeholder rendering so clipboard-origin images show as `Clipboard image` with their original dimensions

- [TODO.md](/home/rok/sync/ideas/rpdf/TODO.md:1)
  - marked `add-clipboard-image-paste-support` done
  - promoted `simplify-gui-and-tool-grouping` to the current task

Example behavior:

- if the clipboard contains a `1600 x 800` image, the paste action creates an imported image item sized to a `480 x 240` default canvas box
- if the clipboard contains no image, the canvas status banner reports a bounded clipboard-image failure instead of silently doing nothing

# Results

Outputs:

- Infinite Canvas Mode now supports clipboard image paste through:
  - a visible `Paste clipboard image` button
  - `Cmd/Ctrl+V` when canvas mode is active and text entry is not focused
- pasted clipboard images now appear as regular imported image items with stored source metadata and immediate selection
- a new report was created at [reports/REPORT21.md](/home/rok/sync/ideas/rpdf/reports/REPORT21.md:1)

Metrics and observations:

- the test suite now has `16` passing tests
- two new tests were added for clipboard-image sizing behavior

Verification:

- ran `cargo fmt`
- ran `cargo test`
- ran `./scripts/run_acceptance_checks.sh`
- all passed

# Decisions

Fact: clipboard images are stored as a dedicated model source variant instead of a dumped temporary file.
Assessment: this keeps save/load honest and avoids creating unmanaged filesystem artifacts for a placeholder-rendered image path.

Fact: the first version reads clipboard image metadata but does not add full bitmap rendering.
Assessment: the app's current imported-image rendering is already placeholder-based, so this keeps the feature aligned with the existing visual contract while still enabling the lower-friction workflow the user requested.

Fact: `Cmd/Ctrl+V` is only consumed when `egui` does not want keyboard input.
Assessment: this avoids breaking normal text pasting inside input fields before the broader shortcut system is formalized.

# Limitations

- Clipboard image paste is implemented only for Infinite Canvas Mode, not PDF Mode.
- The current image renderer still uses placeholder cards rather than drawing the actual bitmap contents, so pasted images appear as labeled image items instead of fully rendered pixel previews.
- The shortcut surface is intentionally narrow: only clipboard-image paste was added here, not the broader shortcut map.
- Live clipboard interaction was not manually exercised in this headless environment, so runtime behavior is verified through compile/test coverage and the acceptance script rather than an actual local GUI paste session here.
- The repository still contains unrelated user-side report moves and planning artifacts, which were intentionally left untouched.

# Next steps

1. Implement `simplify-gui-and-tool-grouping`.
   The tool surface has grown again, so the main UI now needs a focused simplification pass.

2. Implement `add-keyboard-shortcuts-and-discoverability`.
   The canvas now has another real shortcut, which increases the value of a coherent visible shortcut system.

# Reproducibility

1. Work in `/home/rok/sync/ideas/rpdf`.
2. Format the repo:

```bash
cargo fmt
```

3. Run the test suite:

```bash
cargo test
```

4. Run the automated acceptance checks:

```bash
./scripts/run_acceptance_checks.sh
```

5. In a graphical local session, verify manually that:

- clicking `Paste clipboard image` inserts a new image item when the system clipboard contains an image
- pressing `Cmd/Ctrl+V` in canvas mode does the same when no text field is focused
- pressing `Cmd/Ctrl+V` while typing in a text field does not hijack normal text entry
- trying to paste with no image in the clipboard shows a bounded failure message
