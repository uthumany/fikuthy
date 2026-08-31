#!/usr/bin/env bash
set -euo pipefail

REPOSITORY="${UTHARNESS_REPOSITORY:-uthumany/utharnessly}"
VERSION="${UTHARNESS_VERSION:-latest}"
INSTALL_DIR="${UTHARNESS_INSTALL_DIR:-${HOME}/.local/bin}"
BASE_URL="${UTHARNESS_RELEASE_BASE_URL:-https://github.com/${REPOSITORY}/releases}"

usage() {
  cat <<'EOF'
Install utharnessly from a published GitHub release archive.

Environment:
  UTHARNESS_VERSION       Release tag without or with v; default: latest
  UTHARNESS_INSTALL_DIR   Destination directory; default: ~/.local/bin
  UTHARNESS_REPOSITORY    GitHub owner/repository; default: uthumany/utharnessly
  UTHARNESS_RELEASE_BASE_URL  Optional release root override for mirrors/tests
EOF
}

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
  "") ;;
  *) echo "unknown argument: $1" >&2; usage >&2; exit 2 ;;
esac

command -v curl >/dev/null || { echo "curl is required" >&2; exit 1; }
command -v tar >/dev/null || { echo "tar is required" >&2; exit 1; }
command -v node >/dev/null || {
  cat >&2 <<'EOF'
Node.js 18 or newer is required for the full-screen terminal UI.
Install the current Node.js LTS release with your platform package manager, then rerun this installer.
EOF
  exit 1
}
node_major="$(node -p 'Number(process.versions.node.split(".")[0])' 2>/dev/null || echo 0)"
[[ "$node_major" -ge 18 ]] || { echo "Node.js 18 or newer is required; found $(node --version 2>/dev/null || echo unknown)" >&2; exit 1; }
if command -v sha256sum >/dev/null; then
  checksum() { sha256sum "$1" | awk '{print $1}'; }
elif command -v shasum >/dev/null; then
  checksum() { shasum -a 256 "$1" | awk '{print $1}'; }
else
  echo "sha256sum or shasum is required" >&2
  exit 1
fi

os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"
case "$os" in
  linux) platform="linux" ;;
  darwin) platform="macos" ;;
  msys*|mingw*|cygwin*) echo "Run the PowerShell installer on Windows; this script is for POSIX shells." >&2; exit 1 ;;
  *) echo "No published archive target for OS '$os'. Use a supported release archive or build from source." >&2; exit 1 ;;
esac
case "$arch" in
  x86_64|amd64) arch="x64" ;;
  arm64|aarch64) arch="arm64" ;;
  *) echo "No published archive target for architecture '$arch'." >&2; exit 1 ;;
esac

if [[ "$VERSION" == "latest" ]]; then
  release_url="${BASE_URL}/latest/download"
  version_label="latest"
else
  VERSION="${VERSION#v}"
  release_url="${BASE_URL}/download/v${VERSION}"
  version_label="v${VERSION}"
fi
asset="utharnessly-${platform}-${arch}.tar.gz"
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
archive="${tmpdir}/${asset}"
checksums="${tmpdir}/SHA256SUMS"

printf 'Downloading utharnessly %s (%s/%s)…\n' "$version_label" "$platform" "$arch"
if ! curl --fail --location --silent --show-error "${release_url}/${asset}" --output "$archive"; then
  cat >&2 <<EOF
No matching release archive was found.
The public repository may not have published this target yet. Build from source:
  git clone https://github.com/${REPOSITORY}.git
  cd utharnessly
  cargo build --release
  pnpm --dir ui install && pnpm --dir ui build
EOF
  exit 1
fi

curl --fail --location --silent --show-error "${release_url}/SHA256SUMS" --output "$checksums" || {
  echo "checksum manifest download failed; refusing to install an unverified archive" >&2
  exit 1
}
expected="$(awk -v name="$asset" '$2 == name || $2 == "*" name { print $1; exit }' "$checksums")"
actual="$(checksum "$archive")"
[[ -n "$expected" && "$expected" == "$actual" ]] || { echo "checksum verification failed" >&2; exit 1; }

mkdir -p "$INSTALL_DIR"
tar -xzf "$archive" -C "$tmpdir"
package_dir="$(find "$tmpdir" -mindepth 1 -maxdepth 1 -type d -name 'utharnessly-*' -print -quit)"
[[ -n "$package_dir" ]] || { echo "release archive has no utharnessly directory" >&2; exit 1; }
[[ -x "${package_dir}/utharness" ]] || { echo "release archive has no executable utharness binary" >&2; exit 1; }
[[ -s "${package_dir}/ui/dist/index.js" ]] || { echo "release archive has no built terminal UI bundle" >&2; exit 1; }
rm -rf "${INSTALL_DIR}/utharnessly-ui"
install -m 0755 "${package_dir}/utharness" "${INSTALL_DIR}/utharness"
ln -sfn utharness "${INSTALL_DIR}/utharnessly"
mkdir -p "${INSTALL_DIR}/utharnessly-ui"
cp -R "${package_dir}/ui/." "${INSTALL_DIR}/utharnessly-ui/"
installed_version="$(${INSTALL_DIR}/utharness --version)"
[[ "$installed_version" == utharness\ * ]] || { echo "installed binary failed its version health check" >&2; exit 1; }

cat <<EOF
Installed utharnessly to ${INSTALL_DIR}/utharness.
The UI bundle is in ${INSTALL_DIR}/utharnessly-ui.
Verified ${installed_version} with Node $(node --version).
Ensure ${INSTALL_DIR} is on PATH, then run: utharnessly
EOF
