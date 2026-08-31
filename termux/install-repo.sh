#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

PREFIX_VALUE="${PREFIX:?Run this script inside Termux so PREFIX is defined}"
REPO_BASE_URL="${UTHARNESS_TERMUX_REPO_BASE_URL:-https://uthumany.github.io/utharnessly/termux}"
KEY_URL="${UTHARNESS_TERMUX_KEY_URL:-$REPO_BASE_URL/utharness.gpg}"
SOURCE_FILE="$PREFIX_VALUE/etc/apt/sources.list.d/utharness.list"
KEY_FILE="$PREFIX_VALUE/etc/apt/trusted.gpg.d/utharness.gpg"

command -v curl >/dev/null 2>&1 || {
  printf '%s\n' 'curl is required. Install it with: pkg install curl' >&2
  exit 1
}
mkdir -p "$(dirname "$SOURCE_FILE")" "$(dirname "$KEY_FILE")"
TEMP_KEY="$(mktemp)"
trap 'rm -f "$TEMP_KEY"' EXIT
curl -fsSL "$KEY_URL" -o "$TEMP_KEY"
test -s "$TEMP_KEY" || { printf '%s\n' 'The repository signing key was empty.' >&2; exit 1; }
install -m 0644 "$TEMP_KEY" "$KEY_FILE"
printf 'deb [signed-by=%s] %s stable main\n' "$KEY_FILE" "$REPO_BASE_URL" > "$SOURCE_FILE"
printf '%s\n' \
  "Configured signed UTHARNESS Termux repository: $REPO_BASE_URL" \
  'Next commands:' \
  '  pkg update' \
  '  pkg install utharness'
