#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  ./scripts/run_study_session_checklist.sh <readable-pdf> <weak-pdf> <image> [session-record-path]

Purpose:
  Prepare and record a manual rpdf study-session validation run.

Behavior:
  - verifies the supplied local assets exist
  - reruns the automated acceptance baseline
  - checks whether a graphical display is available
  - writes a timestamped markdown session record with the asset paths and checklist

Example:
  ./scripts/run_study_session_checklist.sh \
    samples/text-readable.pdf \
    samples/scanned-or-weak.pdf \
    samples/reference-image.png
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if [[ $# -lt 3 || $# -gt 4 ]]; then
  usage >&2
  exit 1
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

readable_pdf="$1"
weak_pdf="$2"
reference_image="$3"

for asset in "$readable_pdf" "$weak_pdf" "$reference_image"; do
  if [[ ! -f "$asset" ]]; then
    echo "Missing asset: $asset" >&2
    exit 1
  fi
done

timestamp="$(date +"%Y-%m-%dT%H-%M-%S")"
session_dir="$repo_root/study_sessions"
mkdir -p "$session_dir"

session_record_path="${4:-$session_dir/${timestamp}-manual-study-session.md}"

display_summary="none"
launch_hint="GUI not available in this shell. Run this script again from a graphical session before attempting the live checklist."
if [[ -n "${DISPLAY:-}" || -n "${WAYLAND_DISPLAY:-}" ]]; then
  display_summary="available"
  launch_hint="Graphical session detected. After reviewing the generated record, run cargo run and complete the live checklist in the app."
fi

echo "Running automated baseline before the manual study session..."
"$repo_root/scripts/run_acceptance_checks.sh"

cat >"$session_record_path" <<EOF
# rpdf Manual Study Session Record

Date: $(date +"%Y-%m-%d")
Timestamp: $timestamp
Graphical display: $display_summary

## Assets

- Readable PDF: $readable_pdf
- Weak or scanned PDF: $weak_pdf
- Reference image: $reference_image

## Preflight

- [x] Automated baseline rerun with \`./scripts/run_acceptance_checks.sh\`
- [ ] Launch the app with \`cargo run\`
- [ ] Confirm the mode switcher and both modes render locally

## PDF Mode Checklist

- [ ] Open the readable PDF and confirm the document opens cleanly
- [ ] Start TTS and confirm native-text guidance is understandable
- [ ] Confirm highlight behavior matches the selected mode
- [ ] Open the weak or scanned PDF
- [ ] Start TTS and confirm OCR fallback or warning behavior is honest
- [ ] Add ink, highlight, and text-note annotations without losing reading context

## Canvas Mode Checklist

- [ ] Draw multiple pen strokes and judge pressure feel
- [ ] Pan and zoom while keeping the background usable
- [ ] Add typed text
- [ ] Import the reference image
- [ ] Import one PDF page
- [ ] Confirm the grouped toolbar sections reduce navigation friction

## Save And Export Checklist

- [ ] Save and reload a canvas session
- [ ] Save and reload a PDF session
- [ ] Wait for autosave, then verify recovery messaging is understandable
- [ ] Attempt SVG export with an incompatible mixed-content target
- [ ] Attempt SVG export with a compatible vector-only target

## Notes

- Launch hint: $launch_hint
- Observed friction:
- Successful parts:
- Remaining problems:
- Follow-up idea:
EOF

echo
echo "Manual study-session record created:"
echo "  $session_record_path"
echo
echo "$launch_hint"
