//! Canonical serialization of lookup collections; runtime maps retain their types.
use serde::{Serialize, Serializer};
use std::collections::HashMap;

pub fn map<K: Serialize + Ord, V: Serialize, S: Serializer>(
    values: &HashMap<K, V>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut entries: Vec<_> = values.iter().collect();
    entries.sort_unstable_by(|a, b| a.0.cmp(b.0));
    serializer.collect_map(entries)
}
