import hashlib
import platform
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from utharnessly import __version__
from utharnessly.cli import VERSION, _verify_checksum, platform_asset


class LauncherTests(unittest.TestCase):
    def test_version_is_consistent(self):
        self.assertEqual(VERSION, __version__)

    @patch.object(platform, "machine", return_value="x86_64")
    @patch.object(sys, "platform", "linux")
    def test_linux_x64_asset(self, _machine):
        self.assertEqual(platform_asset(), ("utharnessly-linux-x64.tar.gz", "tar.gz"))

    @patch.object(platform, "machine", return_value="AMD64")
    @patch.object(sys, "platform", "win32")
    def test_windows_x64_asset(self, _machine):
        self.assertEqual(platform_asset(), ("utharnessly-windows-x64.zip", "zip"))

    @patch.object(platform, "machine", return_value="arm64")
    @patch.object(sys, "platform", "darwin")
    def test_macos_arm64_asset(self, _machine):
        self.assertEqual(platform_asset(), ("utharnessly-macos-arm64.tar.gz", "tar.gz"))

    @patch.object(platform, "machine", return_value="aarch64")
    @patch.object(sys, "platform", "linux")
    def test_unsupported_architecture_is_explicit(self, _machine):
        with self.assertRaisesRegex(RuntimeError, "No published utharnessly binary"):
            platform_asset()

    def test_checksum_requires_an_exact_asset_name(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive = root / "release.tar.gz"
            archive.write_bytes(b"verified")
            digest = hashlib.sha256(b"verified").hexdigest()
            checksums = root / "SHA256SUMS"
            checksums.write_text(f"{digest}  prefix-release.tar.gz\n", encoding="utf-8")
            with self.assertRaisesRegex(RuntimeError, "does not contain"):
                _verify_checksum(archive, checksums)
            checksums.write_text(f"{digest}  release.tar.gz\n", encoding="utf-8")
            _verify_checksum(archive, checksums)


if __name__ == "__main__":
    unittest.main()
