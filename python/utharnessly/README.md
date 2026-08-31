# utharnessly

`utharnessly` is the Python launcher for the [Utharness](https://github.com/uthumany/utharnessly) local-first agent terminal. It downloads the matching GitHub Release archive on first use, verifies the published SHA-256 checksum, caches the native Rust runtime and bundled Ink UI, and forwards CLI arguments to `utharness`.

## Usage

```bash
python -m pip install utharnessly
utharnessly --help
utharnessly --version
utharnessly
```

The release launcher currently publishes Linux x64, macOS x64/arm64, and Windows x64 artifacts. Other architectures and operating systems should use the documented source-build or remote-host workflow in the repository installation guide.

```bash
utharnessly update
utharnessly uninstall
```

`uninstall` prints the package and cache removal commands; it does not silently mutate the Python environment.

## License

MIT. See the repository for the full license and development instructions.
