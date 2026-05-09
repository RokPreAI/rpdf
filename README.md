# About
A simple minimal pdf reader and anotator writen in rust.
# Freates
- Customizable keyboard shortcuts.
- Recoloring for dislectics and night theme.
- Reading help, which word highlighting and word dimming. (The only word that is visible is not grayed out is the one that is beaing read)
- Text to speach. (Normal text and math equations) (Can be paused on images and figures or automatically continue)
- Simple annotations using vector graphics and support for drawing tablets. (text, highlighting and drawing)
- Infinite canvas when not viewing a pdf.

# Shortcuts
- `Tab`: switch between Infinite Canvas Mode and PDF Mode
- `B`, `H`, `V`, `E`: switch to ink, highlighter, selection, or eraser
- `Cmd/Ctrl+S`: save the current canvas document or PDF session
- `Cmd/Ctrl+L`: load the current canvas document or PDF session
- `Cmd/Ctrl+R`: recover the latest autosave snapshot for the current mode
- Infinite Canvas Mode:
  - `Cmd/Ctrl+V`: paste clipboard image
  - `Cmd/Ctrl+Shift+E`: export SVG
  - hold `Space` and drag: pan
  - double-tap `Space`: fit canvas content to view
- PDF Mode:
  - `Cmd/Ctrl+O`: open PDF
  - `Left` / `Right`: previous or next page
  - `T`: start or stop TTS
