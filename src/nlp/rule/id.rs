// Portions derived from nlprule 0.6.4. See src/nlp/LICENSE.
//! Wire identifiers retained for diagnostics.
use serde::{Deserialize, Serialize};
use std::fmt;
/// Identifies a category.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Category(String);

/// Identifies a rule group.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Group {
    parent: Category,
    inner: String,
}

/// Identifies the children of a rule group.
/// Rules without a group in the original XML are treated as a rule group with one child.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Index {
    parent: Group,
    inner: usize,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.parent, self.inner)
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.parent, self.inner)
    }
}
