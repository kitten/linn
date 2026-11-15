use serde::{Deserialize, Serialize};
use std::{
    hash::{Hash, Hasher},
    sync::OnceLock,
};
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Regex {
    pattern: String,
    #[serde(skip)]
    compiled: OnceLock<regex::Regex>,
}
impl Clone for Regex {
    fn clone(&self) -> Self {
        Self::new(self.pattern.clone())
    }
}
impl Hash for Regex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pattern.hash(state);
    }
}
impl Regex {
    pub fn new(pattern: String) -> Self {
        Self {
            pattern,
            compiled: OnceLock::new(),
        }
    }
    fn compiled(&self) -> &regex::Regex {
        self.compiled
            .get_or_init(|| compile(&self.pattern).expect("unsupported resource regex"))
    }
    pub fn is_match(&self, text: &str) -> bool {
        self.compiled().is_match(text)
    }
    pub fn captures<'t>(&self, text: &'t str) -> Option<regex::Captures<'t>> {
        self.compiled().captures(text)
    }
    pub fn find_iter<'r, 't>(&'r self, text: &'t str) -> regex::Matches<'r, 't> {
        self.compiled().find_iter(text)
    }
    #[cfg(test)]
    pub fn captures_iter<'r, 't>(&'r self, text: &'t str) -> regex::CaptureMatches<'r, 't> {
        self.compiled().captures_iter(text)
    }
}
const URL: &str = "(https?:\\/\\/(?:www\\.|(?!www))[a-zA-Z0-9][a-zA-Z0-9-]+[a-zA-Z0-9]\\.[^\\s]{2,}|www\\.[a-zA-Z0-9][a-zA-Z0-9-]+[a-zA-Z0-9]\\.[^\\s]{2,}|https?:\\/\\/(?:www\\.|(?!www))[a-zA-Z0-9]+\\.[^\\s]{2,}|www\\.[a-zA-Z0-9]+\\.[^\\s]{2,})";
pub(crate) fn compile(pattern: &str) -> Result<regex::Regex, regex::Error> {
    if pattern != URL {
        return regex::Regex::new(pattern);
    }
    let a = "[a-zA-Z0-9]";
    let m = "[a-zA-Z0-9-]";
    let n = "[a-vx-zA-Z0-9]";
    let nm = "[a-vx-zA-Z0-9-]";
    let long = format!("{a}{m}+{a}");
    let restricted_long = format!("(?:{n}{m}+{a}|w{nm}{m}*{a}|ww{n}|ww{nm}{m}*{a})");
    let short = format!("(?:{n}{a}*|w{n}{a}*|ww{n}{a}*|ww|w)");
    regex::Regex::new(&format!(
        r"(https?://(?:www\.{long}|{restricted_long})\.[^\s]{{2,}}|www\.{long}\.[^\s]{{2,}}|https?://(?:www\.{a}+|{short})\.[^\s]{{2,}}|www\.{a}+\.[^\s]{{2,}})"
    ))
}
#[cfg(test)]
mod cache_tests {
    use super::*;
    #[test]
    fn clone_and_decode_start_fresh() {
        let r = Regex::new("a".into());
        assert!(r.compiled.get().is_none());
        assert!(r.is_match("a"));
        assert!(r.compiled.get().is_some());
        assert!(r.clone().compiled.get().is_none());
        let decoded: Regex = bincode::deserialize(&bincode::serialize(&r).unwrap()).unwrap();
        assert!(decoded.compiled.get().is_none());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compare(pattern: &str, text: &str) {
        let old = fancy_regex::Regex::new(pattern).unwrap();
        let new = Regex::new(pattern.to_owned());
        compare_compiled(&old, &new, text);
    }

    fn compare_compiled(old: &fancy_regex::Regex, new: &Regex, text: &str) {
        let expected: Vec<_> = old
            .captures_iter(text)
            .map(|c| {
                c.unwrap()
                    .iter()
                    .map(|m| m.map(|m| (m.start(), m.end(), m.as_str())))
                    .collect::<Vec<_>>()
            })
            .collect();
        let actual: Vec<_> = new
            .captures_iter(text)
            .map(|c| {
                c.iter()
                    .map(|m| m.map(|m| (m.start(), m.end(), m.as_str())))
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(actual, expected, "captures: {text:?}");
        let expected: Vec<_> = old
            .find_iter(text)
            .map(|m| {
                let m = m.unwrap();
                (m.start(), m.end(), m.as_str())
            })
            .collect();
        let actual: Vec<_> = new
            .find_iter(text)
            .map(|m| (m.start(), m.end(), m.as_str()))
            .collect();
        assert_eq!(actual, expected, "spans: {text:?}");
        assert_eq!(new.is_match(text), old.is_match(text).unwrap(), "{text:?}");
        let first = new.captures(text).map(|c| {
            c.iter()
                .map(|m| m.map(|m| (m.start(), m.end())))
                .collect::<Vec<_>>()
        });
        let expected = old.captures(text).unwrap().map(|c| {
            c.iter()
                .map(|m| m.map(|m| (m.start(), m.end())))
                .collect::<Vec<_>>()
        });
        assert_eq!(first, expected, "first capture: {text:?}");
    }

    // fancy 0.5 rewrites empty alternatives to greedy optional expressions.
    // No pinned pattern uses an empty alternative; arbitrary legacy patterns
    // are not covered by the pinned-resource compatibility contract.
    #[test]
    fn legacy_empty_alternative_limit() {
        assert_eq!(
            fancy_regex::Regex::new("(?:|a)")
                .unwrap()
                .find_iter("a")
                .next()
                .unwrap()
                .unwrap()
                .as_str(),
            "a"
        );
        assert_eq!(
            Regex::new("(?:|a)".into())
                .find_iter("a")
                .next()
                .unwrap()
                .as_str(),
            ""
        );
    }

    #[test]
    fn regular_adapter_contract() {
        for pattern in ["(a)?(b)", "()", "a*", "(?m)^|$", "(é)|(x)", "a|ab", "ab|a"] {
            for text in ["", "éaab\n", "xx b a", "ab", "\n", "é🙂"] {
                compare(pattern, text);
            }
        }
        let re = Regex::new("(a)?b".into());
        assert!(re.compiled.get().is_none());
        assert!(re.is_match("xxb"));
        assert!(re.compiled.get().is_some());
        assert!(re.clone().compiled.get().is_none());
        assert!(compile("(?=unsupported)").is_err());
        assert!(
            std::panic::catch_unwind(|| Regex::new("(?=unsupported)".into()).is_match("")).is_err()
        );
        let raw = Regex::new(URL.into());
        assert_eq!(
            bincode::serialize(&raw).unwrap(),
            bincode::serialize(URL).unwrap()
        );
        let decoded: Regex = bincode::deserialize(&bincode::serialize(&raw).unwrap()).unwrap();
        assert_eq!(decoded.pattern, URL);
        assert!(decoded.compiled.get().is_none());
        use std::collections::hash_map::DefaultHasher;
        let hash = |v: &dyn Fn(&mut DefaultHasher)| {
            let mut h = DefaultHasher::new();
            v(&mut h);
            h.finish()
        };
        assert_eq!(hash(&|h| raw.hash(h)), hash(&|h| URL.hash(h)));
    }

    // Frozen input generator: all words length 0..=6 over this host alphabet,
    // all words length 0..=3 over the suffix alphabet, plus fixed-seed noise.
    #[test]
    fn url_differential() {
        let old = fancy_regex::Regex::new(URL).unwrap();
        let new = Regex::new(URL.into());
        let mut count = 0;
        let mut check = |text: &str| {
            compare_compiled(&old, &new, text);
            count += 1;
        };
        fn words(alphabet: &[char], depth: usize, s: &mut String, visit: &mut impl FnMut(&str)) {
            visit(s);
            if depth == 0 {
                return;
            }
            for c in alphabet {
                s.push(*c);
                words(alphabet, depth - 1, s, visit);
                s.pop();
            }
        }
        words(
            &['w', 'W', 'a', '0', '-', '.'],
            6,
            &mut String::new(),
            &mut |host| {
                for prefix in ["http://", "https://", "www.", "https://www."] {
                    check(&format!("é{prefix}{host}.xy\nwww.a.bc"));
                }
            },
        );
        words(
            &['a', '.', '/', ')', '\n', 'é', '\u{a0}'],
            3,
            &mut String::new(),
            &mut |suffix| {
                for host in [
                    "a", "ab", "abc", "a-b", "www", "wwwx", "wwW", "w-w", "www.a",
                ] {
                    check(&format!("https://{host}.{suffix}"));
                }
            },
        );
        for text in [
            "http://www.abc.xy",
            "http://wwwx.xy",
            "http://ww.xy",
            "http://www.xy",
            "http://a.xy\n",
            "http://a.xy\nhttps://b.zz",
            "http://a.xyhttps://abc.zz",
            "www.a.bc,https://abc.de",
            "http://www.www.abc.xy",
            "https://a-b.cd🙂)",
        ] {
            check(text);
        }
        let alphabet = [
            'w', 'W', 'a', '0', '-', '.', '/', ':', '\n', '\r', '\t', ' ', 'é', '🙂', '\u{a0}', ')',
        ];
        let mut seed = 0xc0ffee123456789u64;
        for _ in 0..20_000 {
            let mut text = String::new();
            for part in 0..3 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                if part != 0 {
                    text.push_str(
                        ["http://", "https://", "www.", "https://www."][(seed >> 32) as usize % 4],
                    );
                }
                for _ in 0..(seed as usize % 30) {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    text.push(alphabet[(seed >> 32) as usize % alphabet.len()]);
                }
            }
            check(&text);
        }
        assert_eq!(count, 247_558);
    }
}
