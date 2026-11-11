import hashlib
from pathlib import Path
import tarfile
import tempfile
import unittest

from package_release import TARGETS, filename, manifest, package, version


class ReleasePackaging(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / "package"
        for relative in ["bin/linn", "share/doc/linn/LICENSE",
                         "share/doc/linn/THIRD_PARTY_NOTICES", "share/doc/linn/NLP-LICENSE"]:
            path = self.source / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"synthetic package input\n")
        (self.source / "private.db").write_bytes(b"must not ship")

    def test_archive_allowlist_modes_and_reproducibility(self):
        first = package(self.source, self.root / "a", "aarch64-darwin", "0.1.0")
        second = package(self.source, self.root / "b", "aarch64-darwin", "0.1.0")
        self.assertEqual(first.read_bytes(), second.read_bytes())
        with tarfile.open(first) as tar:
            entries = tar.getmembers()
            self.assertEqual({Path(e.name).name for e in entries},
                             {"linn", "LICENSE", "THIRD_PARTY_NOTICES", "NLP-LICENSE", "README.md"})
            for entry in entries:
                self.assertTrue(entry.isfile())
                self.assertEqual(entry.mode, 0o755 if entry.name.endswith("/linn") else 0o644)
                self.assertEqual(entry.uid, 0)
        digest = hashlib.sha256(first.read_bytes()).hexdigest()
        self.assertEqual(first.with_name(first.name + ".sha256").read_text(),
                         f"{digest}  {first.name}\n")

    def test_tag_matches_manifest_version(self):
        (self.root / "Cargo.toml").write_text('[package]\nversion = "1.2.3-beta.1"\n')
        self.assertEqual(version("v1.2.3-beta.1", self.root), "1.2.3-beta.1")
        for bad in ["v1.2.3", "1.2.3-beta.1", "v9.0.0", "v1.2.3/unsafe"]:
            with self.assertRaises(ValueError):
                version(bad, self.root)

    def test_manifest_rejects_missing_extra_or_corrupt_archives(self):
        output = self.root / "dist"
        for system in TARGETS:
            package(self.source, output, system, "0.1.0")
        manifest(output, "0.1.0")
        self.assertEqual(len((output / "SHA256SUMS").read_text().splitlines()), 3)
        archive = output / filename("0.1.0", "aarch64-linux")
        original = archive.read_bytes()
        archive.write_bytes(original + b"corrupt")
        with self.assertRaises(ValueError):
            manifest(output, "0.1.0")
        archive.write_bytes(original)
        extra = output / "unexpected.tar.gz"
        extra.write_bytes(b"extra")
        with self.assertRaises(ValueError):
            manifest(output, "0.1.0")
        extra.unlink()
        archive.unlink()
        with self.assertRaises(ValueError):
            manifest(output, "0.1.0")

    def test_missing_notice_and_existing_output_are_refused(self):
        package(self.source, self.root / "dist", "x86_64-linux", "0.1.0")
        with self.assertRaises(FileExistsError):
            package(self.source, self.root / "dist", "x86_64-linux", "0.1.0")
        (self.source / "share/doc/linn/NLP-LICENSE").unlink()
        with self.assertRaises(ValueError):
            package(self.source, self.root / "missing", "x86_64-linux", "0.1.0")
        self.assertFalse((self.root / "missing").exists())


if __name__ == "__main__":
    unittest.main()
