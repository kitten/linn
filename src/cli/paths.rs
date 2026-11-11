//! @ref LLP 0004#runtime
use crate::error::{Error, Result};
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};
fn xdg(key: &str, fallback: &str) -> Result<PathBuf> {
    if let Some(path) = env::var_os(key).filter(|v| !v.is_empty()) {
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err(Error::Invalid(format!("{key} must be absolute")));
        }
        Ok(path)
    } else {
        let home = env::var_os("HOME").ok_or_else(|| Error::Invalid("HOME is not set".into()))?;
        Ok(PathBuf::from(home).join(fallback))
    }
}
pub fn store_path(explicit: Option<PathBuf>) -> Result<PathBuf> {
    // Validate supplied XDG values even when a higher-precedence path wins.
    let data = xdg("XDG_DATA_HOME", ".local/share")?;
    let config = xdg("XDG_CONFIG_HOME", ".config")?;
    if let Some(path) = explicit {
        return Ok(path);
    }
    if let Some(path) = env::var_os("LINN_STORE") {
        return Ok(path.into());
    }
    let config = config.join("linn/config.toml");
    if config.exists() {
        let value: toml::Value = toml::from_str(&std::fs::read_to_string(&config)?)
            .map_err(|e| Error::Invalid(format!("{}: {e}", config.display())))?;
        if let Some(path) = value.get("store").and_then(toml::Value::as_str) {
            return Ok(PathBuf::from(path));
        }
    }
    Ok(data.join("linn/store.db"))
}
fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}
pub fn writer_email() -> Option<String> {
    writer_email_with(git)
}

fn writer_email_with(mut git: impl FnMut(&[&str]) -> Option<String>) -> Option<String> {
    git(&["config", "--get", "user.email"])
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

pub fn scope() -> Option<String> {
    scope_with(git)
}
fn scope_with(mut git: impl FnMut(&[&str]) -> Option<String>) -> Option<String> {
    let root = git(&["rev-parse", "--show-toplevel"])?;
    if let Some(verbose) = git(&["remote", "-v"])
        && let Some(url) = verbose_remote(&verbose)
    {
        return Some(
            url.and_then(remote_scope)
                .unwrap_or_else(|| format!("local:{root}")),
        );
    }
    // Unrecognized output retains Git's canonical single-remote URL lookup.
    let remotes = git(&["remote"])?;
    let remotes: Vec<_> = remotes.lines().collect();
    let name = if remotes.contains(&"upstream") {
        Some("upstream")
    } else if remotes.contains(&"origin") {
        Some("origin")
    } else if remotes.len() == 1 {
        Some(remotes[0])
    } else {
        None
    };
    name.and_then(|n| git(&["remote", "get-url", n]))
        .and_then(|url| remote_scope(&url))
        .or_else(|| Some(format!("local:{root}")))
}
fn verbose_remote(output: &str) -> Option<Option<&str>> {
    let mut remotes = std::collections::BTreeMap::new();
    for line in output.lines() {
        let (name, description) = line.split_once('\t')?;
        if name.is_empty() {
            return None;
        }
        let entry = remotes.entry(name).or_insert(None);
        if let Some((url, suffix)) = description.rsplit_once(" (fetch)") {
            // Git may append a partial-clone filter after the fetch marker.
            if !suffix.is_empty() && !(suffix.starts_with(" [") && suffix.ends_with(']')) {
                return None;
            }
            entry.get_or_insert(url.trim());
        } else if !description.ends_with(" (push)") {
            return None;
        }
    }
    let selected = remotes
        .get("upstream")
        .or_else(|| remotes.get("origin"))
        .or_else(|| {
            (remotes.len() == 1)
                .then(|| remotes.values().next())
                .flatten()
        });
    // Missing fetch URLs require the ordinary get-url fallback.
    match selected {
        Some(Some(url)) => Some(Some(url)),
        Some(None) => None,
        None => Some(None),
    }
}
pub fn remote_scope(url: &str) -> Option<String> {
    let path = if let Some((_, rest)) = url.split_once("://") {
        rest.split_once('/').map(|(_, p)| p)?
    } else {
        url.split_once(':').map(|(_, p)| p)?
    };
    let path = path
        .trim_matches('/')
        .strip_suffix(".git")
        .unwrap_or(path.trim_matches('/'));
    (!path.is_empty()).then(|| path.to_owned())
}
pub fn source_path(path: &Path) -> Result<String> {
    let absolute = std::fs::canonicalize(path)?;
    let root = git(&["rev-parse", "--show-toplevel"]).map(PathBuf::from);
    Ok(root
        .and_then(|root| absolute.strip_prefix(root).ok().map(PathBuf::from))
        .unwrap_or(absolute)
        .to_string_lossy()
        .into())
}

/// Git computes the evidence blob with the checkout's attribute filters.
pub fn evidence(path: &Path) -> Result<String> {
    let absolute = std::fs::canonicalize(path)?;
    if git(&["rev-parse", "--show-toplevel"]).is_none_or(|root| !absolute.starts_with(root)) {
        return Ok(crate::storage::record::hash(&std::fs::read(absolute)?));
    }
    let output = Command::new("git")
        .args(["hash-object", "--"])
        .arg(&absolute)
        .output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().into())
    } else {
        Ok(crate::storage::record::hash(&std::fs::read(absolute)?))
    }
}

#[cfg(test)]
mod scope_tests {
    use super::*;

    fn run(dir: &Path, args: &[&str]) -> Option<String> {
        let output = Command::new("git")
            .current_dir(dir)
            .env_clear()
            .env("PATH", env::var_os("PATH").unwrap_or_default())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_COUNT", "0")
            .args(args)
            .output()
            .unwrap();
        output
            .status
            .success()
            .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }
    fn original(dir: &Path) -> Option<String> {
        let root = run(dir, &["rev-parse", "--show-toplevel"])?;
        let names = run(dir, &["remote"])?;
        let names: Vec<_> = names.lines().collect();
        let name = if names.contains(&"upstream") {
            Some("upstream")
        } else if names.contains(&"origin") {
            Some("origin")
        } else if names.len() == 1 {
            Some(names[0])
        } else {
            None
        };
        Some(
            name.and_then(|n| run(dir, &["remote", "get-url", n]))
                .and_then(|s| remote_scope(&s))
                .unwrap_or_else(|| format!("local:{root}")),
        )
    }
    fn check(dir: &Path, calls: usize) {
        let mut actual_calls = 0;
        let actual = scope_with(|args| {
            actual_calls += 1;
            run(dir, args)
        });
        assert_eq!(actual, original(dir));
        assert_eq!(actual_calls, calls);
    }
    #[test]
    fn verbose_scope_preserves_git_url_selection_and_rewrites() {
        let dir = tempfile::tempdir().unwrap();
        check(dir.path(), 1);
        assert!(run(dir.path(), &["init", "-q"]).is_some());
        check(dir.path(), 2);
        for (key, value) in [
            ("remote.other.url", "https://example.invalid/team/other.git"),
            (
                "remote.second.url",
                "ssh://git@example.invalid/group/second.git",
            ),
            ("remote.origin.url", "short:group/origin.git"),
            ("url.ssh://git@example.invalid/.insteadOf", "short:"),
            (
                "remote.origin.pushurl",
                "https://elsewhere.invalid/push/only.git",
            ),
            (
                "remote.upstream.url",
                "git@example.invalid:group/subgroup/upstream.git",
            ),
            ("remote.upstream.partialclonefilter", "blob:none"),
        ] {
            assert!(run(dir.path(), &["config", key, value]).is_some());
            check(dir.path(), 2);
        }
        assert!(
            run(
                dir.path(),
                &[
                    "config",
                    "--add",
                    "remote.upstream.url",
                    "https://example.invalid/second-url.git"
                ]
            )
            .is_some()
        );
        check(dir.path(), 2);
        assert!(
            run(
                dir.path(),
                &["config", "--unset-all", "remote.upstream.url"]
            )
            .is_some()
        );
        assert!(
            run(
                dir.path(),
                &[
                    "config",
                    "remote.upstream.fetch",
                    "+refs/heads/*:refs/remotes/upstream/*"
                ]
            )
            .is_some()
        );
        check(dir.path(), 4);
        assert!(
            run(
                dir.path(),
                &[
                    "config",
                    "remote.upstream.url",
                    "ssh://example.invalid/line\nbreak.git"
                ]
            )
            .is_some()
        );
        check(dir.path(), 4);
        let bare = tempfile::tempdir().unwrap();
        assert!(run(bare.path(), &["init", "--bare", "-q"]).is_some());
        check(bare.path(), 1);
    }
}

#[cfg(test)]
mod writer_tests {
    use super::*;

    #[test]
    fn configured_email_or_unknown() {
        for (value, expected) in [
            (Some(" person@example.org\n"), Some("person@example.org")),
            (Some(""), None),
            (Some("  \n"), None),
            (None, None),
        ] {
            assert_eq!(
                writer_email_with(|args| {
                    assert_eq!(args, ["config", "--get", "user.email"]);
                    value.map(str::to_owned)
                }),
                expected.map(str::to_owned)
            );
        }
    }
}
