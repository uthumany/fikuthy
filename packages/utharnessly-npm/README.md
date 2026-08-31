# utharnessly

`utharnessly` is the npm launcher for the [Utharness](https://github.com/uthumany/utharnessly) local-first agent terminal. It downloads the matching GitHub Release archive on first use, verifies the published SHA-256 checksum, caches the native Rust runtime and bundled Ink UI, and forwards CLI arguments to `utharness`.

## Usage

```bash
npx utharnessly --help
npx utharnessly --version
npx utharnessly
npm install --global utharnessly
utharness init
```

The release launcher currently publishes Linux x64, macOS x64/arm64, and Windows x64 artifacts. Other architectures and operating systems should use the documented source-build or remote-host workflow in the repository installation guide.

```bash
utharnessly update
utharnessly uninstall
```

`uninstall` prints the package and cache removal commands; it does not silently mutate the global npm installation.

## License

MIT. See the repository for the full license and development instructions.
