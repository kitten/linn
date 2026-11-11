// Portions derived from nlprule 0.6.4. See src/nlp/LICENSE.
//! An abstraction on top of Regular Expressions to add support for Serialization and
//! to modularize the Regex backend.
//! Adapts the approach from https://github.com/trishume/syntect/pull/270 with feature flags for the
//! different backends.

use once_cell::sync::OnceCell;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::hash::{Hash, Hasher};

pub use regex_impl::CaptureMatches;

#[derive(Debug)]
pub struct Regex {
    regex_str: String,
    regex: OnceCell<regex_impl::Regex>,
}

impl Clone for Regex {
    fn clone(&self) -> Self {
        Regex {
            regex_str: self.regex_str.clone(),
            regex: OnceCell::new(),
        }
    }
}

impl Serialize for Regex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.regex_str)
    }
}

impl<'de> Deserialize<'de> for Regex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let regex_str = String::deserialize(deserializer)?;
        Ok(Regex::new(regex_str))
    }
}

impl Hash for Regex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.regex_str.hash(state);
    }
}

impl Regex {
    /// Create a new regex from the pattern string.
    ///
    /// Note that the regex compilation happens on first use, which is why this method does not
    /// return a result.
    pub fn new(regex_str: String) -> Self {
        Self {
            regex_str,
            regex: OnceCell::new(),
        }
    }

    /// Check whether the pattern compiles as a valid regex.
    #[allow(dead_code)] // used only in compile module
    pub fn try_compile(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        regex_impl::Regex::new(&self.regex_str).map(|_| ())
    }

    fn regex(&self) -> &regex_impl::Regex {
        self.regex.get_or_init(|| {
            regex_impl::Regex::new(&self.regex_str)
                .unwrap_or_else(|_| panic!("regex string should be pre-tested: {}", self.regex_str))
        })
    }

    pub fn is_match(&self, text: &str) -> bool {
        self.regex().is_match(text)
    }

    pub fn captures_iter<'r, 't>(&'r self, text: &'t str) -> regex_impl::CaptureMatches<'r, 't> {
        self.regex().captures_iter(text)
    }

    pub fn find_iter<'r, 't>(&'r self, text: &'t str) -> regex_impl::Matches<'r, 't> {
        self.regex().find_iter(text)
    }

    #[allow(dead_code)] // used only in compile module
    pub fn captures_len(&self) -> usize {
        self.regex().captures_len()
    }

    pub fn captures<'t>(&self, text: &'t str) -> Option<regex_impl::Captures<'t>> {
        self.regex().captures(text)
    }
}

mod regex_impl_fancy {
    pub use fancy_regex::{Captures, Match};
    use std::error::Error;
    pub struct Matches<'r, 't>(fancy_regex::Matches<'r, 't>);

    impl<'r, 't> Iterator for Matches<'r, 't> {
        type Item = Match<'t>;

        fn next(&mut self) -> Option<Self::Item> {
            match self.0.next() {
                Some(Ok(mat)) => Some(mat),
                // stop if an error is encountered
                None | Some(Err(_)) => None,
            }
        }
    }

    pub struct CaptureMatches<'r, 't>(fancy_regex::CaptureMatches<'r, 't>);

    impl<'r, 't> Iterator for CaptureMatches<'r, 't> {
        type Item = Captures<'t>;

        fn next(&mut self) -> Option<Self::Item> {
            match self.0.next() {
                Some(Ok(caps)) => Some(caps),
                // stop if an error is encountered
                None | Some(Err(_)) => None,
            }
        }
    }

    #[derive(Debug)]
    pub struct Regex {
        regex: fancy_regex::Regex,
    }

    impl Regex {
        pub fn new(regex_str: &str) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
            Ok(Regex {
                regex: fancy_regex::Regex::new(regex_str)?,
            })
        }

        pub fn is_match(&self, text: &str) -> bool {
            // errors are treated as non-matches
            self.regex.is_match(text).unwrap_or(false)
        }

        pub fn captures_iter<'r, 't>(&'r self, text: &'t str) -> CaptureMatches<'r, 't> {
            CaptureMatches(self.regex.captures_iter(text))
        }

        pub fn find_iter<'r, 't>(&'r self, text: &'t str) -> Matches<'r, 't> {
            Matches(self.regex.find_iter(text))
        }

        pub fn captures_len(&self) -> usize {
            self.regex.captures_len()
        }

        pub fn captures<'t>(&self, text: &'t str) -> Option<Captures<'t>> {
            match self.regex.captures(text) {
                Ok(Some(captures)) => Some(captures),
                // errors treated as not matching
                Ok(None) | Err(_) => None,
            }
        }
    }
}

use regex_impl_fancy as regex_impl;
