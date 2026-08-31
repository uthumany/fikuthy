#!/usr/bin/env bash
set -euo pipefail

# Build a Termux-compatible APT repository from prebuilt .deb files.
# Signing is mandatory for publishable output. Set UTHARNESS_GPG_KEY_ID and
# provide the corresponding private key in the caller's isolated GPG home.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INPUT_DIR="${1:-$ROOT/dist/termux}"
OUTPUT_DIR="${2:-$ROOT/dist/termux-repository}"
KEY_ID="${UTHARNESS_GPG_KEY_ID:-}"
VERSION="${UTHARNESS_VERSION:-0.2.13}"

if [[ -z "$KEY_ID" && "${UTHARNESS_ALLOW_UNSIGNED:-0}" != "1" ]]; then
  echo 'UTHARNESS_GPG_KEY_ID is required for a publishable signed repository.' >&2
  echo 'Set UTHARNESS_ALLOW_UNSIGNED=1 only for local metadata tests.' >&2
  exit 2
fi
command -v dpkg-deb >/dev/null || { echo 'dpkg-deb is required.' >&2; exit 2; }
command -v sha256sum >/dev/null || { echo 'sha256sum is required.' >&2; exit 2; }

rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/dists/stable/main" "$OUTPUT_DIR/pool/main" "$OUTPUT_DIR/pool/main/utharness"
shopt -s nullglob
packages=("$INPUT_DIR"/*.deb)
((${#packages[@]} > 0)) || { echo "No .deb packages found in $INPUT_DIR" >&2; exit 3; }
for package in "${packages[@]}"; do
  cp "$package" "$OUTPUT_DIR/pool/main/utharness/"
done

for arch in aarch64 x86_64; do
  binary_dir="$OUTPUT_DIR/dists/stable/main/binary-$arch"
  mkdir -p "$binary_dir"
  packages_file="$binary_dir/Packages"
  : > "$packages_file"
  for package in "$OUTPUT_DIR/pool/main/utharness/"*_"$arch".deb; do
    [[ -f "$package" ]] || continue
    relative="${package#"$OUTPUT_DIR/"}"
    size="$(stat -c '%s' "$package")"
    sha="$(sha256sum "$package" | awk '{print $1}')"
    printf 'Package: %s\n' "$(dpkg-deb -f "$package" Package)" >> "$packages_file"
    printf 'Version: %s\n' "$(dpkg-deb -f "$package" Version)" >> "$packages_file"
    printf 'Architecture: %s\n' "$(dpkg-deb -f "$package" Architecture)" >> "$packages_file"
    printf 'Maintainer: %s\n' "$(dpkg-deb -f "$package" Maintainer)" >> "$packages_file"
    printf 'Installed-Size: %s\n' "$(dpkg-deb -f "$package" Installed-Size)" >> "$packages_file"
    printf 'Depends: %s\n' "$(dpkg-deb -f "$package" Depends)" >> "$packages_file"
    printf 'Filename: %s\n' "$relative" >> "$packages_file"
    printf 'Size: %s\n' "$size" >> "$packages_file"
    printf 'SHA256: %s\n' "$sha" >> "$packages_file"
    printf 'Section: %s\n' "$(dpkg-deb -f "$package" Section)" >> "$packages_file"
    printf 'Priority: %s\n' "$(dpkg-deb -f "$package" Priority)" >> "$packages_file"
    printf 'Description: %s\n' "$(dpkg-deb -f "$package" Description | head -n1)" >> "$packages_file"
    printf '\n' >> "$packages_file"
  done
  gzip -9 -c "$packages_file" > "$packages_file.gz"
done

release="$OUTPUT_DIR/dists/stable/Release"
cat > "$release" <<RELEASE
Origin: UTHARNESS
Label: UTHARNESS Termux
Suite: stable
Codename: stable
Version: $VERSION
Architectures: aarch64 x86_64
Components: main
Description: Signed UTHARNESS packages for Termux
Date: $(date -Ru)
MD5Sum:
RELEASE
while IFS= read -r file; do
  relative="${file#"$OUTPUT_DIR/dists/stable/"}"
  printf ' %s %16s %s\n' "$(md5sum "$file" | awk '{print $1}')" "$(stat -c '%s' "$file")" "$relative" >> "$release"
done < <(find "$OUTPUT_DIR/dists/stable/main" -type f | sort)
printf 'SHA256:\n' >> "$release"
while IFS= read -r file; do
  relative="${file#"$OUTPUT_DIR/dists/stable/"}"
  printf ' %s %16s %s\n' "$(sha256sum "$file" | awk '{print $1}')" "$(stat -c '%s' "$file")" "$relative" >> "$release"
done < <(find "$OUTPUT_DIR/dists/stable/main" -type f | sort)

if [[ -n "$KEY_ID" ]]; then
  command -v gpg >/dev/null || { echo 'gpg is required for signed repository output.' >&2; exit 2; }
  gpg --batch --yes --export "$KEY_ID" > "$OUTPUT_DIR/utharness.gpg"
  gpg --batch --yes --local-user "$KEY_ID" --clearsign --output "$OUTPUT_DIR/dists/stable/InRelease" "$release"
  gpg --batch --yes --local-user "$KEY_ID" --detach-sign --output "$OUTPUT_DIR/dists/stable/Release.gpg" "$release"
else
  : > "$OUTPUT_DIR/UNSIGNED-LOCAL-TEST-ONLY"
fi

find "$OUTPUT_DIR" -type f ! -name SHA256SUMS -printf '%P\n' | sort | while read -r relative; do
  (cd "$OUTPUT_DIR" && sha256sum "$relative")
done > "$OUTPUT_DIR/SHA256SUMS"
mkdir -p "$(dirname "$OUTPUT_DIR")"
tar -C "$(dirname "$OUTPUT_DIR")" -czf "$(dirname "$OUTPUT_DIR")/utharness-termux-repository-${VERSION}.tar.gz" "$(basename "$OUTPUT_DIR")"
sha256sum "$(dirname "$OUTPUT_DIR")/utharness-termux-repository-${VERSION}.tar.gz" > "$(dirname "$OUTPUT_DIR")/utharness-termux-repository-${VERSION}.tar.gz.sha256"
printf '%s\n' "$OUTPUT_DIR"
