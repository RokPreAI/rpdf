# Title

Recent PDF quick-open list report

# Context

The selected worker slice was `add-recent-pdf-quick-open-list` from `TODO.md`.

This was the right next action because:

- it is a bounded PDF-side workflow improvement with a clear user request behind it
- it avoids mixing the next worker commit with the already-dirty canvas file
- it directly reduces the reopen friction when PDF state is lost or a document needs to be reopened quickly

# Goals

- Primary success criteria:
  - keep the 5 most recent PDF paths available inside the first PDF sidebar card
  - allow one-click reopening from that list
  - persist the list across mode switches and restarts

- Secondary success criteria:
  - keep the change local to the PDF workspace UI and local storage
  - avoid widening into a full recent-project/session system

# Approach

- Chosen approach:
  - store recent PDF paths in browser local storage under a dedicated key
  - update the list only after successful PDF open/import flows
  - render the list as a compact set of quick-open buttons below the existing path/open controls

- Rejected options:
  - adding recent-PDF state to the saved PDF session format would have widened the feature into document metadata design
  - storing the list in the app shell would have created extra cross-workspace coupling that this slice did not need
  - rendering the buttons from string-built HTML with embedded path data was rejected in favor of real DOM nodes for path safety

# Implementation

- Architecture / flow:
  - The PDF workspace now reads and writes a dedicated recent-path list in local storage.
  - Successful `Open PDF` actions push the document path to the front of the list, deduplicate it, and clamp the list to 5 items.
  - Imported/restored PDF sessions also refresh the recent list so reopened study sessions stay discoverable.
  - The first PDF sidebar card now shows a `Recent PDFs` section with quick-open buttons that repopulate the path input and reopen the chosen file directly.

- Key files or components:
  - `src/features/pdf/workspace.ts`
    - added recent PDF local-storage helpers
    - added recent-path rendering and quick-open button handling
    - updated successful open/import flows to refresh the recent list
  - `src/styles.css`
    - added compact styling for the recent PDF list and path buttons

# Results

- Outputs:
  - PDF Mode now shows up to 5 recent PDF paths in the first sidebar card.
  - Clicking a recent item immediately fills the path field and reopens that PDF.
  - The list persists locally across workspace remounts and app restarts.

- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - Recent-path state is stored as local UI history, not as part of the study-session document.
  - Assessment:
  - This keeps the feature lightweight and directly aligned with the user’s reopen-speed goal.

- Fact:
  - The quick-open list is rendered with real DOM nodes instead of interpolated HTML attributes.
  - Assessment:
  - This avoids path-escaping edge cases and keeps odd local file paths intact.

# Limitations

- Manual live-window verification of the recent-list interaction was not performed in this turn.
- This slice does not add remove/pin controls for recent entries.
- The backlog file `TODO.md` was not included in this commit because it already contains unrelated uncommitted edits.

# Next steps

1. Implement `add-multi-select` as the next major canvas interaction foundation.
2. Implement `add-marquee-selection` after multi-select is in place.
3. Revisit `fix-pdf-mode-annotations` only with live manual verification, since the code-side fix exists but the backlog still reflects user uncertainty.

# Reproducibility

1. Inspect:
   - `src/features/pdf/workspace.ts`
   - `src/styles.css`
2. Run:
   - `npm run build`
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch:
   - `npm run tauri dev`
4. In PDF Mode, open several PDFs by path, switch away and back if needed, and confirm the first sidebar card shows the 5 most recent PDFs as quick-open buttons.
