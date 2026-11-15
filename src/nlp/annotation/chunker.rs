use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
    hash::{Hash, Hasher},
};
pub(crate) mod hash {
    use super::*;
    pub(crate) fn hash_str(s: &str) -> u64 {
        hash_slice(&[s])
    }
    pub(crate) fn hash_slice(parts: &[&str]) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for s in parts {
            h.write(s.as_bytes());
        }
        h.write_u8(255);
        h.finish()
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct FeatureMap {
    #[serde(with = "slot_bytes")]
    slots: Vec<Slot>,
}
#[derive(Clone)]
struct Slot {
    key: u64,
    offset: u32,
    len: u8,
}
impl FeatureMap {
    #[allow(dead_code)] // Used by the build-only model constructor.
    fn new(entries: Vec<(u64, u8)>) -> Self {
        let cap = (entries.len() * 2).max(1).next_power_of_two();
        let mut m = Self {
            slots: vec![
                Slot {
                    key: 0,
                    offset: u32::MAX,
                    len: 0
                };
                cap
            ],
        };
        let mut offset = 0;
        for (key, len) in entries {
            let mut i = key as usize & (cap - 1);
            while m.slots[i].offset != u32::MAX && m.slots[i].key != key {
                i = (i + 1) & (cap - 1);
            }
            m.slots[i] = Slot { key, offset, len };
            offset += u32::from(len);
        }
        m
    }
    fn get(&self, key: u64) -> Option<(usize, usize)> {
        let mut i = key as usize & (self.slots.len() - 1);
        loop {
            let s = &self.slots[i];
            if s.offset == u32::MAX {
                return None;
            }
            if s.key == key {
                return Some((s.offset as usize, s.len as usize));
            }
            i = (i + 1) & (self.slots.len() - 1);
        }
    }
}
mod raw_bytes {
    use serde::{Deserializer, Serializer};
    pub fn serialize<S: Serializer>(v: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(v)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        super::decode_bytes(d, |bytes| Ok(bytes.to_vec()))
    }
}
fn decode_bytes<'de, D, T>(
    d: D,
    decode: fn(&[u8]) -> Result<T, &'static str>,
) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Decoder<T>(fn(&[u8]) -> Result<T, &'static str>);
    impl<'de, T> serde::de::Visitor<'de> for Decoder<T> {
        type Value = T;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("packed bytes")
        }
        fn visit_bytes<E: serde::de::Error>(self, bytes: &[u8]) -> Result<T, E> {
            (self.0)(bytes).map_err(E::custom)
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<T, A::Error> {
            let mut bytes = Vec::with_capacity(seq.size_hint().unwrap_or(0));
            while let Some(byte) = seq.next_element::<u8>()? {
                bytes.push(byte);
            }
            self.visit_bytes(&bytes)
        }
    }
    d.deserialize_bytes(Decoder(decode))
}
mod float_bytes {
    use super::*;
    pub fn serialize<S: serde::Serializer>(v: &[f32], s: S) -> Result<S::Ok, S::Error> {
        let mut bytes = Vec::with_capacity(v.len() * 4);
        for x in v {
            bytes.extend_from_slice(&x.to_bits().to_le_bytes());
        }
        s.serialize_bytes(&bytes)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<f32>, D::Error> {
        decode_bytes(d, |b| {
            if b.len() % 4 != 0 {
                return Err("incomplete float");
            }
            Ok(b.chunks_exact(4)
                .map(|x| f32::from_bits(u32::from_le_bytes(x.try_into().unwrap())))
                .collect())
        })
    }
}
mod slot_bytes {
    use super::*;
    pub fn serialize<S: serde::Serializer>(v: &[Slot], s: S) -> Result<S::Ok, S::Error> {
        let mut b = Vec::with_capacity(v.len() * 13);
        for x in v {
            b.extend(x.key.to_le_bytes());
            b.extend(x.offset.to_le_bytes());
            b.push(x.len)
        }
        s.serialize_bytes(&b)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Slot>, D::Error> {
        decode_bytes(d, |b| {
            if b.len() % 13 != 0 {
                return Err("incomplete slot");
            }
            Ok(b.chunks_exact(13)
                .map(|x| Slot {
                    key: u64::from_le_bytes(x[..8].try_into().unwrap()),
                    offset: u32::from_le_bytes(x[8..12].try_into().unwrap()),
                    len: x[12],
                })
                .collect())
        })
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Model {
    pub(crate) outcome_labels: Vec<String>,
    #[serde(with = "raw_bytes")]
    pub(crate) outcomes: Vec<u8>,
    #[serde(with = "float_bytes")]
    pub(crate) parameters: Vec<f32>,
    features: FeatureMap,
}
impl Model {
    #[allow(dead_code)] // Build-only asset constructor.
    pub(crate) fn from_parts(
        outcome_labels: Vec<String>,
        entries: Vec<(u64, u8)>,
        outcomes: Vec<u8>,
        parameters: Vec<f32>,
    ) -> Self {
        Self {
            outcome_labels,
            outcomes,
            parameters,
            features: FeatureMap::new(entries),
        }
    }
    fn eval(&self, context: &[u64]) -> Vec<f32> {
        let mut p =
            vec![(1.0f32 / self.outcome_labels.len() as f32).ln(); self.outcome_labels.len()];
        for key in context {
            if let Some((start, len)) = self.features.get(*key) {
                for i in start..start + len {
                    p[self.outcomes[i] as usize] += self.parameters[i];
                }
            }
        }
        let mut sum = 0.0f32;
        for x in &mut p {
            *x = x.exp();
            sum += *x;
        }
        for x in &mut p {
            *x /= sum;
        }
        p
    }
}
fn top(p: &[f32], n: usize, threshold: f32) -> Vec<usize> {
    let mut ids: Vec<_> = (0..p.len()).filter(|&i| p[i] >= threshold).collect();
    ids.sort_by(|&a, &b| p[b].partial_cmp(&p[a]).unwrap_or(Ordering::Equal));
    ids.truncate(n);
    ids
}
#[derive(Clone, Debug)]
struct Sequence<'m> {
    labels: Vec<&'m str>,
    probabilities: Vec<f32>,
    score: f32,
}
impl<'m> Sequence<'m> {
    fn empty() -> Self {
        Self {
            labels: vec![],
            probabilities: vec![],
            score: 0.0,
        }
    }
    fn extend(&self, label: &'m str, p: f32) -> Self {
        let mut labels = Vec::with_capacity(self.labels.len() + 1);
        labels.extend_from_slice(&self.labels);
        labels.push(label);
        let mut probabilities = Vec::with_capacity(self.probabilities.len() + 1);
        probabilities.extend_from_slice(&self.probabilities);
        probabilities.push(p);
        let score = probabilities.iter().fold(0.0f32, |a, x| a + x.ln());
        Self {
            labels,
            probabilities,
            score,
        }
    }
}
impl PartialEq for Sequence<'_> {
    fn eq(&self, b: &Self) -> bool {
        self.score == b.score
    }
}
impl Eq for Sequence<'_> {}
impl PartialOrd for Sequence<'_> {
    fn partial_cmp(&self, b: &Self) -> Option<Ordering> {
        Some(self.cmp(b))
    }
}
impl Ord for Sequence<'_> {
    fn cmp(&self, b: &Self) -> Ordering {
        b.score.partial_cmp(&self.score).unwrap_or(Ordering::Equal)
    }
}
fn history_hash<P: AsRef<str>>(preds: &[P], i: usize) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::default();
    for p in preds.iter().rev().take(2) {
        p.as_ref().hash(&mut h)
    }
    i.hash(&mut h);
    h.finish()
}
fn beam<'m, C, V, H>(
    model: &'m Model,
    len: usize,
    size: usize,
    mut context: C,
    valid: V,
    key: H,
) -> Vec<Sequence<'m>>
where
    C: FnMut(usize, &[&str]) -> Vec<u64>,
    V: Fn(usize, &[&str], &str) -> bool,
    H: Fn(usize, &[&str]) -> u64,
{
    let mut heap = BinaryHeap::new();
    heap.push(Sequence::empty());
    let mut cache = HashMap::new();
    let mut next = BinaryHeap::new();
    for i in 0..len {
        while heap.len() > size {
            heap.pop();
        }
        next.clear();
        for s in &heap {
            let p = cache
                .entry(key(i, &s.labels))
                .or_insert_with(|| model.eval(&context(i, &s.labels)));
            for id in top(p, size, 0.1) {
                let label = &model.outcome_labels[id];
                if valid(i, &s.labels, label) {
                    next.push(s.extend(label, p[id]));
                }
            }
            if next.is_empty() {
                for (id, label) in model.outcome_labels.iter().enumerate() {
                    if valid(i, &s.labels, label) {
                        next.push(s.extend(label, p[id]));
                    }
                }
            }
        }
        heap.clear();
        std::mem::swap(&mut heap, &mut next);
    }
    let mut out: Vec<_> = heap.drain().collect();
    out.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
    out
}
fn hashes(v: Vec<String>) -> Vec<u64> {
    v.iter().map(|s| hash::hash_str(s)).collect()
}
fn char_features(v: &mut Vec<String>, k: &str, c: char) {
    v.push(format!("{k}={c}"));
    let class = if c.is_alphabetic() {
        Some("alpha")
    } else if c.is_ascii_digit() {
        Some("num")
    } else if c.is_whitespace() {
        Some("ws")
    } else if ".?!".contains(c) {
        Some("eos")
    } else if "`\"'".contains(c) {
        Some("quote")
    } else if "[{(".contains(c) {
        Some("lp")
    } else if "]})".contains(c) {
        Some("rp")
    } else {
        None
    };
    if let Some(class) = class {
        v.push(format!("{k}_{class}"));
        if class == "alpha" && c.is_uppercase() {
            v.push(format!("{k}_caps"));
        }
    }
}
fn tok_context(text: &str, i: usize) -> Vec<u64> {
    let c: Vec<_> = text.chars().collect();
    let mut v = vec![
        format!("p={}", c[..i].iter().collect::<String>()),
        format!("s={}", c[i..].iter().collect::<String>()),
    ];
    if i > 0 {
        char_features(&mut v, "p1", c[i - 1]);
        if i > 1 {
            char_features(&mut v, "p2", c[i - 2]);
            v.push(format!("p21={}{}", c[i - 2], c[i - 1]));
        } else {
            v.push("p2=bok".into());
        }
        v.push(format!("p1f1={}{}", c[i - 1], c[i]));
    } else {
        v.push("b1=bok".into());
    }
    char_features(&mut v, "f1", c[i]);
    if i + 1 < c.len() {
        char_features(&mut v, "f2", c[i + 1]);
        v.push(format!("f12={}{}", c[i], c[i + 1]));
    } else {
        v.push("f2=bok".into());
    }
    if text.starts_with('&') && text.ends_with(';') {
        v.push("cc".into());
    }
    hashes(v)
}
fn pos_context<T: AsRef<str>, P: AsRef<str>>(tokens: &[T], preds: &[P], i: usize) -> Vec<u64> {
    let w = tokens[i].as_ref();
    let mut v = Vec::with_capacity(20);
    v.push(hash::hash_str("default"));
    v.push(hash::hash_slice(&["w=", w]));
    let mut suffix = w.len();
    for _ in 0..4 {
        if suffix > 0 {
            suffix = w[..suffix].char_indices().next_back().unwrap().0;
        }
        v.push(hash::hash_slice(&["suf=", &w[suffix..]]));
    }
    let mut prefix = 0;
    for _ in 0..4 {
        if prefix < w.len() {
            prefix += w[prefix..].chars().next().unwrap().len_utf8();
        }
        v.push(hash::hash_slice(&["pre=", &w[..prefix]]));
    }
    if w.contains('-') {
        v.push(hash::hash_str("h"));
    }
    if w.bytes().any(|c| c.is_ascii_uppercase()) {
        v.push(hash::hash_str("c"));
    }
    if w.bytes().any(|c| c.is_ascii_digit()) {
        v.push(hash::hash_str("d"));
    }
    let p = if i > 0 {
        tokens[i - 1].as_ref()
    } else {
        "*SB*"
    };
    v.push(hash::hash_slice(&["p=", p]));
    if p != "*SB*" {
        v.push(hash::hash_slice(&[
            "pp=",
            if i > 1 {
                tokens[i - 2].as_ref()
            } else {
                "*SB*"
            },
        ]));
    }
    let n = tokens.get(i + 1).map(AsRef::as_ref).unwrap_or("*SE*");
    v.push(hash::hash_slice(&["n=", n]));
    if n != "*SE*" {
        v.push(hash::hash_slice(&[
            "nn=",
            tokens.get(i + 2).map(AsRef::as_ref).unwrap_or("*SE*"),
        ]));
    }
    if let Some(t) = preds.last() {
        let t = t.as_ref();
        v.push(hash::hash_slice(&["t=", t]));
        if preds.len() > 1 {
            v.push(hash::hash_slice(&[
                "t2=",
                preds[preds.len() - 2].as_ref(),
                ",",
                t,
            ]));
        }
    }
    v
}
fn chunk_context<T: AsRef<str>, P: AsRef<str>>(
    input: &[(T, T)],
    preds: &[P],
    i: usize,
) -> Vec<u64> {
    let names = ["_2", "_1", "0", "1", "2"];
    let mut w = Vec::with_capacity(5);
    let mut t = Vec::with_capacity(5);
    for (j, name) in names.iter().enumerate() {
        let index = i as isize + j as isize - 2;
        let (word, tag) = if index < 0 {
            ("bos", "bos")
        } else if index as usize >= input.len() {
            ("eos", "eos")
        } else {
            (
                input[index as usize].0.as_ref(),
                input[index as usize].1.as_ref(),
            )
        };
        w.push(format!("w{name}={word}"));
        t.push(format!("t{name}={tag}"));
    }
    let p2 = if preds.len() > 1 {
        format!("p_2{}", preds[preds.len() - 2].as_ref())
    } else {
        "p_2=bos".into()
    };
    let p1 = format!("p_1={}", preds.last().map(AsRef::as_ref).unwrap_or("bos"));
    let mut out = Vec::with_capacity(41);
    let mut emit = |prefix: Option<&str>,
                    features: &[String],
                    width: usize,
                    starts: std::ops::Range<usize>| {
        for start in starts {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            if let Some(prefix) = prefix {
                h.write(prefix.as_bytes());
            }
            for f in &features[start..start + width] {
                h.write(f.as_bytes());
            }
            h.write_u8(255);
            out.push(h.finish());
        }
    };
    emit(None, &w, 1, 0..5);
    emit(None, &w, 2, 1..3);
    emit(None, &t, 1, 0..5);
    emit(None, &t, 2, 0..4);
    emit(None, &t, 3, 0..3);
    // End the closure borrow before inserting history features.
    out.extend([
        hash::hash_str(&p2),
        hash::hash_str(&p1),
        hash::hash_slice(&[&p2, &p1]),
    ]);
    for (features, width, starts) in [
        (&t, 1, 0..5),
        (&t, 2, 0..4),
        (&t, 3, 0..3),
        (&w, 1, 0..5),
        (&w, 2, 1..3),
    ] {
        for start in starts {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            h.write(p1.as_bytes());
            for f in &features[start..start + width] {
                h.write(f.as_bytes());
            }
            h.write_u8(255);
            out.push(h.finish());
        }
    }
    out
}
fn chunk_valid<P: AsRef<str>>(preds: &[P], label: &str) -> bool {
    if !label.starts_with("I-") {
        return true;
    }
    preds.last().is_some_and(|p| {
        let p = p.as_ref();
        p == "O" || p[2..] == label[2..]
    })
}
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct MaxentTokenizer {
    pub(crate) model: Model,
}
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct MaxentPosTagger {
    pub(crate) model: Model,
    #[serde(serialize_with = "sorted_dictionary")]
    pub(crate) tagdict: crate::nlp::annotation::types::DefaultHashMap<String, Vec<String>>,
}
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct MaxentChunker {
    pub(crate) model: Model,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Chunker {
    pub(crate) token_model: MaxentTokenizer,
    pub(crate) pos_model: MaxentPosTagger,
    pub(crate) chunk_model: MaxentChunker,
}
impl MaxentTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<&'a str> {
        let mut out = vec![];
        for word in text.split_whitespace() {
            if word.chars().count() < 2 || word.bytes().all(|c| c.is_ascii_alphanumeric()) {
                out.push(word);
                continue;
            }
            let mut start = 0;
            for (i, (byte, _)) in word.char_indices().enumerate().skip(1) {
                let p = self.model.eval(&tok_context(word, i));
                let id = top(&p, 1, 0.5)[0];
                if self.model.outcome_labels[id] == "T" {
                    out.push(&word[start..byte]);
                    start = byte;
                }
            }
            out.push(&word[start..]);
        }
        out
    }
}
impl MaxentPosTagger {
    fn tag<T: AsRef<str>>(&self, tokens: &[T]) -> Sequence<'_> {
        beam(
            &self.model,
            tokens.len(),
            3,
            |i, p| pos_context(tokens, p, i),
            |i, _, l| {
                self.tagdict
                    .get(tokens[i].as_ref())
                    .is_none_or(|v| v.iter().any(|x| x == l))
            },
            |i, p| history_hash(p, i),
        )
        .into_iter()
        .next()
        .unwrap()
    }
}
impl MaxentChunker {
    fn chunk<T: AsRef<str>>(&self, input: &[(T, T)]) -> Sequence<'_> {
        beam(
            &self.model,
            input.len(),
            7,
            |i, p| chunk_context(input, p, i),
            |_, p, l| chunk_valid(p, l),
            |i, p| history_hash(p, i),
        )
        .into_iter()
        .next()
        .unwrap()
    }
}
impl Chunker {
    pub fn apply(&self, sentence: &mut crate::nlp::annotation::types::IncompleteSentence<'_>) {
        let text = sentence.text().replace('\u{2019}', "'");
        let tokens = self.token_model.tokenize(&text);
        let tags = self.pos_model.tag(&tokens);
        let input: Vec<_> = tokens.iter().copied().zip(tags.labels).collect();
        let labels = self.chunk_model.chunk(&input).labels;
        let mut cursor = 0;
        let mut scalar_cursor = 0;
        let spans: Vec<_> = tokens
            .iter()
            .map(|token| {
                let offset = text[cursor..].find(token).unwrap() + cursor;
                let start = scalar_cursor + text[cursor..offset].chars().count();
                cursor = offset + token.len();
                scalar_cursor = start + token.chars().count();
                start..scalar_cursor
            })
            .collect();
        let mut number = "singular";
        for (i, &label) in labels.iter().enumerate() {
            if label == "B-NP" {
                number = "singular";
                for j in i..labels.len() {
                    if labels[j] != "B-NP" && labels[j] != "I-NP" {
                        break;
                    }
                    if sentence.iter().any(|t| {
                        t.span().char() == &spans[j]
                            && t.word().tags().iter().any(|d| d.pos().as_str() == "NNS")
                    }) {
                        number = "plural";
                    }
                }
            }
            let ending = i + 1 == labels.len() || labels[i + 1] != "I-NP";
            let chunks = if label == "B-NP" {
                let mut c = vec![format!("B-NP-{number}")];
                if ending {
                    c.push(format!("E-NP-{number}"));
                }
                c
            } else if label == "I-NP" {
                vec![format!("{}-NP-{number}", if ending { "E" } else { "I" })]
            } else {
                vec![label.to_string()]
            };
            for token in sentence.iter_mut() {
                if token.span().char() == &spans[i] {
                    *token.chunks_mut() = chunks.clone();
                }
            }
        }
    }
}

fn sorted_dictionary<S: serde::Serializer>(
    map: &crate::nlp::annotation::types::DefaultHashMap<String, Vec<String>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    map.iter()
        .collect::<std::collections::BTreeMap<_, _>>()
        .serialize(serializer)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packed_parameters_preserve_payloads_and_reject_partial_elements() {
        let bits = [0, 0x80000000, 0x7fc01234, 0xffc05678];
        let m = Model::from_parts(vec![], vec![], vec![], bits.map(f32::from_bits).to_vec());
        let bytes = bincode::serialize(&m).unwrap();
        let decoded: Model = bincode::deserialize(&bytes).unwrap();
        assert_eq!(
            decoded
                .parameters
                .iter()
                .map(|p| p.to_bits())
                .collect::<Vec<_>>(),
            bits
        );
        #[derive(Deserialize)]
        struct Packed(#[serde(with = "float_bytes")] Vec<f32>);
        assert!(
            bincode::deserialize::<Packed>(&bincode::serialize(&vec![0u8; 3]).unwrap()).is_err()
        );
        let Packed(values) =
            bincode::deserialize(&bincode::serialize(&vec![0u8; 4]).unwrap()).unwrap();
        assert_eq!(values[0].to_bits(), 0);
    }
    #[test]
    fn tied_beam_entries_keep_heap_order() {
        let mut heap = BinaryHeap::new();
        for label in ["a", "b", "c"] {
            heap.push(Sequence::empty().extend(label, 0.5));
        }
        assert_eq!(
            (0..3)
                .map(|_| heap.pop().unwrap().labels[0])
                .collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
    }
}
