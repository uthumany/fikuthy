#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

BASE_URL="${UTHARNESS_TERMUX_REPO_BASE_URL:-https://uthumany.github.io/utharnessly/termux}"
SCRIPT_URL="${UTHARNESS_TERMUX_INSTALL_REPO_URL:-$BASE_URL/install-repo.sh}"
if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$SCRIPT_URL" | bash
  exit 0
fi
printf '%s\n' 'curl is required. Install it first with: pkg update && pkg install curl' >&2
exit 1
