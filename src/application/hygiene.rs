//! @ref LLP 0004#write-time-hygiene-warnings
use regex::Regex;
use serde_json::Value;
use std::{path::Path, sync::LazyLock};

static PATTERNS: LazyLock<Vec<(&str, Regex)>> = LazyLock::new(|| {
    [
        ("GitHub token", r"gh[pousr]_[A-Za-z0-9]{36,255}"),
        (
            "GitHub fine-grained token",
            r"github_pat_[A-Za-z0-9_]{50,255}",
        ),
        ("AWS access ID", r"AKIA[A-Z0-9]{16}"),
        ("Slack token", r"xox[abprs]-[A-Za-z0-9-]{20,255}"),
        (
            "API key",
            r"sk-(?:ant|proj|svcacct)-[A-Za-z0-9_-]{20,255}|sk-[A-Za-z0-9]{48}",
        ),
        (
            "private key",
            r"-----BEGIN (?:RSA |EC |DSA |OPENSSH |ENCRYPTED )?PRIVATE KEY-----",
        ),
    ]
    .into_iter()
    .map(|(name, pattern)| (name, Regex::new(pattern).unwrap()))
    .collect()
});
static LINKS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"`[^`\n]*`|\]\(([^)]+)\)").unwrap());

pub(crate) enum Check<'a> {
    Add,
    Export,
    NoteLinks(&'a Path),
}

fn ephemeral(path: &str) -> bool {
    let path = path.strip_prefix("file://").unwrap_or(path);
    if [
        "/tmp/",
        "/private/tmp/",
        "/var/folders/",
        "/private/var/folders/",
    ]
    .iter()
    .any(|prefix| path.starts_with(prefix))
    {
        return true;
    }
    if path.contains("://") {
        return false;
    }
    let parts: Vec<_> = path.split('/').collect();
    parts
        .iter()
        .any(|p| matches!(*p, "scratchpad" | "scratchpads" | ".scratchpad"))
        || parts.windows(2).any(|p| {
            matches!(p, [".claude", "projects"] | [".codex", "sessions"])
                && (path.starts_with("~/") || path.starts_with('/'))
        })
}

pub(crate) fn scan(record: &Value, check: Check<'_>) -> Vec<String> {
    let mut warnings = Vec::new();
    if !matches!(check, Check::NoteLinks(_)) {
        let mut strings = Vec::new();
        let mut pending = vec![record];
        while let Some(value) = pending.pop() {
            match value {
                Value::String(s) => strings.push(s.as_str()),
                Value::Array(values) => pending.extend(values),
                Value::Object(values) => pending.extend(values.values()),
                _ => {}
            }
        }
        for (name, pattern) in PATTERNS.iter() {
            if strings.iter().any(|text| {
                pattern.find_iter(text).any(|m| {
                    let token = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-';
                    !text[..m.start()].chars().next_back().is_some_and(token)
                        && !text[m.end()..].chars().next().is_some_and(token)
                })
            }) {
                warnings.push(format!("Possible secret: {name}"));
            }
        }
    }
    if matches!(check, Check::Export) {
        return warnings;
    }
    let source = record["source_path"].as_str().unwrap_or("");
    if matches!(check, Check::NoteLinks(_))
        && (source.contains(':') || source.starts_with('~') || ephemeral(source))
    {
        return warnings;
    }
    if matches!(check, Check::Add) && ephemeral(source) {
        warnings.push("Ephemeral citation in source_path".into());
    }
    let body = record["body"].as_str().unwrap_or("");
    for captures in LINKS.captures_iter(body) {
        let Some(destination) = captures.get(1) else {
            continue;
        };
        let destination = destination.as_str().trim();
        let destination = if let Some(angled) = destination.strip_prefix('<') {
            angled.split('>').next().unwrap_or(angled)
        } else {
            destination.split_whitespace().next().unwrap_or("")
        };
        let warning = match check {
            Check::Add if ephemeral(destination) => "Ephemeral citation in Markdown link",
            Check::NoteLinks(root) => {
                let path = destination.split(['#', '?']).next().unwrap_or("");
                if path.is_empty()
                    || path.contains(':')
                    || path.starts_with(['/', '~'])
                    || ephemeral(path)
                {
                    continue;
                }
                let base = Path::new(source).parent().unwrap_or(Path::new(""));
                let resolved = root.join(base).join(path);
                if !std::fs::metadata(resolved)
                    .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
                {
                    continue;
                }
                "Unresolved relative repository link in note"
            }
            _ => continue,
        };
        if !warnings.iter().any(|w| w == warning) {
            warnings.push(warning.into());
        }
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn secret_families_boundaries_and_dedup() {
        for token in [
            format!("ghp_{}", "A".repeat(36)),
            format!("github_pat_{}", "B".repeat(50)),
            format!("AKIA{}", "C".repeat(16)),
            format!("xoxb-{}", "D".repeat(24)),
            format!("sk-ant-{}", "E".repeat(24)),
            format!("sk-proj-{}", "F".repeat(24)),
            format!("sk-svcacct-{}", "G".repeat(24)),
            format!("sk-{}", "H".repeat(48)),
            format!("-----BEGIN {}-----", "OPENSSH PRIVATE KEY"),
        ] {
            let warnings = scan(
                &json!({"body": format!("`{token}`\n`{token}`")}),
                Check::Add,
            );
            assert_eq!(warnings.len(), 1);
            assert!(!warnings[0].contains(&token));
            assert!(
                scan(
                    &json!({"body": format!("prefix{token}suffix")}),
                    Check::Export
                )
                .is_empty()
            );
            assert_eq!(scan(&json!({"label": token}), Check::Export).len(), 1);
        }
    }

    #[test]
    fn ephemeral_components_and_export_exclusion() {
        for path in [
            "/tmp/a",
            "/private/tmp/a",
            "/var/folders/a",
            "/private/var/folders/a",
            "~/.claude/projects/a",
            "/Users/test/.codex/sessions/a",
            "notes/scratchpad/a",
            ".scratchpad/a",
            "scratchpads/a",
        ] {
            let record = json!({"source_path":path,"body":format!("[citation]({path})")});
            assert_eq!(scan(&record, Check::Add).len(), 2, "{path}");
            assert!(scan(&record, Check::Export).is_empty());
        }
    }

    #[test]
    fn clean_input_and_relative_links() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("notes")).unwrap();
        std::fs::write(dir.path().join("README.md"), "exists").unwrap();
        let examples =
            json!({"body":"`handlers[i](event)` and `](dest)` and `[example](/tmp/fake.md)`"});
        assert!(scan(&examples, Check::Add).is_empty());
        assert!(scan(&examples, Check::NoteLinks(dir.path())).is_empty());
        let clean = json!({"source_path":"notes/scratchpadish.md", "body":"[ok](../README.md#intro) [title](../README.md \"Read me\") [angled](<../README.md> \"Read me\") [web](https://example.test/scratchpad/a) [anchor](#a) ghp_ github_pat_ AKIA sk-ant- /tmp/prose"});
        assert!(scan(&clean, Check::Add).is_empty());
        assert!(scan(&clean, Check::NoteLinks(dir.path())).is_empty());
        let missing = json!({"source_path":"notes/note.md", "body":"[missing](missing.rs) [missing again](missing.rs)"});
        assert_eq!(scan(&missing, Check::NoteLinks(dir.path())).len(), 1);
        for source in [
            "https://example.test/docs/note.md",
            "/tmp/fake-note.md",
            "~/notes/note.md",
        ] {
            let record = json!({"source_path":source,"body":"[readme](README.md)"});
            assert!(scan(&record, Check::NoteLinks(dir.path())).is_empty());
        }
    }
}
