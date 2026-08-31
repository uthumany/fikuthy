#!/data/data/com.termux/files/usr/bin/bash
set -euo pipefail

if command -v utharness >/dev/null 2>&1; then
  exec utharness termux doctor
fi
printf '%s\n' 'UTHARNESS TERMUX PRE-INSTALL DOCTOR'
printf 'PREFIX: %s\n' "${PREFIX:-missing}"
printf 'HOME:   %s\n' "${HOME:-missing}"
printf 'ARCH:   %s\n' "$(uname -m 2>/dev/null || printf unknown)"
printf 'TERM:   %s\n' "${TERM:-unknown}"
for command in curl tar sha256sum node python git ssh openssl; do
  if command -v "$command" >/dev/null 2>&1; then
    printf '✓ %-8s available\n' "$command"
  else
    printf '! %-8s missing\n' "$command"
  fi
done
printf '%s\n' 'Install the missing prerequisites with pkg, then run: utharness setup'
