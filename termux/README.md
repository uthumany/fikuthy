# UTHARNESS for Termux

The intended user experience is:

```bash
pkg update
pkg install curl
curl -fsSL https://uthumany.github.io/utharnessly/termux/install.sh | bash
pkg update
pkg install utharness
utharness setup
utharness
```

The first command configures the live signed UTHARNESS APT repository. The package is installed and upgraded by Termux’s package manager; UTHARNESS never overwrites files owned by `pkg`. Repository signatures, package indexes, packages, and checksums are verified during release. Physical-device, Android-version-specific, and real Termux:API behavior still require testing on an actual Android/Termux environment.

```bash
pkg update
pkg upgrade utharness
pkg uninstall utharness
```

User state is kept outside the package under:

```text
$HOME/.config/utharness
$HOME/.local/share/utharness
$HOME/.cache/utharness
```

Package files are limited to the Termux prefix:

```text
$PREFIX/bin/utharness
$PREFIX/lib/utharness
$PREFIX/share/utharness
```

Optional capabilities can be installed separately:

```bash
pkg install termux-api git openssh python
utharness termux api
utharness termux doctor
```

The repository is signed by a release key. The GitHub Actions workflow requires the maintainer’s protected signing secret before it publishes APT metadata; unsigned local metadata is allowed only for package-builder tests and is marked explicitly as local-test output.

## Local package build

The standalone builder expects a release-built native binary and UI bundle:

```bash
pnpm --dir ui install --frozen-lockfile
pnpm --dir ui build
cargo build --release
TERMUX_ARCH=aarch64 packages/utharness/build.sh
```

The generated `.deb` uses the Android/Termux prefix layout and does not require root on the target device. The host must produce an Android-compatible binary for the target architecture; a normal Linux binary is not a valid Termux package payload.
