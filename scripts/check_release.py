#!/usr/bin/env python3
"""Exercise a packaged binary using synthetic data and an isolated home.

Optional --ssh requires ssh, sshd and ssh-keygen on PATH and permission to bind
a loopback port. No user configuration, credentials or corpus files are used.
"""
import argparse
import getpass
import hashlib
import json
import os
from pathlib import Path
import select
import shlex
import shutil
import socket
import sqlite3
import subprocess
import tempfile
import time


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


def exchange(child, request):
    child.stdin.write(json.dumps(request) + "\n")
    child.stdin.flush()
    if not select.select([child.stdout], [], [], 30)[0]:
        raise AssertionError("protocol response timed out")
    return json.loads(child.stdout.readline())


def stop(child):
    if child.poll() is None:
        child.terminate()
    try:
        child.wait(timeout=5)
    except subprocess.TimeoutExpired:
        child.kill()
        child.wait()


def smoke(binary, peer, root, env):
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
        assert db.execute("SELECT id FROM tombstones WHERE id=?", (forgotten,)).fetchone()
        # Recreate the supported previous schema, without changing original bytes.
        db.executescript("DROP INDEX records_writer_created; DROP INDEX records_writer_expiry; PRAGMA user_version=4;")
    cli("restored.db", "list")
    with sqlite3.connect(root / "restored.db") as db:
        assert db.execute("PRAGMA user_version").fetchone()[0] == 5
    assert originals(root / "restored.db") == before
    cli("restored.db", "index:check")

    # A truncated backup must fail before creating the destination.
    damaged = root / "truncated.jsonl"
    damaged.write_bytes(backup.read_bytes()[:-1])
    cli("uncreated.db", "import", damaged, code=2)
    assert not (root / "uncreated.db").exists()

    envelope = json.dumps({"id": "interrupted", "body": "Original interrupted transfer."})
    item = {
        "id": "interrupted", "envelope": envelope,
        "envelope_hash": hashlib.sha256(envelope.encode()).hexdigest(),
        "body_hash": hashlib.sha256(b"Original interrupted transfer.").hexdigest(),
    }
    with (root / "protocol.stderr").open("w") as log:
        child = subprocess.Popen(
            [str(peer), "--store", str(root / "interrupted.db"), "sync:serve", "--stdio"],
            env=env, cwd=root, text=True, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=log,
        )
        try:
            assert exchange(child, {"op": "hello", "version": 1}) == {"result": "hello", "version": 1}
            assert exchange(child, {"op": "apply", "item": item, "scope": None}) == {"result": "applied", "changed": True}
            child.kill()  # Original committed, no finish/publication message.
            child.wait(timeout=5)
        finally:
            stop(child)
    cli("interrupted.db", "list", code=2)
    cli("interrupted.db", "sync")
    cli("interrupted.db", "index:check")
    assert json.loads(cli("interrupted.db", "show", "interrupted", "--all", "--json"))["text"] == "Original interrupted transfer."
    print("PASS: isolated binary, search/show, exact backup restore, retry, schema upgrade, truncation and interrupted publication")


def ssh_smoke(binary, peer, root, env):
    tools = {name: shutil.which(name) for name in ("ssh", "sshd", "ssh-keygen")}
    if not all(tools.values()):
        raise RuntimeError("--ssh requires ssh, sshd and ssh-keygen on PATH")
    for name in ("host", "client"):
        run([tools["ssh-keygen"], "-q", "-t", "ed25519", "-N", "", "-f", root / name], env=env, cwd=root)
    wrapper = root / "remote-command"
    wrapper.write_text(
        "#!/bin/sh\n"
        f"export PATH={shlex.quote(str(peer.parent) + os.pathsep + env['PATH'])}\n"
        f"export HOME={shlex.quote(env['HOME'])}\n"
        "export GIT_CONFIG_NOSYSTEM=1\n"
        'exec /bin/sh -c "$SSH_ORIGINAL_COMMAND"\n'
    )
    wrapper.chmod(0o700)
    (root / "authorized_keys").write_text(
        f'command="{wrapper}",no-agent-forwarding,no-port-forwarding,no-pty ' + (root / "client.pub").read_text()
    )
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        port = sock.getsockname()[1]
    (root / "sshd.conf").write_text(
        f"Port {port}\nListenAddress 127.0.0.1\nHostKey {root}/host\n"
        f"AuthorizedKeysFile {root}/authorized_keys\nPidFile {root}/pid\n"
        "StrictModes no\nPasswordAuthentication no\nKbdInteractiveAuthentication no\nUsePAM no\n"
        f"AllowUsers {getpass.getuser()}\nLogLevel VERBOSE\n"
    )
    (root / "known_hosts").write_text(f"[127.0.0.1]:{port} " + (root / "host.pub").read_text())
    (root / "ssh.conf").write_text(
        f"Host linn-test\n HostName 127.0.0.1\n Port {port}\n User {getpass.getuser()}\n"
        f" IdentityFile {root}/client\n IdentitiesOnly yes\n UserKnownHostsFile {root}/known_hosts\n"
        " GlobalKnownHostsFile /dev/null\n StrictHostKeyChecking yes\n"
    )
    (root / "ssh").write_text(f"#!/bin/sh\nexec {shlex.quote(tools['ssh'])} -F {shlex.quote(str(root / 'ssh.conf'))} \"$@\"\n")
    (root / "ssh").chmod(0o700)
    client_env = dict(env, PATH=str(root) + os.pathsep + env["PATH"])
    with (root / "sshd.log").open("w+") as log:
        server = subprocess.Popen([tools["sshd"], "-D", "-e", "-f", str(root / "sshd.conf")], stdout=log, stderr=log, env=env)
        try:
            for _ in range(100):
                if server.poll() is not None:
                    log.seek(0)
                    raise RuntimeError(log.read())
                try:
                    with socket.create_connection(("127.0.0.1", port), timeout=.1):
                        break
                except OSError:
                    time.sleep(.05)
            else:
                raise RuntimeError("loopback sshd did not become ready")
            local, remote = root / "ssh-local.db", root / "ssh-peer.db"
            def cli(exe, db, *args, **kwargs):
                return run([exe, "--store", db, *args], env=client_env, cwd=root, **kwargs)
            record = json.loads(cli(peer, remote, "add", "-", "--json", input="Original SSH resolver memory."))["record"]
            cli(binary, local, "sync:remote", "pair", "peer", f"ssh://linn-test{remote}")
            assert json.loads(cli(binary, local, "sync", "--json")) == {"pulled": 1, "pushed": 0, "remotes": 1}
            assert json.loads(cli(binary, local, "sync", "--json")) == {"pulled": 0, "pushed": 0, "remotes": 1}
            cli(binary, local, "add:forget", record)
            assert json.loads(cli(binary, local, "sync", "--json"))["pushed"] == 1
            for exe, db in [(binary, local), (peer, remote)]:
                cli(exe, db, "index:check")
                with sqlite3.connect(db) as conn:
                    assert conn.execute("SELECT id FROM tombstones WHERE id=?", (record,)).fetchone()
                    assert not conn.execute("SELECT id FROM records WHERE id=?", (record,)).fetchone()
            # A host-key mismatch must prevent the next transfer.
            (root / "known_hosts").write_text(f"[127.0.0.1]:{port} " + (root / "client.pub").read_text())
            cli(binary, local, "sync", code=2)
            print("PASS: SSH key authentication, strict host verification, pull/retry, deletion propagation and bad-host rejection")
        finally:
            stop(server)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--peer-binary", type=Path, help="Optional compatible peer executable for protocol compatibility testing")
    parser.add_argument("--ssh", action="store_true", help="Also run a real loopback SSH test (requires local socket permissions)")
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="linn-release-") as directory, tempfile.TemporaryDirectory(
        prefix="ls-", dir=os.environ.get("LINN_TEST_TMPDIR", "/tmp")
    ) as runtime:
        Path(runtime).chmod(0o700)
        root = Path(directory).resolve()
        for name, source in [("local-bin", args.binary), ("peer-bin", args.peer_binary or args.binary)]:
            (root / name).mkdir()
            shutil.copy2(source.resolve(), root / name / "linn")
        (root / "home").mkdir()
        env = {
            "HOME": str(root / "home"), "PATH": os.environ.get("PATH", ""),
            "XDG_CONFIG_HOME": str(root / "config"), "XDG_DATA_HOME": str(root / "data"),
            "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull,
            "LANG": "C", "TMPDIR": str(root), "XDG_RUNTIME_DIR": runtime,
        }
        binary, peer = root / "local-bin/linn", root / "peer-bin/linn"
        try:
            smoke(binary, peer, root, env)
            if args.ssh:
                ssh_smoke(binary, peer, root, env)
        finally:
            for store in root.rglob("*.db"):
                subprocess.run(
                    [str(binary), "--store", str(store), "daemon:stop"],
                    env=env, cwd=root, capture_output=True, timeout=30,
                )


if __name__ == "__main__":
    main()
