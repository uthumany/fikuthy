"""Thin Python launcher for the utharnessly native release."""

from __future__ import annotations

import hashlib
import os
import platform
import shutil
import subprocess
import sys
import tarfile
import tempfile
import urllib.request
import zipfile
from pathlib import Path

VERSION = "0.2.13"
REPOSITORY = "uthumany/utharnessly"


def release_base_url() -> str:
    return os.environ.get(
        "UTHARNESSLY_RELEASE_BASE_URL",
        f"https://github.com/{REPOSITORY}/releases/download/v{VERSION}",
    ).rstrip("/")


def platform_asset() -> tuple[str, str]:
    system = sys.platform
    machine = platform.machine().lower()
    if system.startswith("linux") and machine in {"x86_64", "amd64"}:
        return "utharnessly-linux-x64.tar.gz", "tar.gz"
    if system == "darwin" and machine in {"x86_64", "amd64"}:
        return "utharnessly-macos-x64.tar.gz", "tar.gz"
    if system == "darwin" and machine in {"arm64", "aarch64"}:
        return "utharnessly-macos-arm64.tar.gz", "tar.gz"
    if system.startswith("win") and machine in {"x86_64", "amd64"}:
        return "utharnessly-windows-x64.zip", "zip"
    raise RuntimeError(
        f"No published utharnessly binary for {system}/{machine}. "
        "Supported release targets are Linux x64, macOS x64/arm64, and Windows x64; "
        f"use the source instructions at https://github.com/{REPOSITORY} on other targets."
    )


def cache_root() -> Path:
    if os.name == "nt":
        base = Path(os.environ.get("LOCALAPPDATA", Path.home()))
    else:
        base = Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache"))
    return base / "utharnessly" / VERSION


def _download(url: str, destination: Path) -> None:
    request = urllib.request.Request(url, headers={"User-Agent": "utharnessly-python-launcher"})
    with urllib.request.urlopen(request, timeout=60) as response, destination.open("wb") as output:
        shutil.copyfileobj(response, output)


def _verify_checksum(archive: Path, checksums: Path) -> None:
    asset = archive.name
    line = next(
        (
            row
            for row in checksums.read_text(encoding="utf-8").splitlines()
            if len(row.split()) >= 2 and row.split()[-1].lstrip("*") == asset
        ),
        None,
    )
    if not line:
        raise RuntimeError(f"SHA256SUMS does not contain {asset}")
    expected = line.split()[0].lower()
    actual = hashlib.sha256(archive.read_bytes()).hexdigest()
    if expected != actual:
        raise RuntimeError(f"checksum verification failed for {asset}")


def _extract(archive: Path, format_name: str, destination: Path) -> None:
    if format_name == "tar.gz":
        with tarfile.open(archive, "r:gz") as handle:
            handle.extractall(destination, filter="data")
    elif format_name == "zip":
        with zipfile.ZipFile(archive) as handle:
            handle.extractall(destination)
    else:
        raise RuntimeError(f"unsupported archive format: {format_name}")


def ensure_binary(force: bool = False) -> tuple[Path, Path]:
    asset, format_name = platform_asset()
    root = cache_root()
    binary = root / ("utharness.exe" if os.name == "nt" else "utharness")
    ui = root / "ui"
    if not force and binary.is_file():
        return binary, ui
    shutil.rmtree(root, ignore_errors=True)
    root.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="utharnessly-") as temporary:
        temporary_path = Path(temporary)
        archive = temporary_path / asset
        checksums = temporary_path / "SHA256SUMS"
        print(f"Downloading utharnessly v{VERSION} ({sys.platform}/{platform.machine()})…", file=sys.stderr)
        _download(f"{release_base_url()}/{asset}", archive)
        _download(f"{release_base_url()}/SHA256SUMS", checksums)
        _verify_checksum(archive, checksums)
        extracted = temporary_path / "extracted"
        extracted.mkdir()
        _extract(archive, format_name, extracted)
        package_root = next((entry for entry in extracted.iterdir() if entry.is_dir() and entry.name.startswith("utharnessly-")), None)
        if package_root is None:
            raise RuntimeError("release archive did not contain an utharnessly directory")
        shutil.copy2(package_root / binary.name, binary)
        if os.name != "nt":
            binary.chmod(0o755)
        shutil.copytree(package_root / "ui", ui)
    return binary, ui


def main() -> int:
    args = sys.argv[1:]
    if args in (["--version"], ["-V"]):
        print(f"utharnessly {VERSION}")
        return 0
    if args and args[0] == "update":
        try:
            ensure_binary(force=True)
            print(f"utharnessly {VERSION} is ready.")
            return 0
        except Exception as error:  # pragma: no cover - platform/network-specific
            print(f"utharnessly update failed: {error}", file=sys.stderr)
            return 1
    if args and args[0] == "uninstall":
        print("Remove the Python package with: python -m pip uninstall utharnessly")
        cache_command = f'rmdir /s /q "{cache_root()}"' if os.name == "nt" else f'rm -rf "{cache_root()}"'
        print(f"Remove the cached native runtime with: {cache_command}")
        return 0
    try:
        binary, _ui = ensure_binary()
    except Exception as error:  # pragma: no cover - platform/network-specific
        print(f"utharnessly: {error}", file=sys.stderr)
        return 1
    environment = os.environ.copy()
    environment["UTHARNESS_RUNTIME_BIN"] = str(binary)
    completed = subprocess.run([str(binary), *args], env=environment, check=False)
    return completed.returncode


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())
