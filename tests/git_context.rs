//! File-based caller scope must preserve Git's effective configuration semantics.
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
fn command(program: &str, cwd: &Path, env: &[(&str, &str)]) -> Command {
    let mut command = Command::new(program);
    command
        .current_dir(cwd)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_COUNT", "0")
        .env_remove("GIT_CONFIG_PARAMETERS");
    for (key, value) in env {
        command.env(key, value);
    }
    command
}
fn git(cwd: &Path, args: &[&str]) -> String {
    let out = command("git", cwd, &[]).args(args).output().unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_owned()
}
async fn check(cwd: &Path, db: &Path, env: &[(&str, &str)], expected: Option<&str>) {
    let mut child = command(env!("CARGO_BIN_EXE_linn"), cwd, env)
        .arg("--store")
        .arg(db)
        .args(["add", "-", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"Original caller-context verification.\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let database = libsql::Builder::new_local(db).build().await.unwrap();
    let connection = database.connect().unwrap();
    let mut rows = connection
        .query(
            "SELECT original_scope FROM records WHERE id=?",
            [value["record"].as_str().unwrap()],
        )
        .await
        .unwrap();
    let actual: Option<String> = rows.next().await.unwrap().unwrap().get(0).unwrap();
    assert_eq!(actual.as_deref(), expected, "cwd={}", cwd.display());
}
#[tokio::test]
async fn file_configuration_scope_covers_legacy_includes_and_worktrees() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let repo = root.join("repo");
    std::fs::create_dir(&repo).unwrap();
    git(&repo, &["init", "-q"]);
    git(
        &repo,
        &["config", "remote.origin.url", "ssh://host/group/origin.git"],
    );
    check(&repo, &root.join("context.db"), &[], Some("group/origin")).await;
    // Search must resolve file-based scope even when invoking Git is unavailable.
    let fake = root.join("no-git");
    std::fs::create_dir(&fake).unwrap();
    let git_log = root.join("git-called");
    std::fs::write(
        fake.join("git"),
        "#!/bin/sh\n: > \"$GIT_CALL_LOG\"\nexit 42\n",
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(fake.join("git"), std::fs::Permissions::from_mode(0o700)).unwrap();
    let output = command(env!("CARGO_BIN_EXE_linn"), &repo, &[])
        .arg("--store")
        .arg(root.join("context.db"))
        .args(["search", "verification", "--json"])
        .env("PATH", &fake)
        .env("GIT_CALL_LOG", &git_log)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!git_log.exists(), "scope must not invoke a Git subprocess");
    // Older dotted headers are valid Git configuration, with lowercase subsections.
    let legacy = root.join("legacy.inc");
    std::fs::write(&legacy,"[remote.UpStream]\nurl = short:group/legacy.git\n[url \"ssh://host/\"]\ninsteadOf = short:\n").unwrap();
    git(&repo, &["config", "include.path", legacy.to_str().unwrap()]);
    assert!(git(&repo, &["remote", "-v"]).contains("ssh://host/group/legacy.git (fetch)"));
    check(&repo, &root.join("context.db"), &[], Some("group/legacy")).await;
    git(&repo, &["config", "--unset", "include.path"]);
    // Longest prefix wins; a push-only rewrite must not alter fetch scope.
    git(
        &repo,
        &[
            "config",
            "remote.origin.url",
            "short:specific/repository.git",
        ],
    );
    git(
        &repo,
        &["config", "url.ssh://host/broad/.insteadOf", "short:"],
    );
    git(
        &repo,
        &[
            "config",
            "url.ssh://host/narrow/.insteadOf",
            "short:specific/",
        ],
    );
    git(
        &repo,
        &[
            "config",
            "url.ssh://host/push-only/.pushInsteadOf",
            "short:",
        ],
    );
    assert!(git(&repo, &["remote", "-v"]).contains("ssh://host/narrow/repository.git (fetch)"));
    check(
        &repo,
        &root.join("context.db"),
        &[],
        Some("narrow/repository"),
    )
    .await;
    git(
        &repo,
        &[
            "config",
            "--add",
            "remote.origin.url",
            "ssh://host/ignored/second.git",
        ],
    );
    check(
        &repo,
        &root.join("context.db"),
        &[],
        Some("narrow/repository"),
    )
    .await;
    // URL strings are original metadata, including valid quoted escapes.
    git(&repo, &["config", "--unset-all", "remote.origin.url"]);
    git(
        &repo,
        &[
            "config",
            "remote.origin.url",
            "ssh://host/group/line\nbreak.git",
        ],
    );
    check(
        &repo,
        &root.join("context.db"),
        &[],
        Some("group/line\nbreak"),
    )
    .await;
    // Command-scoped injected remote config is intentionally outside this file-based resolver.
    let env = [
        ("GIT_CONFIG_COUNT", "1"),
        ("GIT_CONFIG_KEY_0", "remote.upstream.url"),
        ("GIT_CONFIG_VALUE_0", "ssh://host/group/environment.git"),
    ];
    let native = command("git", &repo, &env)
        .args(["remote", "-v"])
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&native.stdout)
            .contains("ssh://host/group/environment.git (fetch)")
    );
    check(
        &repo,
        &root.join("context.db"),
        &env,
        Some("group/line\nbreak"),
    )
    .await;
    // Conditional includes are resolved relative to the real Git directory.
    let included = root.join("conditional.inc");
    std::fs::write(
        &included,
        "[remote \"upstream\"]\nurl = ssh://host/group/conditional.git\n",
    )
    .unwrap();
    let key = format!(
        "includeIf.gitdir:{}/.git.path",
        std::fs::canonicalize(&repo).unwrap().display()
    );
    git(&repo, &["config", &key, included.to_str().unwrap()]);
    assert!(git(&repo, &["remote", "-v"]).contains("ssh://host/group/conditional.git (fetch)"));
    check(
        &repo,
        &root.join("context.db"),
        &[],
        Some("group/conditional"),
    )
    .await;
    git(&repo, &["config", "--unset", &key]);
    // Global configuration contributes before the repository's additional URLs.
    let global = root.join("global.config");
    std::fs::write(
        &global,
        "[remote \"upstream\"]\nurl = ssh://host/group/global-first.git\n",
    )
    .unwrap();
    git(
        &repo,
        &[
            "config",
            "remote.upstream.url",
            "ssh://host/group/local-second.git",
        ],
    );
    let global_env = [("GIT_CONFIG_GLOBAL", global.to_str().unwrap())];
    let native = command("git", &repo, &global_env)
        .args(["remote", "get-url", "upstream"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8(native.stdout).unwrap().trim(),
        "ssh://host/group/global-first.git"
    );
    check(
        &repo,
        &root.join("context.db"),
        &global_env,
        Some("group/global-first"),
    )
    .await;
    git(&repo, &["config", "--unset", "remote.upstream.url"]);

    git(
        &repo,
        &["config", "remote.origin.url", "ssh://host/group/origin.git"],
    );
    git(
        &repo,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "--allow-empty",
            "-qm",
            "seed",
        ],
    );
    let worktree = root.join("worktree");
    git(
        &repo,
        &[
            "worktree",
            "add",
            "-qb",
            "worktree",
            worktree.to_str().unwrap(),
        ],
    );
    check(
        &worktree,
        &root.join("context.db"),
        &[],
        Some("group/origin"),
    )
    .await;
    git(&repo, &["config", "extensions.worktreeConfig", "true"]);
    git(
        &worktree,
        &[
            "config",
            "--worktree",
            "remote.upstream.url",
            "ssh://host/group/worktree.git",
        ],
    );
    check(
        &worktree,
        &root.join("context.db"),
        &[],
        Some("group/worktree"),
    )
    .await;
    let bare = root.join("bare");
    std::fs::create_dir(&bare).unwrap();
    git(&bare, &["init", "--bare", "-q"]);
    check(&bare, &root.join("context.db"), &[], None).await;
    check(root, &root.join("context.db"), &[], None).await;
}

#[tokio::test]
async fn file_source_ref_reuses_body_hash_without_git_commands() {
    for format in ["sha1", "sha256"] {
        file_input_case(format).await;
    }
}
async fn file_input_case(format: &str) {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::Builder::new()
        .prefix("lgi-")
        .tempdir_in("/tmp")
        .unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let repo = dir.path().join("repo");
    std::fs::create_dir(&repo).unwrap();
    git(&repo, &["init", "-q", &format!("--object-format={format}")]);
    git(&repo, &["config", "user.email", "file@example.org"]);
    git(
        &repo,
        &[
            "config",
            "filter.marker.clean",
            "printf '%s' \"$CLEAN_MARKER\"; cat",
        ],
    );
    std::fs::write(repo.join(".gitattributes"), "*.md filter=marker\n").unwrap();
    let body = "Original snapshot.\r\n";
    std::fs::write(repo.join("document.md"), body).unwrap();
    let db = dir.path().join("store.db");
    let bin = dir.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let log = dir.path().join("git-calls");
    let quote = |s: &str| format!("'{}'", s.replace('\'', "'\\''"));
    std::fs::write(
        bin.join("git"),
        format!(
            "#!/bin/sh\nprintf 'called\\n' >> {}\nexit 42\n",
            quote(log.to_str().unwrap())
        ),
    )
    .unwrap();
    std::fs::set_permissions(bin.join("git"), std::fs::Permissions::from_mode(0o700)).unwrap();
    let path = format!("{}:{}", bin.display(), std::env::var("PATH").unwrap());
    let cli = |cwd: &Path| {
        let mut command = command(env!("CARGO_BIN_EXE_linn"), cwd, &[]);
        command
            .arg("--store")
            .arg(&db)
            .env("XDG_RUNTIME_DIR", dir.path())
            .env("PATH", &path);
        command
    };
    let first = cli(dir.path()).args(["list", "--json"]).output().unwrap();
    assert_eq!(
        first.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let output = cli(&repo)
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "user.email")
        .env("GIT_CONFIG_VALUE_0", "ignored@example.org")
        .args(["add", "document.md", "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let id: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let database = libsql::Builder::new_local(&db).build().await.unwrap();
    let connection = database.connect().unwrap();
    let mut rows = connection
        .query(
            "SELECT envelope,body_hash FROM records WHERE id=?",
            [id["record"].as_str().unwrap()],
        )
        .await
        .unwrap();
    let row = rows.next().await.unwrap().unwrap();
    let envelope: serde_json::Value =
        serde_json::from_slice(&row.get::<Vec<u8>>(0).unwrap()).unwrap();
    let body_hash: String = row.get(1).unwrap();
    assert_eq!(envelope["body"], body);
    assert_eq!(envelope["writer"], "file@example.org");
    assert_eq!(envelope["source_path"], "document.md");
    assert_eq!(envelope["source"]["ref"], body_hash);
    assert_eq!(body_hash, linn::record::hash(body.as_bytes()));
    assert!(!log.exists(), "file ingest must not invoke Git");
    assert_eq!(
        std::fs::read_dir(repo.join(".git/objects"))
            .unwrap()
            .count(),
        2
    );
}
