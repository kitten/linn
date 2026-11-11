#!/usr/bin/env python3
"""Create allowlisted release archives and verify the complete release manifest."""
import argparse
import gzip
import hashlib
import io
from pathlib import Path
import re
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parent.parent
TARGETS = {
    "aarch64-darwin": "macos-arm64",
    "x86_64-linux": "linux-x86_64",
    "aarch64-linux": "linux-arm64",
}


def version(tag="", root=ROOT):
    value = tomllib.loads((root / "Cargo.toml").read_text())["package"]["version"]
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?", value):
        raise ValueError("release version must be major.minor.patch with an optional prerelease")
    if tag and tag != f"v{value}":
        raise ValueError(f"tag {tag!r} does not match Cargo.toml version v{value}")
    return value


def filename(value, system):
    return f"linn-{value}-{TARGETS[system]}.tar.gz"


def checksum(path):
    with path.open("rb") as file:
        return hashlib.file_digest(file, "sha256").hexdigest()


def package(source, output, system, value):
    name = filename(value, system)
    prefix = name.removesuffix(".tar.gz")
    files = {
        "linn": source / "bin/linn",
        "LICENSE": source / "share/doc/linn/LICENSE",
        "THIRD_PARTY_NOTICES": source / "share/doc/linn/THIRD_PARTY_NOTICES",
        "NLP-LICENSE": source / "share/doc/linn/NLP-LICENSE",
        "README.md": ROOT / "README.md",
    }
    # Validate everything before writing an archive; never recurse over the checkout.
    for file in files.values():
        if not file.is_file() or file.stat().st_size == 0:
            raise ValueError(f"missing or empty release input: {file}")
    output.mkdir(parents=True, exist_ok=True)
    archive = output / name
    with archive.open("xb") as raw, gzip.GzipFile(filename="", fileobj=raw, mode="wb", mtime=0) as zipped:
        with tarfile.open(fileobj=zipped, mode="w", format=tarfile.USTAR_FORMAT) as tar:
            for relative, source_file in sorted(files.items()):
                data = source_file.read_bytes()
                entry = tarfile.TarInfo(f"{prefix}/{relative}")
                entry.size = len(data)
                entry.mode = 0o755 if relative == "linn" else 0o644
                entry.mtime = entry.uid = entry.gid = 0
                tar.addfile(entry, io.BytesIO(data))
    (output / f"{name}.sha256").write_text(f"{checksum(archive)}  {name}\n")
    return archive


def manifest(directory, value):
    expected = sorted(filename(value, system) for system in TARGETS)
    actual = sorted(p.name for p in directory.glob("*.tar.gz"))
    if actual != expected:
        raise ValueError(f"release archives differ: expected {expected}, got {actual}")
    lines = []
    for name in expected:
        line = f"{checksum(directory / name)}  {name}\n"
        if (directory / f"{name}.sha256").read_text() != line:
            raise ValueError(f"checksum mismatch: {name}")
        lines.append(line)
    (directory / "SHA256SUMS").write_text("".join(lines))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--version", action="store_true")
    mode.add_argument("--package", type=Path)
    mode.add_argument("--manifest", type=Path)
    parser.add_argument("--system", choices=TARGETS)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--tag", default="")
    args = parser.parse_args()
    try:
        value = version(args.tag)
        if args.version:
            print(value)
        elif args.manifest:
            manifest(args.manifest, value)
        else:
            if not args.system or not args.output:
                parser.error("--package requires --system and --output")
            print(package(args.package, args.output, args.system, value))
    except (ValueError, OSError) as error:
        parser.exit(1, f"{error}\n")


if __name__ == "__main__":
    main()
