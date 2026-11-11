//! @ref LLP 0002#lexical-evidence
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};
use unicode_general_category::{GeneralCategory, get_general_category};
use unicode_normalization::UnicodeNormalization;
pub static WORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\p{L}\p{N}_][\p{L}\p{N}_-]*").unwrap());
static BARE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\p{L}\p{N}_-]+").unwrap());
static PREFIX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(^|\s)([a-z][a-z-]*):("([^"]*)"|\S+)"#).unwrap());
pub static QUOTE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#""([^"]+)""#).unwrap());
pub static CAMEL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[a-z][A-Z]").unwrap());
const STOP: &str = "a an and are as at be but by can do does for from has have how i if in into is it its of on or so that the their then there these this to use used using was we what when where which who why will with you your not no yes";
const KIND: &str = "command commands flag flags option options variable variables setting settings helper function version pinned recommendation replacement directory file files project repo repository memory note notes document docs documentation rule rules procedure";
pub fn stop(s: &str) -> bool {
    STOP.split_whitespace().any(|w| w == s)
}
pub fn norm(s: &str) -> String {
    s.to_lowercase()
        .nfd()
        .filter(|c| get_general_category(*c) != GeneralCategory::NonspacingMark)
        .collect()
}
pub fn surface(s: &str) -> Vec<String> {
    WORD.find_iter(s)
        .map(|m| m.as_str().to_lowercase())
        .collect()
}
pub fn words(s: &str) -> HashSet<String> {
    surface(s).iter().map(|s| norm(s)).collect()
}
pub fn split(q: &str) -> (Vec<(String, String)>, String) {
    let filters = PREFIX
        .captures_iter(q)
        .map(|c| {
            (
                c[2].to_owned(),
                c.get(4).or_else(|| c.get(3)).unwrap().as_str().to_owned(),
            )
        })
        .collect();
    (filters, PREFIX.replace_all(q, " ").to_string())
}
/// Rewrite parsed field values without replacing equal text elsewhere in the query.
pub(crate) fn rewrite_filter_values(
    q: &str,
    replacements: &HashMap<(String, String), String>,
) -> String {
    PREFIX
        .replace_all(q, |c: &regex::Captures<'_>| {
            let value = c.get(4).or_else(|| c.get(3)).unwrap().as_str();
            match replacements.get(&(c[2].to_owned(), value.to_owned())) {
                Some(replacement) => {
                    let quoted = c.get(4).is_some() || replacement.chars().any(char::is_whitespace);
                    format!(
                        "{}{}:{}{}{}",
                        &c[1],
                        &c[2],
                        if quoted { "\"" } else { "" },
                        replacement,
                        if quoted { "\"" } else { "" }
                    )
                }
                None => c[0].to_owned(),
            }
        })
        .into_owned()
}
pub fn content(q: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    surface(&split(q).1)
        .into_iter()
        .filter(|s| s.chars().count() > 2 && !stop(s) && seen.insert(s.clone()))
        .collect()
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Term {
    pub text: String,
    pub phrase: bool,
}
impl Term {
    pub fn hard(&self) -> bool {
        self.phrase || self.identifier() || CAMEL.is_match(&self.text)
    }
    pub fn identifier(&self) -> bool {
        self.text.contains(['_', '-'])
    }
    pub fn content(&self) -> bool {
        self.phrase || !KIND.split_whitespace().any(|w| w == norm(&self.text))
    }
    pub fn expression(&self, b: bool) -> String {
        let cols = if b && self.identifier() {
            "{body}"
        } else if self.hard() {
            "{heading body}"
        } else {
            "{heading body parts}"
        };
        format!("{cols}:{}", quote(&self.text))
    }
}
pub struct Query {
    pub(crate) original: String,
    pub filters: Vec<(String, String)>,
    pub terms: Vec<Term>,
    pub(crate) words: HashSet<String>,
    pub(crate) hard_words: Vec<String>,
    pub(crate) need: usize,
    paths: HashMap<String, PathPattern>,
}
impl Query {
    pub fn parse(q: &str) -> std::result::Result<Self, String> {
        let (filters, rest) = split(q);
        if rest.matches('"').count() % 2 != 0 {
            return Err("Unclosed quote".into());
        }
        if filters.iter().any(|(k, _)| {
            ![
                "scope",
                "path",
                "under",
                "kind",
                "writer",
                "state",
                "id",
                "cites",
                "linked-from",
                "version-of",
                "replaces",
                "sibling-of",
                "heading",
                "body",
            ]
            .contains(&k.as_str())
        }) {
            return Err("Unknown prefix".into());
        }
        let scan = Regex::new(r#""([^"]+)"|(\S+)"#).unwrap();
        let mut terms = Vec::new();
        for c in scan.captures_iter(&rest) {
            if let Some(p) = c.get(1) {
                terms.push(Term {
                    text: p.as_str().into(),
                    phrase: true,
                });
            } else {
                for m in BARE.find_iter(c[2].trim_matches(|c| ".,;:!?()[]{}'`".contains(c))) {
                    let text = m.as_str();
                    let n = norm(text);
                    if !stop(&n) || n == "not" || n == "no" {
                        terms.push(Term {
                            text: text.into(),
                            phrase: false,
                        });
                    }
                }
            }
        }
        let mut seen = HashSet::new();
        terms.retain(|t| seen.insert(t.clone()));
        let content: Vec<_> = terms.iter().filter(|t| t.content()).collect();
        let words = content.iter().map(|t| norm(&t.text)).collect();
        let need = need(content.len());
        let hard_words = terms
            .iter()
            .filter(|t| t.hard())
            .map(|t| norm(&t.text))
            .collect();
        let paths = filters
            .iter()
            .filter(|(k, _)| k == "path" || k == "cites")
            .map(|(_, v)| (v.clone(), PathPattern::new(v)))
            .collect();
        Ok(Self {
            original: q.to_owned(),
            filters,
            terms,
            words,
            hard_words,
            need,
            paths,
        })
    }
    pub fn content(&self) -> Vec<&Term> {
        self.terms.iter().filter(|t| t.content()).collect()
    }
    pub(crate) fn path_matches(&self, pattern: &str, path: &str) -> bool {
        self.paths.get(pattern).is_some_and(|p| p.matches(path))
    }
}
pub fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}
pub fn need(n: usize) -> usize {
    n.min(2.max(n.div_ceil(2)))
}
pub fn pathmatch(pat: &str, path: &str) -> bool {
    PathPattern::new(pat).matches(path)
}
struct PathPattern {
    regex: Option<Regex>,
    literal: Option<String>,
}
impl PathPattern {
    fn new(pat: &str) -> Self {
        let rx = regex::escape(pat)
            .replace(r"\*\*", ".*")
            .replace(r"\*", "[^/]*")
            .replace(r"\?", "[^/]");
        let regex = Regex::new(&format!(
            "{}{rx}$",
            if pat.starts_with('/') { "^" } else { "(^|/)" }
        ))
        .ok();
        Self {
            regex,
            literal: (!pat.contains(['*', '?'])).then(|| pat.to_owned()),
        }
    }
    fn matches(&self, path: &str) -> bool {
        self.regex.as_ref().is_some_and(|r| r.is_match(path))
            || self
                .literal
                .as_ref()
                .is_some_and(|value| path.contains(value))
    }
}
/// Quote-aware token walk preserves whitespace and quoted filter values.
pub fn rewrite(q: &str, mut f: impl FnMut(&str) -> String) -> String {
    let mut out = String::new();
    let mut pos = 0;
    while pos < q.len() {
        let rest = &q[pos..];
        let c = rest.chars().next().unwrap();
        if c == '"' {
            let end = rest[1..].find('"').map(|i| i + 2).unwrap_or(rest.len());
            out.push_str(&rest[..end]);
            pos += end;
        } else if c.is_whitespace() {
            out.push(c);
            pos += c.len_utf8();
        } else {
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            out.push_str(&f(&rest[..end]));
            pos += end;
        }
    }
    out
}
pub fn pieces(s: &str) -> Vec<String> {
    let re = Regex::new(r"([a-z0-9])([A-Z])").unwrap();
    re.replace_all(s, "$1 $2")
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| s.len() > 2 && s.chars().all(char::is_alphabetic))
        .map(str::to_lowercase)
        .collect()
}
