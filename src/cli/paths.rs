//! @ref LLP 0004#runtime
use crate::error::{Error, Result};
use std::{
    env,
    path::{Path, PathBuf},
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
pub fn writer_email() -> Option<String> {
    let config = match repository() {
        Ok(repo) => repo.config().ok()?,
        Err(error) if error.code() == git2::ErrorCode::NotFound => default_config()?,
        Err(_) => return None,
    };
    use git2::ConfigLevel::*;
    let global = env::var_os("GIT_CONFIG_GLOBAL");
    for level in [Worktree, Local, Global, XDG, System, ProgramData] {
        // libgit2 still loads XDG alongside an explicit global override.
        if (level == XDG && global.is_some())
            || (level == Global && global.as_ref().is_some_and(|path| path.is_empty()))
        {
            continue;
        }
        let Ok(config) = config.open_level(level) else {
            continue;
        };
        match config.get_entry("user.email") {
            Ok(entry) => return email(entry.has_value().then(|| entry.value_bytes())),
            Err(error) if error.code() == git2::ErrorCode::NotFound => {}
            Err(_) => return None,
        }
    }
    None
}
fn email(value: Option<&[u8]>) -> Option<String> {
    value
        .map(|value| String::from_utf8_lossy(value).trim().to_owned())
        .filter(|value| !value.is_empty())
}
fn default_config() -> Option<git2::Config> {
    use git2::{Config, ConfigLevel};
    let mut config = Config::new().ok()?;
    let system = if env::var("GIT_CONFIG_NOSYSTEM")
        .ok()
        .and_then(|value| Config::parse_bool(value).ok())
        .unwrap_or(false)
    {
        None
    } else {
        env::var_os("GIT_CONFIG_SYSTEM")
            .map(PathBuf::from)
            .or_else(|| Config::find_system().ok())
    };
    let global = env::var_os("GIT_CONFIG_GLOBAL");
    let xdg = if global.is_none() {
        Config::find_xdg().ok()
    } else {
        None
    };
    let global = global
        .map(PathBuf::from)
        .or_else(|| Config::find_global().ok());
    for (path, level) in [
        (system, ConfigLevel::System),
        (xdg, ConfigLevel::XDG),
        (global, ConfigLevel::Global),
    ] {
        if let Some(path) = path.filter(|path| !path.as_os_str().is_empty()) {
            match config.add_file(&path, level, false) {
                Ok(()) => {}
                Err(error) if error.code() == git2::ErrorCode::NotFound => {}
                Err(_) => return None,
            }
        }
    }
    Some(config)
}

pub fn scope() -> Option<String> {
    scope_in(&repository().ok()?)
}
fn repository() -> std::result::Result<git2::Repository, git2::Error> {
    git2::Repository::open_ext(
        ".",
        git2::RepositoryOpenFlags::FROM_ENV,
        None::<&std::ffi::OsStr>,
    )
}
fn workdir() -> Option<PathBuf> {
    repository().ok()?.workdir()?.canonicalize().ok()
}
fn scope_in(repo: &git2::Repository) -> Option<String> {
    let root = repo.workdir()?;
    let config = repo.config().ok()?;
    let mut entries = config.entries(None).ok()?;
    let mut names = std::collections::BTreeSet::new();
    while let Some(entry) = entries.next() {
        let entry = entry.ok()?;
        let key = String::from_utf8_lossy(entry.name_bytes());
        if let Some((name, _)) = key
            .strip_prefix("remote.")
            .and_then(|key| key.rsplit_once('.'))
        {
            names.insert(name.to_owned());
        }
    }
    let selected = names
        .iter()
        .find(|name| name.as_str() == "upstream")
        .or_else(|| names.iter().find(|name| name.as_str() == "origin"))
        .or_else(|| (names.len() == 1).then(|| names.first()).flatten());
    selected
        .and_then(|name| fetch_remote(repo, &config, name))
        .and_then(|remote| remote_scope(String::from_utf8_lossy(remote.url_bytes()).trim()))
        .or_else(|| Some(format!("local:{}", root.canonicalize().ok()?.display())))
}
// libgit2's named-remote API selects the last URL; Linn uses Git's first fetch URL.
fn fetch_remote<'a>(
    repo: &'a git2::Repository,
    config: &git2::Config,
    name: &str,
) -> Option<git2::Remote<'a>> {
    let mut values = config.multivar(&format!("remote.{name}.url"), None).ok()?;
    let mut first = None;
    while let Some(entry) = values.next() {
        let entry = entry.ok()?;
        if entry.value_bytes().is_empty() {
            first = None;
        } else if first.is_none() {
            first = Some(entry.value_bytes().to_vec());
        }
    }
    repo.remote_anonymous(&String::from_utf8_lossy(&first?))
        .ok()
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
    let root = workdir();
    Ok(root
        .and_then(|root| absolute.strip_prefix(root).ok().map(PathBuf::from))
        .unwrap_or(absolute)
        .to_string_lossy()
        .into())
}

pub fn evidence(path: &Path) -> Result<String> {
    Ok(crate::storage::record::hash(&std::fs::read(path)?))
}

pub(crate) fn file_input(path: &Path) -> Result<crate::application::FileInput> {
    Ok(crate::application::FileInput {
        path: std::fs::canonicalize(path)?,
        repository: workdir(),
    })
}

#[cfg(test)]
mod scope_tests {
    use super::*;
    use std::process::Command;

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
    fn check(dir: &Path) {
        let actual = git2::Repository::discover(dir).ok().and_then(|repo| {
            let config = git2::Config::open(&repo.path().join("config")).ok()?;
            repo.set_config(&config).ok()?;
            scope_in(&repo)
        });
        assert_eq!(actual, original(dir));
    }
    #[test]
    fn file_scope_preserves_git_url_selection_and_rewrites() {
        let dir = tempfile::tempdir().unwrap();
        check(dir.path());
        assert!(run(dir.path(), &["init", "-q"]).is_some());
        check(dir.path());
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
            check(dir.path());
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
        check(dir.path());
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
        check(dir.path());
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
        check(dir.path());
        let bare = tempfile::tempdir().unwrap();
        assert!(run(bare.path(), &["init", "--bare", "-q"]).is_some());
        check(bare.path());
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
            assert_eq!(email(value.map(str::as_bytes)), expected.map(str::to_owned));
        }
    }
}
