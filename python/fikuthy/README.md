# fikuthy

`fikuthy` is the Python launcher for the [Fikuthy](https://github.com/fikuthy/fikuthy) local-first agent terminal. It downloads the matching GitHub Release archive on first use, verifies the published SHA-256 checksum, caches the native Rust runtime and bundled Ink UI, and forwards CLI arguments to `fikuthy`.

## Usage

```bash
python -m pip install fikuthy
fikuthy --help
fikuthy --version
fikuthy
```

The release launcher currently publishes Linux x64, macOS x64/arm64, and Windows x64 artifacts. Other architectures and operating systems should use the documented source-build or remote-host workflow in the repository installation guide.

```bash
fikuthy update
fikuthy uninstall
```

`uninstall` prints the package and cache removal commands; it does not silently mutate the Python environment.

## License

MIT. See the repository for the full license and development instructions.
