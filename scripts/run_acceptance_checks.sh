#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

echo "Running offline automated acceptance checks for rpdf..."
cargo test

cat <<'EOF'

Automated acceptance checks passed.

Next manual checks:
- Read ACCEPTANCE_CHECKS.md
- Run the tablet/manual workflow checks that cannot be covered by cargo test
EOF
