//! Integer posting-set traversal compatibility for stable one-under ties.
//!
//! @ref LLP 0002#lexical-evidence
//!
//! Choose the first counted posting after BM25 and ordinal ties.
//! Preserve its open-addressed integer posting traversal, including union growth,
//! rather than depending on Rust's randomized HashSet iteration.
pub struct Postings {
    table: Vec<Option<usize>>,
    used: usize,
}
impl Postings {
    pub fn new(ids: impl IntoIterator<Item = usize>) -> Self {
        let mut ids: Vec<_> = ids.into_iter().collect();
        ids.sort_unstable();
        ids.dedup();
        let mut out = Self {
            table: vec![None; 8],
            used: 0,
        };
        for id in ids {
            out.add(id);
        }
        out
    }
    fn add(&mut self, id: usize) {
        if insert(&mut self.table, id) {
            self.used += 1;
            if self.used * 5 >= (self.table.len() - 1) * 3 {
                self.resize(self.used * 4);
            }
        }
    }
    fn resize(&mut self, minimum: usize) {
        let mut size = 8;
        while size <= minimum {
            size *= 2;
        }
        if size == self.table.len() {
            return;
        }
        let old = std::mem::replace(&mut self.table, vec![None; size]);
        for id in old.into_iter().flatten() {
            insert(&mut self.table, id);
        }
    }
    pub fn merge(&mut self, other: &Self) {
        if other.used == 0 {
            return;
        }
        if (self.used + other.used) * 5 >= (self.table.len() - 1) * 3 {
            self.resize((self.used + other.used) * 2);
        }
        if self.used == 0 && self.table.len() == other.table.len() {
            self.table.clone_from(&other.table);
            self.used = other.used;
            return;
        }
        for id in other.table.iter().flatten() {
            self.add(*id);
        }
    }
    pub fn order(&self) -> Vec<usize> {
        self.table.iter().flatten().copied().collect()
    }
}
fn insert(table: &mut [Option<usize>], id: usize) -> bool {
    let mask = table.len() - 1;
    let mut i = id & mask;
    let mut perturb = id;
    loop {
        let probes = if i + 9 <= mask { 9 } else { 0 };
        for slot in &mut table[i..=i + probes] {
            if slot.is_none() {
                *slot = Some(id);
                return true;
            }
            if *slot == Some(id) {
                return false;
            }
        }
        perturb >>= 5;
        i = (i * 5 + 1 + perturb) & mask;
    }
}
