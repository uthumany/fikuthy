#!/usr/bin/env bash
set -euo pipefail

# This script supports standalone CI/local package assembly. When sourced by the
# official termux-packages framework, the TERMUX_PKG_* variables below are also
# available for downstream integration.
TERMUX_PKG_HOMEPAGE="https://github.com/uthumany/utharnessly"
TERMUX_PKG_DESCRIPTION="Local-first autonomous AI agent terminal and TUI for Termux"
TERMUX_PKG_LICENSE="MIT"
TERMUX_PKG_MAINTAINER="UTHARNESS Contributors"
TERMUX_PKG_VERSION="${TERMUX_PKG_VERSION:-${UTHARNESS_VERSION:-0.2.13}}"
TERMUX_PKG_SRCURL="https://github.com/uthumany/utharnessly/archive/refs/tags/v${TERMUX_PKG_VERSION}.tar.gz"
TERMUX_PKG_DEPENDS="nodejs-lts, ca-certificates, openssl, termux-tools"
TERMUX_PKG_SUGGESTS="termux-api, git, openssh, python"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${TERMUX_PACKAGE_OUT:-$ROOT/dist/termux}"
WORK_DIR="${TERMUX_PACKAGE_WORK:-$OUT_DIR/work}"
STAGE="$WORK_DIR/stage"
PREFIX_ROOT="${TERMUX_PREFIX:-data/data/com.termux/files/usr}"
RUST_BINARY="${TERMUX_RUST_BINARY:-$ROOT/target/release/utharness}"
UI_DIST="${TERMUX_UI_DIST:-$ROOT/ui/dist}"
ARCH="${TERMUX_ARCH:-}"

if [[ -z "$ARCH" ]]; then
  machine="$(uname -m)"
  case "$machine" in
    aarch64|arm64) ARCH=aarch64 ;;
    x86_64|amd64) ARCH=x86_64 ;;
    armv7*|armv8l) ARCH=arm ;;
    i686|x86) ARCH=i686 ;;
    *) echo "Unsupported Termux architecture: $machine" >&2; exit 2 ;;
  esac
fi

if [[ ! -x "$RUST_BINARY" ]]; then
  echo "Native binary not found or not executable: $RUST_BINARY" >&2
  echo "Build with cargo --release first, or set TERMUX_RUST_BINARY." >&2
  exit 3
fi
if [[ ! -f "$UI_DIST/index.js" ]]; then
  echo "Bundled UI not found: $UI_DIST/index.js" >&2
  echo "Build with pnpm --dir ui build first, or set TERMUX_UI_DIST." >&2
  exit 3
fi

rm -rf "$STAGE"
mkdir -p "$OUT_DIR" "$STAGE/$PREFIX_ROOT/bin" "$STAGE/$PREFIX_ROOT/lib/utharness" "$STAGE/$PREFIX_ROOT/share/utharness" "$STAGE/DEBIAN"
install -Dm755 "$RUST_BINARY" "$STAGE/$PREFIX_ROOT/bin/utharness"
cp -R "$UI_DIST" "$STAGE/$PREFIX_ROOT/lib/utharness/dist"
install -Dm644 "$ROOT/ui/package.json" "$STAGE/$PREFIX_ROOT/lib/utharness/package.json"
install -Dm644 "$ROOT/LICENSE" "$STAGE/$PREFIX_ROOT/share/utharness/LICENSE"
install -Dm644 "$ROOT/README.md" "$STAGE/$PREFIX_ROOT/share/utharness/README.md"
install -Dm644 "$ROOT/termux/environment.sh" "$STAGE/$PREFIX_ROOT/share/utharness/environment.sh"

cat > "$STAGE/DEBIAN/control" <<CONTROL
Package: utharness
Version: $TERMUX_PKG_VERSION
Section: misc
Priority: optional
Architecture: $ARCH
Maintainer: $TERMUX_PKG_MAINTAINER
Depends: $TERMUX_PKG_DEPENDS
Suggests: $TERMUX_PKG_SUGGESTS
Homepage: $TERMUX_PKG_HOMEPAGE
Description: $TERMUX_PKG_DESCRIPTION
 UTHARNESS runs a local-first Rust agent runtime with an Ink terminal UI.
 It stores user state only below the Termux home directory and never requires root.
CONTROL

install -Dm755 "$ROOT/termux/postinst" "$STAGE/DEBIAN/postinst"
install -Dm755 "$ROOT/termux/prerm" "$STAGE/DEBIAN/prerm"

PACKAGE="$OUT_DIR/utharness_${TERMUX_PKG_VERSION}_${ARCH}.deb"
dpkg-deb --root-owner-group --build "$STAGE" "$PACKAGE" >/dev/null
sha256sum "$PACKAGE" > "$PACKAGE.sha256"
printf '%s\n' "$PACKAGE"
