#!/usr/bin/env python3
"""Validate Linn's Linked Literate Programming references."""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
LLP_DIR = ROOT / "llp"

SKIP_DIRS = {".git", "target", "dist", "build", "node_modules"}
TEXT_EXTENSIONS = {
    ".py",
    ".rs",
    ".sh",
    ".toml",
    ".txt",
    ".yaml",
    ".yml",
}
SKIP_FILES = {"scripts/check_llp_refs.py"}

REF_RE = re.compile(
    r"@ref\s+LLP\s+(\d{4})(?:#([A-Za-z0-9._-]+))?"
    r"(?:\s+\[[^\]]+\])?(?:\s+—[^\n]*)?"
)
HEADING_RE = re.compile(r"^(#{1,6})\s+(.*?)\s*$")
INLINE_CODE_RE = re.compile(r"`([^`]*)`")
HTML_TAG_RE = re.compile(r"<[^>]+>")
MARKDOWN_LINK_RE = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
EXTERNAL_URL_RE = re.compile(r"\b(?:https?|file)://", re.IGNORECASE)
MACHINE_LOCAL_RE = re.compile(r"(?:^|\s)(?:~/git/|/Users/|/home/)")


def slugify(heading: str) -> str:
    heading = INLINE_CODE_RE.sub(r"\1", heading)
    heading = HTML_TAG_RE.sub("", heading).strip().lower()
    chars: list[str] = []
    previous_dash = False
    for char in heading:
        if char.isalnum():
            chars.append(char)
            previous_dash = False
        elif char in {" ", "-"} and not previous_dash:
            chars.append("-")
            previous_dash = True
    return "".join(chars).strip("-")


def load_llp_docs() -> dict[str, tuple[Path, set[str]]]:
    if not LLP_DIR.is_dir():
        raise SystemExit("Missing llp/ directory")

    documents: dict[str, tuple[Path, set[str]]] = {}
    for path in sorted(LLP_DIR.glob("[0-9][0-9][0-9][0-9]-*.md")):
        number = path.name[:4]
        if number in documents:
            raise SystemExit(f"Duplicate LLP number {number}")

        anchors: set[str] = set()
        seen: dict[str, int] = {}
        for line in path.read_text(encoding="utf-8").splitlines():
            match = HEADING_RE.match(line)
            if match is None:
                continue
            base = slugify(match.group(2))
            if not base:
                continue
            count = seen.get(base, 0)
            seen[base] = count + 1
            anchors.add(base if count == 0 else f"{base}-{count}")
        documents[number] = (path, anchors)

    if not documents:
        raise SystemExit("No numbered LLP documents found")
    return documents


def scan_files() -> list[Path]:
    files: list[Path] = []
    for path in ROOT.rglob("*"):
        if not path.is_file():
            continue
        relative = path.relative_to(ROOT)
        # Captured corpora and historical experiment snapshots are not source.
        if relative.parts[:2] in {("evals", "output"), ("evals", "corpus")}:
            continue
        if relative.as_posix() in SKIP_FILES:
            continue
        if "llp" in relative.parts or any(part in SKIP_DIRS for part in relative.parts):
            continue
        if path.suffix in TEXT_EXTENSIONS:
            files.append(path)
    return sorted(files)


def validate_llp_reference_boundary(errors: list[str]) -> None:
    """Keep durable LLP rationale closed to this repository."""
    for path in sorted(LLP_DIR.glob("[0-9][0-9][0-9][0-9]-*.md")):
        for line_number, line in enumerate(
            path.read_text(encoding="utf-8").splitlines(), start=1
        ):
            relative = path.relative_to(ROOT).as_posix()
            prose = INLINE_CODE_RE.sub("", line)
            if EXTERNAL_URL_RE.search(prose):
                errors.append(
                    f"{relative}:{line_number}: LLP references an external URL"
                )
            if MACHINE_LOCAL_RE.search(line):
                errors.append(
                    f"{relative}:{line_number}: LLP references a machine-local document"
                )
            for target in MARKDOWN_LINK_RE.findall(line):
                target = target.split("#", 1)[0]
                if not target:
                    continue
                resolved = (path.parent / target).resolve()
                if not resolved.is_relative_to(ROOT) or not resolved.exists():
                    errors.append(
                        f"{relative}:{line_number}: LLP link target is not an "
                        "existing in-repository artifact"
                    )


def main() -> int:
    documents = load_llp_docs()
    errors: list[str] = []
    validate_llp_reference_boundary(errors)

    for path in scan_files():
        relative = path.relative_to(ROOT).as_posix()
        try:
            lines = path.read_text(encoding="utf-8").splitlines()
        except UnicodeDecodeError:
            errors.append(f"{relative}: unable to decode as UTF-8")
            continue

        for line_number, line in enumerate(lines, start=1):
            if "@ref" not in line:
                continue
            matches = list(REF_RE.finditer(line))
            if not matches:
                errors.append(f"{relative}:{line_number}: unsupported @ref syntax")
                continue
            for match in matches:
                number, anchor = match.groups()
                document = documents.get(number)
                if document is None:
                    errors.append(
                        f"{relative}:{line_number}: references missing LLP {number}"
                    )
                elif anchor and anchor not in document[1]:
                    target = document[0].relative_to(ROOT).as_posix()
                    errors.append(
                        f"{relative}:{line_number}: anchor #{anchor} not found "
                        f"in LLP {number} ({target})"
                    )

    if errors:
        print("LLP reference check failed.")
        for error in errors:
            print(f"- {error}")
        return 1

    print("LLP reference check passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
