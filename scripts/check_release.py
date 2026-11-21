#!/usr/bin/env python3
"""Exercise a packaged binary using synthetic data and an isolated home.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import tempfile


def run(command, *, env, cwd, input=None, code=0):
    result = subprocess.run(
        [str(arg) for arg in command], env=env, cwd=cwd, input=input,
        text=True, capture_output=True, timeout=120,
    )
    if result.returncode != code:
        raise AssertionError(f"{command}: exit {result.returncode}\n{result.stderr}")
    return result.stdout


def originals(path):
    with sqlite3.connect(path) as db:
        return db.execute(
            "SELECT id,body,envelope,body_hash,envelope_hash FROM records ORDER BY id"
        ).fetchall()


def smoke(binary, root, env):
    def cli(db, *args, **kwargs):
        return run([binary, "--store", root / db, *args], env=env, cwd=root, **kwargs)

    body = "The resolver preserves query parameters. Café remains UTF-8.\n"
    record = json.loads(cli("source.db", "add", "-", "--json", input=body))["record"]
    page = json.loads(cli("source.db", "search", "resolver query parameters", "--json"))
    assert any(card["record"] == record for card in page["cards"])
    assert json.loads(cli("source.db", "show", record, "--all", "--json"))["text"] == body
    forgotten = json.loads(cli("source.db", "add", "-", "--json", input="Obsolete protocol detail."))["record"]
    cli("source.db", "add:forget", forgotten)
    before = originals(root / "source.db")

    backup = root / "backup.jsonl"
    cli("source.db", "export", "--all-states", "--output", backup)
    cli("restored.db", "import", backup)
    assert originals(root / "restored.db") == before
    assert json.loads(cli("restored.db", "import", backup, "--json"))["changed"] == 0
    with sqlite3.connect(root / "restored.db") as db:
        for (name,) in db.execute(
            "SELECT name FROM sqlite_schema WHERE type='trigger' AND name GLOB 'sync_sizes_*'"
        ).fetchall():
            db.execute(f'DROP TRIGGER "{name}"')
        db.executescript("DROP VIEW sync_size_source; DROP TABLE sync_sizes;")
        assert db.execute("SELECT id FROM tombstones WHERE id=?", (forgotten,)).fetchone()
        # Recreate the supported previous schema, without changing original bytes.
        db.executescript("DROP TABLE mesh_pending; DROP TABLE mesh_conflicts; DROP TABLE mesh_roots; DROP TABLE mesh_event_ids; DROP TABLE mesh_successors; DROP TABLE mesh_membership_states; DROP TABLE mesh_events; DROP TABLE mesh_identity; DROP TABLE sync_incoming; DROP INDEX records_writer_created; DROP INDEX records_writer_expiry; CREATE TABLE sync_remotes(name TEXT PRIMARY KEY,origin TEXT NOT NULL,scope TEXT); INSERT INTO sync_remotes VALUES('old','ssh://retired/store',NULL); PRAGMA user_version=4;")
    cli("restored.db", "list")
    with sqlite3.connect(root / "restored.db") as db:
        assert db.execute("PRAGMA user_version").fetchone()[0] == len(list(
            (Path(__file__).resolve().parents[1] / "src/storage/migrations").glob("[0-9][0-9][0-9][0-9]-*.sql")))
        assert not db.execute("SELECT 1 FROM sqlite_master WHERE name='sync_remotes'").fetchone()
        db.execute("INSERT OR REPLACE INTO sync_state VALUES('pending',1)")
    cli("restored.db", "daemon:stop")
    cli("restored.db", "sync")
    cli("restored.db", "list")
    with sqlite3.connect(root / "restored.db") as db:
        assert not db.execute("SELECT 1 FROM sync_state WHERE key='pending'").fetchone()
    assert originals(root / "restored.db") == before
    cli("restored.db", "index:check")

    # A truncated backup must fail before creating the destination.
    damaged = root / "truncated.jsonl"
    damaged.write_bytes(backup.read_bytes()[:-1])
    cli("uncreated.db", "import", damaged, code=2)
    assert not (root / "uncreated.db").exists()

    for command in ["sync:remote", "sync:serve"]:
        cli("source.db", command, code=2)
    cli("source.db", "sync")
    print("PASS: isolated binary, search/show, exact backup restore, retry, schema upgrade, truncation and retired-command rejection")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="linn-release-") as directory, tempfile.TemporaryDirectory(
        prefix="ls-", dir=os.environ.get("LINN_TEST_TMPDIR", "/tmp")
    ) as runtime:
        Path(runtime).chmod(0o700)
        root = Path(directory).resolve()
        (root / "bin").mkdir()
        binary = root / "bin/linn"
        shutil.copy2(args.binary.resolve(), binary)
        (root / "home").mkdir()
        env = {
            "HOME": str(root / "home"), "PATH": os.environ.get("PATH", ""),
            "XDG_CONFIG_HOME": str(root / "config"), "XDG_DATA_HOME": str(root / "data"),
            "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull,
            "LANG": "C", "TMPDIR": str(root), "XDG_RUNTIME_DIR": runtime,
        }
        try:
            smoke(binary, root, env)
        finally:
            for store in root.rglob("*.db"):
                subprocess.run(
                    [str(binary), "--store", str(store), "daemon:stop"],
                    env=env, cwd=root, capture_output=True, timeout=30,
                )


if __name__ == "__main__":
    main()
