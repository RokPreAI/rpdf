# Title

Add canvas text font size controls

# Context

- Problem:
  - Canvas text had a stored `fontSize`, but the user had no direct way to control it as first-class UI.
  - New text size was only derived indirectly from stroke width, and selected text could not have its glyph size edited afterwards.
- Constraints:
  - this slice needed to stay about actual font size only
  - text container resize and wrapping are handled by a separate task
  - the repo still had unrelated local edits in [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md:1), [src/app/shell.ts](/home/rok/sync/ideas/rpdf2/src/app/shell.ts:1), [src/styles.css](/home/rok/sync/ideas/rpdf2/src/styles.css:1), and an untracked sync-conflict copy of `TODO.md`, so the commit had to stay narrow

# Goals

- Primary success criteria:
  - add an explicit text-size slider
  - use it for new text insertion
  - use it for selected text item font-size editing later
- Secondary success criteria:
  - keep the font-size value as real document state
  - keep text edits integrated with the new action-history system

# Approach

- Chosen approach:
  - extend the existing top-left canvas control card with a `Text size` slider
  - keep a dedicated default text-size preference separate from stroke width
  - make the same control selection-aware for text items and live-update the inline text editor when active
- Why this was the right next action:
  - the current text model already persisted `fontSize`, so the missing piece was control wiring, not a document-model redesign
  - the recent snapshot-history work means font-size edits can now be tracked as real undoable actions
- Rejected option:
  - continuing to derive text size from stroke width would have kept the feature implicit and would not satisfy the user requirement of CSS-like explicit font-size control

# Implementation

- Task hash:
  - `add-text-font-size-controls`
- Matching task file:
  - [todos/add-text-font-size-controls](/home/rok/sync/ideas/rpdf2/todos/add-text-font-size-controls:1)
- Architecture / flow:
  - added a dedicated `Text size` slider in the existing canvas control overlay
  - added `defaultTextFontSize` to the local canvas preferences model
  - selected text items now report a current shared font size to the control when possible
  - changing the slider updates:
    - the default size for newly inserted text
    - the selected text item font size when text is selected
    - the live inline text editor font size when the editor is open
- Key files or components:
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:247)
    - added the `Text size` slider markup to the canvas controls
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:416)
    - added `defaultTextFontSize` to local preference read/write handling
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:581)
    - added selection-aware text font-size helpers
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:672)
    - made style-control sync handle text size
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:3275)
    - new text creation now uses the explicit text-size value instead of stroke-width-derived sizing
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:3346)
    - existing text edits now preserve and update the chosen font size

# Results

- Outputs:
  - new text now uses the chosen text-size slider value
  - selecting text lets the same slider change the real glyph size later
  - text-size changes are part of canvas history through `set-selection-text-font-size` and text-edit commits
  - saved and reloaded documents continue to preserve the chosen text sizes
- Verification:
  - `npm run build`
  - `cargo check --manifest-path src-tauri/Cargo.toml`

# Decisions

- Fact:
  - text font size is conceptually different from stroke width
- Assessment:
  - storing and controlling it separately gives the user the explicit CSS-like behavior they asked for
- Fact:
  - the inline text editor is the real insertion surface
- Assessment:
  - the control needed to live-update the editor as well, otherwise “set size while inserting” would only partially work

# Limitations

- I did not add text container width or wrapping behavior in this slice.
- I did not manually runtime-test every combination of:
  - new text insertion
  - selected-text resizing via slider
  - re-editing an existing text item with the editor open
- Mixed multi-selection containing both text and non-text items still favors the default text-size state instead of presenting a richer mixed-state UI.

# Next steps

1. Manually verify:
   - pick the text tool
   - change `Text size`
   - place new text
   - select that text later and change the slider again
   - confirm undo/redo still works for the font-size edit
2. Implement [todos/add-text-container-resize-and-wrap](/home/rok/sync/ideas/rpdf2/todos/add-text-container-resize-and-wrap:1) next so text width/layout can be controlled separately from glyph size.

# Reproducibility

1. Inspect:
   - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:247)
   - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:672)
   - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:3275)
2. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
