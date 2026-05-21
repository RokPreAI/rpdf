# rpdf

`rpdf` is a Tauri desktop app for studying PDFs and taking spatial notes.

It currently has two working modes:
- **Canvas Mode** for infinite-canvas note taking, sketching, selection, editing, and export
- **PDF Mode** for opening PDFs, viewing pages, extracting text, local reading/TTS, recoloring, and importing PDF pages into the canvas

The frontend is written in **TypeScript** and the desktop/backend/file-system side is handled in **Rust**.

## Current repository state

The old README TODO list is no longer accurate. The repository already includes the following implemented areas.

### App shell
- separate **Canvas** and **PDF** modes
- mode state preservation while switching
- shared save/load controls
- autosave and recovery controls
- configurable app bootstrap from a generated user config file

### Canvas Mode
- infinite canvas pan/zoom workflow
- pen drawing with Linux native pressure bridge support
- pressure sensitivity toggle
- configurable input quality / sampling behavior
- text tool with re-edit support
- text sizing, box resize, and wrap support
- rectangle, ellipse, line, and arrow tools
- color picker and keyboard color shortcuts
- selection, marquee selection, multi-select, and select-all
- move, resize, multi-resize, and delete selection
- Ctrl-held grid snapping while moving
- edge alignment guides while moving
- image paste/import
- imported PDF page placements on canvas
- image recolor controls for selected images
- SVG export with eligibility checks and guarded save flow

### PDF Mode
- PDF workspace shell with page navigation
- backend status / trust-state display
- recent PDF quick-open list
- page rendering through the Rust PDF backend boundary
- page-scoped annotation overlay
- native text extraction path
- explicit OCR fallback path
- local reading/TTS pipeline with stop control
- PDF recolor controls
- import current PDF page into Canvas Mode with recolor preserved

### Storage and recovery
- save/load canvas project files
- save/load PDF study session files
- autosave and recovery state
- optimized project save file size
- versioned document/session model

### UX and configuration
- editable app config file
- configurable tool shortcuts and color shortcuts
- toolbar icons and shortcut hints
- responsive layout hardening

## Tech stack
- **Frontend:** TypeScript + Vite
- **Desktop shell / backend:** Tauri 2 + Rust
- **Clipboard integration:** `@tauri-apps/plugin-clipboard-manager`
- **PDF backend path:** Rust service boundary with optional `pdfium-render` feature

## Development

### Install
- Node.js
- Rust toolchain
- Tauri prerequisites for your platform

Then install frontend dependencies:

```bash
npm install
```

### Run the frontend build

```bash
npm run build
```

### Run the app with Tauri

```bash
npm run tauri dev
```

## Configuration

The backend bootstraps an app config file automatically if it does not exist.

On Linux, the config is resolved under one of these locations:
- `$XDG_CONFIG_HOME/rpdf/config.json`
- `~/.config/rpdf/config.json`

The config is used for things like:
- theme colors
- tool shortcuts
- color shortcuts
- canvas background behavior

## Notes about PDF support

The Rust side exposes the PDF operations through a backend/service boundary.

The repo includes an optional `pdfium` Cargo feature:

```toml
[features]
default = []
pdfium = ["dep:pdfium-render"]
```

That means the exact runtime behavior of PDF rendering/extraction depends on how the app is built and what is available on the machine.

## Verification commands commonly used in this repo

```bash
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

## Project structure

High-level structure:

```text
src/
  app/
  features/
    canvas/
    pdf/

src-tauri/
  src/
    app/
    contracts/
    domain/
    infrastructure/
```

## Sources / inspiration
- [pdfViewer](https://www.nutrient.io/blog/how-to-build-a-tauri-pdf-viewer-with-pspdfkit/)
- [velin](https://github.com/mpannu03/velin)
- [twain](https://www.dynamsoft.com/codepool/tauri-document-scanning-desktop-app.html)
