// Portions derived from nlprule 0.6.4. See src/nlp/LICENSE.
//! A Chunker ported from [OpenNLP](https://opennlp.apache.org/).

use half::bf16;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::{cmp::Ordering, collections::BinaryHeap};

use crate::nlp::types::{DefaultHashMap, DefaultHasher, IncompleteSentence};

fn softmax(vec: &mut [f32]) {
    for x in vec.iter_mut() {
        *x = x.exp();
    }

    let sum = vec.iter().fold(0., |a, b| a + b);
    for x in vec.iter_mut() {
        *x /= sum;
    }
}

#[derive(Debug, Clone)]
struct Sequence<'a> {
    outcomes: Vec<&'a str>,
    probs: Vec<f32>,
    log_prob: f32,
}

impl<'a> Eq for Sequence<'a> {}

impl<'a> PartialEq for Sequence<'a> {
    fn eq(&self, other: &Self) -> bool {
        other.outcomes == self.outcomes
    }
}

impl<'a> Ord for Sequence<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

// Retain the pinned model's partial comparison, including NaN behavior.
#[allow(clippy::non_canonical_partial_ord_impl)]
impl<'a> PartialOrd for Sequence<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.log_prob.partial_cmp(&self.log_prob)
    }
}

impl<'a> Default for Sequence<'a> {
    fn default() -> Self {
        Sequence {
            outcomes: Vec::new(),
            probs: Vec::new(),
            log_prob: 0.,
        }
    }
}

impl<'a> Sequence<'a> {
    fn new(outcomes: Vec<&'a str>, probs: Vec<f32>) -> Self {
        let log_prob = probs.iter().fold(0., |a, b| a + b.ln());
        Sequence {
            outcomes,
            probs,
            log_prob,
        }
    }

    #[inline]
    fn outcomes(&self) -> &[&'a str] {
        &self.outcomes
    }

    #[inline]
    pub fn probs(&self) -> &[f32] {
        &self.probs
    }
}

pub(crate) mod hash {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    pub fn hash_str(string: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        string.hash(&mut hasher);
        hasher.finish()
    }

    pub fn hash_slice(slice: &[&str]) -> u64 {
        let mut hasher = DefaultHasher::new();
        for s in slice.iter() {
            hasher.write(s.as_bytes());
        }
        // This is intentional.
        // See https://doc.rust-lang.org/stable/src/core/hash/mod.rs.html#598-603
        "".hash(&mut hasher);
        hasher.finish()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn same_hash_if_same_join() {
            let arr = &["ab", "cde", "fg"];
            let string = "abcdefg";

            assert_eq!(hash_slice(arr), hash_str(string));
        }
    }
}

#[derive(Deserialize)]
struct ModelFields {
    outcome_labels: Vec<String>,
    pmap: Vec<(u64, u8)>,
    outcomes: Vec<u8>,
    parameters: Vec<bf16>,
}

// Feature keys are already hashed. Build immutable linear-probe slots once,
// rather than hashing and inserting every feature during each CLI invocation.
#[derive(Serialize, Deserialize, Clone)]
struct FeatureSlot {
    key: u64,
    offset: u32,
    length: u8,
}

#[derive(Serialize, Deserialize, Clone)]
struct FeatureMap {
    #[serde(with = "packed_slots")]
    slots: Vec<FeatureSlot>,
}

impl FeatureMap {
    fn new(entries: Vec<(u64, u8)>) -> Self {
        let capacity = entries.len().saturating_mul(2).max(1).next_power_of_two();
        let mut slots = vec![
            FeatureSlot {
                key: 0,
                offset: u32::MAX,
                length: 0
            };
            capacity
        ];
        let mut offset = 0;
        for (key, length) in entries {
            let mut index = key as usize & (capacity - 1);
            while slots[index].offset != u32::MAX && slots[index].key != key {
                index = (index + 1) & (capacity - 1);
            }
            slots[index] = FeatureSlot {
                key,
                offset: u32::try_from(offset).expect("feature parameter offset"),
                length,
            };
            offset += usize::from(length);
        }
        Self { slots }
    }

    fn get(&self, key: &u64) -> Option<(usize, usize)> {
        let mask = self.slots.len() - 1;
        let mut index = *key as usize & mask;
        loop {
            let slot = &self.slots[index];
            if slot.offset == u32::MAX {
                return None;
            }
            if slot.key == *key {
                return Some((slot.offset as usize, usize::from(slot.length)));
            }
            index = (index + 1) & mask;
        }
    }
}

#[cfg(test)]
mod feature_tests {
    use super::*;

    #[test]
    fn immutable_slots_preserve_collisions_empty_features_and_replacements() {
        let entries: Vec<_> = (0..1000u64)
            .map(|i| (i * 4096, (i % 13) as u8))
            .chain([(0, 3), (u64::MAX, 0), (4096, 5)])
            .collect();
        let mut expected = DefaultHashMap::new();
        let mut offset = 0;
        for &(key, length) in &entries {
            expected.insert(key, (offset, usize::from(length)));
            offset += usize::from(length);
        }
        let map = FeatureMap::new(entries);
        let bytes = bincode::serialize(&map).unwrap();
        let loaded: FeatureMap = bincode::deserialize(&bytes).unwrap();
        for key in expected.keys().copied().chain([1, 4097, u64::MAX - 1]) {
            assert_eq!(loaded.get(&key), expected.get(&key).copied());
        }
        assert_eq!(FeatureMap::new(vec![]).get(&0), None);
    }
}

impl From<ModelFields> for Model {
    fn from(data: ModelFields) -> Self {
        let pmap = FeatureMap::new(data.pmap);

        let outcomes = data.outcomes;
        let parameters = data
            .parameters
            .into_iter()
            .map(|parameter| parameter.to_f32())
            .collect();

        Model {
            outcome_labels: data.outcome_labels,
            pmap,
            outcomes,
            parameters,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Model {
    pub(crate) outcome_labels: Vec<String>,
    #[serde(with = "packed_outcomes")]
    pub(crate) outcomes: Vec<u8>,
    #[serde(with = "packed_parameters")]
    pub(crate) parameters: Vec<f32>,
    pmap: FeatureMap,
}

impl Model {
    fn eval(&self, context: &[u64]) -> Vec<f32> {
        let mut prior =
            vec![(1. / (self.outcome_labels.len() as f32)).ln(); self.outcome_labels.len()];

        for (offset, length) in context.iter().filter_map(|x| self.pmap.get(x)) {
            let outcomes = &self.outcomes[offset..offset + length];
            let parameters = &self.parameters[offset..offset + length];

            for (idx, param) in outcomes.iter().zip(parameters.iter()) {
                prior[usize::from(*idx)] += param;
            }
        }

        softmax(&mut prior);
        prior
    }

    fn get_top_n(&self, probs: &[f32], n: usize, threshold: f32) -> Vec<(usize, f32, &str)> {
        let mut probs: Vec<_> = probs
            .iter()
            .enumerate()
            .filter(|(_, x)| **x >= threshold) // this is slightly different than LT behaviour but deemed okay as long as all tests pass
            .collect();
        probs.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

        probs
            .into_iter()
            .map(|(i, p)| (i, *p, self.outcome_labels[i].as_str()))
            .take(n)
            .collect::<Vec<_>>()
    }

    fn beam_search<
        S,
        C: Fn(&[S], &[&str], usize) -> Vec<u64>,
        H: Fn(&[&str], usize) -> u64,
        V: Fn(&[S], &[&str], usize, &str) -> bool,
    >(
        &self,
        tokens: &[S],
        context_fn: C,
        hash_fn: H,
        valid_fn: V,
        size: usize,
    ) -> Vec<Sequence<'_>> {
        let mut prev: BinaryHeap<Sequence> = BinaryHeap::new();
        let mut next: BinaryHeap<Sequence> = BinaryHeap::new();
        prev.push(Sequence::default());

        let mut cache: DefaultHashMap<u64, Vec<f32>> = DefaultHashMap::default();

        for i in 0..tokens.len() {
            while prev.len() > size {
                prev.pop();
            }

            for seq in prev.iter() {
                let hash = hash_fn(seq.outcomes(), i);
                cache.entry(hash).or_insert_with(|| {
                    let context = context_fn(tokens, seq.outcomes(), i);
                    self.eval(&context)
                });
                let scores = cache.get(&hash).unwrap();
                let top_n = self.get_top_n(scores, size, 1e-1);

                for (_, p, pred) in top_n {
                    if valid_fn(tokens, seq.outcomes(), i, pred) {
                        let next_outcomes: Vec<_> = [seq.outcomes(), &[pred]].concat();
                        let next_probs: Vec<_> = [seq.probs(), &[p]].concat();

                        next.push(Sequence::new(next_outcomes, next_probs));
                    }
                }

                // if no advanced sequences, advance all valid to match OpenNLP behaviour
                if next.is_empty() {
                    for (j, p) in scores.iter().enumerate() {
                        let pred = self.outcome_labels[j].as_str();
                        if valid_fn(tokens, seq.outcomes(), i, pred) {
                            let mut next_outcomes: Vec<_> = seq.outcomes().to_vec();
                            next_outcomes.push(pred);

                            let mut next_probs: Vec<_> = seq.probs().to_vec();
                            next_probs.push(*p);

                            next.push(Sequence::new(next_outcomes, next_probs));
                        }
                    }
                }
            }

            prev = next.clone();
            next.clear();
        }

        let mut out: Vec<_> = prev.drain().collect();
        out.sort();
        out
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct MaxentTokenizer {
    pub(crate) model: Model,
}

impl MaxentTokenizer {
    fn add_char_context(key: &str, c: char, context: &mut Vec<u64>) {
        macro_rules! add {
            ($x: expr) => {
                context.push(hash::hash_slice(&[key, $x]));
            };
        }

        context.push(hash::hash_slice(&[key, "=", &String::from(c)]));

        if c.is_alphabetic() {
            add!("_alpha");
            if c.is_uppercase() {
                add!("_caps");
            }
        } else if c.is_ascii_digit() {
            add!("_num");
        } else if c.is_whitespace() {
            add!("_ws");
        } else if ['.', '?', '!'].contains(&c) {
            add!("_eos");
        } else if ['`', '"', '\''].contains(&c) {
            add!("_quote");
        } else if ['[', '{', '('].contains(&c) {
            add!("_lp");
        } else if [']', '}', ')'].contains(&c) {
            add!("_rp");
        }
    }

    fn context(chars: &[char], i: usize) -> Vec<u64> {
        let mut context = Vec::new();

        let prefix: String = chars[..i].iter().collect();
        let suffix: String = chars[i..].iter().collect();

        context.push(hash::hash_slice(&["p=", &prefix]));
        context.push(hash::hash_slice(&["s=", &suffix]));

        if i > 0 {
            Self::add_char_context("p1", chars[i - 1], &mut context);
            if i > 1 {
                Self::add_char_context("p2", chars[i - 2], &mut context);
                context.push(hash::hash_slice(&[
                    "p21=",
                    &String::from(chars[i - 2]),
                    &String::from(chars[i - 1]),
                ]));
            } else {
                context.push(hash::hash_str("p2=bok"));
            }
            context.push(hash::hash_slice(&[
                "p1f1=",
                &String::from(chars[i - 1]),
                &String::from(chars[i]),
            ]));
        } else {
            context.push(hash::hash_str("b1=bok"));
        }

        Self::add_char_context("f1", chars[i], &mut context);
        if i + 1 < chars.len() {
            Self::add_char_context("f2", chars[i + 1], &mut context);
            context.push(hash::hash_slice(&[
                "f12=",
                &String::from(chars[i]),
                &String::from(chars[i + 1]),
            ]));
        } else {
            context.push(hash::hash_str("f2=bok"));
        }

        if chars[0] == '&' && chars[chars.len() - 1] == ';' {
            context.push(hash::hash_str("cc")); // character code
        }

        // TODO: add abbreviations
        if i == chars.len() - 1 && [].contains(&chars) {
            context.push(hash::hash_str("pabb"));
        }

        context
    }

    fn tokenize<'a>(&self, text: &'a str) -> Vec<&'a str> {
        let mut tokens = Vec::new();
        let char_indices: Vec<_> = text.char_indices().collect();

        for pre_token in text.split_whitespace() {
            let byte_offset = pre_token.as_ptr() as usize - text.as_ptr() as usize;
            let char_start = char_indices
                .iter()
                .position(|(i, _)| *i == byte_offset)
                .unwrap();
            let byte_start = char_indices[char_start].0;

            let token_char_indices =
                &char_indices[char_start..char_start + pre_token.chars().count()];
            let token_chars: Vec<_> = token_char_indices.iter().map(|(_, c)| c).cloned().collect();

            if (token_char_indices.len() < 2)
                || pre_token.chars().all(|x| x.is_ascii_alphanumeric())
            {
                tokens.push(pre_token);
            } else {
                let mut start = byte_start;

                for i in 1..token_char_indices.len() {
                    let context = Self::context(&token_chars, i);
                    let (_, _, best) = self.model.get_top_n(&self.model.eval(&context), 1, 0.5)[0];

                    if best == "T" {
                        tokens.push(&text[start..token_char_indices[i].0]);
                        start = token_char_indices[i].0;
                    }
                }

                tokens.push(&text[start..byte_start + pre_token.len()]);
            }
        }

        tokens
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct MaxentPosTagger {
    pub(crate) model: Model,
    #[serde(serialize_with = "crate::nlp::utils::asset::map")]
    pub(crate) tagdict: DefaultHashMap<String, Vec<String>>,
}

impl MaxentPosTagger {
    fn get_suffixes_prefixes(string: &str) -> Vec<u64> {
        let chars: Vec<_> = string.chars().collect();
        let mut output = Vec::new();

        for i in 0..4 {
            output.push(hash::hash_slice(&[
                "suf=",
                &chars[std::cmp::max((chars.len() as isize) - 1 - i, 0) as usize..]
                    .iter()
                    .collect::<String>(),
            ]));
        }

        for i in 0..4 {
            output.push(hash::hash_slice(&[
                "pre=",
                &chars[..std::cmp::min(i + 1, chars.len())]
                    .iter()
                    .collect::<String>(),
            ]));
        }

        output
    }

    fn hash(tags: &[&str], i: usize) -> u64 {
        let mut s = DefaultHasher::default();
        if i >= 1 {
            tags[i - 1].hash(&mut s);
        }
        if i >= 2 {
            tags[i - 2].hash(&mut s);
        }
        i.hash(&mut s);
        s.finish()
    }

    fn context(tokens: &[&str], tags: &[&str], i: usize) -> Vec<u64> {
        let mut context = Vec::new();

        let lex = tokens[i];
        let next = if i + 1 < tokens.len() {
            tokens[i + 1]
        } else {
            "*SE*"
        };
        let nextnext = if i + 2 < tokens.len() {
            tokens[i + 2]
        } else {
            "*SE*"
        };
        let (prev, tagprev) = if i >= 1 {
            (tokens[i - 1], Some(tags[i - 1]))
        } else {
            ("*SB*", None)
        };
        let (prevprev, tagprevprev) = if i >= 2 {
            (tokens[i - 2], Some(tags[i - 2]))
        } else {
            ("*SB*", None)
        };

        context.push(hash::hash_str("default"));
        context.push(hash::hash_slice(&["w=", lex]));

        context.extend(Self::get_suffixes_prefixes(lex));

        if lex.contains('-') {
            context.push(hash::hash_str("h"));
        }
        if lex.chars().any(|c| c.is_ascii_uppercase()) {
            context.push(hash::hash_str("c"));
        }
        if lex.chars().any(|c| c.is_ascii_digit()) {
            context.push(hash::hash_str("d"));
        }

        context.push(hash::hash_slice(&["p=", prev]));
        if prev != "*SB*" {
            context.push(hash::hash_slice(&["pp=", prevprev]));
        }

        context.push(hash::hash_slice(&["n=", next]));
        if next != "*SE*" {
            context.push(hash::hash_slice(&["nn=", nextnext]));
        }

        if let Some(tagprev) = tagprev {
            context.push(hash::hash_slice(&["t=", tagprev]));

            if let Some(tagprevprev) = tagprevprev {
                context.push(hash::hash_slice(&["t2=", tagprevprev, ",", tagprev]));
            }
        }

        context
    }

    fn valid(&self, tokens: &[&str], _outcomes: &[&str], i: usize, outcome: &str) -> bool {
        if let Some(allowed) = self.tagdict.get(tokens[i]) {
            allowed.contains(&outcome.to_string())
        } else {
            true
        }
    }

    fn tag(&self, tokens: &[&str]) -> Sequence<'_> {
        self.model
            .beam_search(
                tokens,
                Self::context,
                Self::hash,
                |a, b, c, d| self.valid(a, b, c, d),
                3,
            )
            .remove(0)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct MaxentChunker {
    pub(crate) model: Model,
}

impl MaxentChunker {
    #[allow(clippy::manual_strip)]
    fn valid(&self, _input: &[(&str, &str)], outcomes: &[&str], _i: usize, outcome: &str) -> bool {
        if outcome.starts_with("I-") {
            if let Some(prev_outcome) = outcomes.iter().next_back() {
                // can byte index here because "I, B, E, -" are all 1 byte
                *prev_outcome == "O" || prev_outcome[2..] == outcome[2..]
            } else {
                false
            }
        } else {
            true
        }
    }

    fn hash(preds: &[&str], i: usize) -> u64 {
        let mut s = DefaultHasher::default();
        if i >= 1 {
            preds[i - 1].hash(&mut s);
        }
        if i >= 2 {
            preds[i - 2].hash(&mut s);
        }
        i.hash(&mut s);
        s.finish()
    }

    fn context(input: &[(&str, &str)], preds: &[&str], i: usize) -> Vec<u64> {
        let (tokens, tags): (Vec<&str>, Vec<&str>) = input.iter().cloned().unzip();

        let (w_2, t_2, p_2) = if i < 2 {
            ("w_2=bos".into(), "t_2=bos".into(), "p_2=bos".into())
        } else {
            (
                format!("w_2={}", tokens[i - 2]),
                format!("t_2={}", tags[i - 2]),
                format!("p_2{}", preds[i - 2]),
            )
        };

        let (w_1, t_1, p_1) = if i < 1 {
            ("w_1=bos".into(), "t_1=bos".into(), "p_1=bos".into())
        } else {
            (
                format!("w_1={}", tokens[i - 1]),
                format!("t_1={}", tags[i - 1]),
                format!("p_1={}", preds[i - 1]),
            )
        };

        let w0 = format!("w0={}", tokens[i]);
        let t0 = format!("t0={}", tags[i]);

        let (w1, t1) = if i + 1 >= tokens.len() {
            ("w1=eos".into(), "t1=eos".into())
        } else {
            (
                format!("w1={}", tokens[i + 1]),
                format!("t1={}", tags[i + 1]),
            )
        };

        let (w2, t2) = if i + 2 >= tokens.len() {
            ("w2=eos".into(), "t2=eos".into())
        } else {
            (
                format!("w2={}", tokens[i + 2]),
                format!("t2={}", tags[i + 2]),
            )
        };

        vec![
            // add word features
            hash::hash_str(&w_2),
            hash::hash_str(&w_1),
            hash::hash_str(&w0),
            hash::hash_str(&w1),
            hash::hash_str(&w2),
            hash::hash_slice(&[&w_1, &w0]),
            hash::hash_slice(&[&w0, &w1]),
            // add tag features
            hash::hash_str(&t_2),
            hash::hash_str(&t_1),
            hash::hash_str(&t0),
            hash::hash_str(&t1),
            hash::hash_str(&t2),
            hash::hash_slice(&[&t_2, &t_1]),
            hash::hash_slice(&[&t_1, &t0]),
            hash::hash_slice(&[&t0, &t1]),
            hash::hash_slice(&[&t1, &t2]),
            hash::hash_slice(&[&t_2, &t_1, &t0]),
            hash::hash_slice(&[&t_1, &t0, &t1]),
            hash::hash_slice(&[&t0, &t1, &t2]),
            // add pred tags
            hash::hash_str(&p_2),
            hash::hash_str(&p_1),
            hash::hash_slice(&[&p_2, &p_1]),
            // add pred and tag
            hash::hash_slice(&[&p_1, &t_2]),
            hash::hash_slice(&[&p_1, &t_1]),
            hash::hash_slice(&[&p_1, &t0]),
            hash::hash_slice(&[&p_1, &t1]),
            hash::hash_slice(&[&p_1, &t2]),
            hash::hash_slice(&[&p_1, &t_2, &t_1]),
            hash::hash_slice(&[&p_1, &t_1, &t0]),
            hash::hash_slice(&[&p_1, &t0, &t1]),
            hash::hash_slice(&[&p_1, &t1, &t2]),
            hash::hash_slice(&[&p_1, &t_2, &t_1, &t0]),
            hash::hash_slice(&[&p_1, &t_1, &t0, &t1]),
            hash::hash_slice(&[&p_1, &t0, &t1, &t2]),
            // add pred and word
            hash::hash_slice(&[&p_1, &w_2]),
            hash::hash_slice(&[&p_1, &w_1]),
            hash::hash_slice(&[&p_1, &w0]),
            hash::hash_slice(&[&p_1, &w1]),
            hash::hash_slice(&[&p_1, &w2]),
            hash::hash_slice(&[&p_1, &w_1, &w0]),
            hash::hash_slice(&[&p_1, &w0, &w1]),
        ]
    }

    fn chunk(&self, input: &[(&str, &str)]) -> Sequence<'_> {
        self.model
            .beam_search(
                input,
                Self::context,
                Self::hash,
                |a, b, c, d| self.valid(a, b, c, d),
                7,
            )
            .remove(0)
    }
}

/// Predicts noun chunks and verb chunks through a [Maximum Entropy Model](https://www.aclweb.org/anthology/W00-0729.pdf).
/// Grammatical number (i. e. singular and plural) is also assigned through the part-of-speech tags of the tokens.
#[derive(Serialize, Deserialize, Clone)]
pub struct Chunker {
    pub(crate) token_model: MaxentTokenizer,
    pub(crate) pos_model: MaxentPosTagger,
    pub(crate) chunk_model: MaxentChunker,
}

#[derive(Deserialize)]
struct WireSingleModel {
    model: ModelFields,
}

#[derive(Deserialize)]
struct WirePosModel {
    model: ModelFields,
    tagdict: DefaultHashMap<String, Vec<String>>,
}

/// Original resource layout, decoded only by the asset compiler.
#[derive(Deserialize)]
pub(super) struct WireChunker {
    token_model: WireSingleModel,
    pos_model: WirePosModel,
    chunk_model: WireSingleModel,
}

impl From<WireChunker> for Chunker {
    fn from(wire: WireChunker) -> Self {
        Self {
            token_model: MaxentTokenizer {
                model: wire.token_model.model.into(),
            },
            pos_model: MaxentPosTagger {
                model: wire.pos_model.model.into(),
                tagdict: wire.pos_model.tagdict,
            },
            chunk_model: MaxentChunker {
                model: wire.chunk_model.model.into(),
            },
        }
    }
}

impl Chunker {
    /// Populates the `.chunks` field of the passed tokens by predicting with the maximum entropy model.
    pub fn apply(&self, sentence: &mut IncompleteSentence) {
        let text = sentence.text().replace('’', "\'");

        let mut bi_to_ci: DefaultHashMap<usize, usize> = text
            .char_indices()
            .enumerate()
            .map(|(ci, (bi, _))| (bi, ci))
            .collect();
        bi_to_ci.insert(text.len(), text.chars().count());

        // the chunker expects tokens tokenized with a maximum entropy tokenizer
        let internal_tokens = self.token_model.tokenize(&text);
        // the chunker gets part-of-speech tags as input so we also have to run a maximum entropy POSTagger before the chunker
        let tags = self.pos_model.tag(&internal_tokens);
        let chunks = self.chunk_model.chunk(
            &internal_tokens
                .iter()
                .cloned()
                .zip(tags.outcomes().iter().cloned())
                .collect::<Vec<_>>(),
        );

        // compute the char span of each chunk to be able to match it with the input tokens
        let internal_chunks: Vec<_> = chunks
            .outcomes()
            .iter()
            .zip(internal_tokens)
            .map(|(chunk, token)| {
                let byte_start = token.as_ptr() as usize - text.as_ptr() as usize;
                let char_start = *bi_to_ci
                    .get(&byte_start)
                    .expect("byte index is at char boundary");
                let char_end = *bi_to_ci
                    .get(&(byte_start + token.len()))
                    .expect("byte index is at char boundary");

                (*chunk, char_start..char_end)
            })
            .collect();
        let mut chunks = Vec::new();
        let mut number = "singular";

        for i in 0..internal_chunks.len() {
            let chunk = internal_chunks[i].0;

            if chunk == "B-NP" {
                number = "singular";

                for (next_chunk, char_span) in internal_chunks[i..].iter().cloned() {
                    if next_chunk != "I-NP" && next_chunk != "B-NP" {
                        break;
                    }

                    let contains_nns = sentence
                        .iter()
                        .find(|token| *token.span().char() == char_span)
                        .map(|token| {
                            token
                                .word()
                                .tags()
                                .iter()
                                .any(|tag| tag.pos().as_str() == "NNS")
                        })
                        .unwrap_or(false);

                    if contains_nns {
                        number = "plural";
                    }
                }
            }

            let is_noun_end = i + 1 >= internal_chunks.len() || internal_chunks[i + 1].0 != "I-NP";
            let mut to_push = Vec::new();

            if chunk == "B-NP" {
                to_push.push(format!("B-NP-{}", number));
                if is_noun_end {
                    to_push.push(format!("E-NP-{}", number));
                }
            } else if chunk == "I-NP" {
                if is_noun_end {
                    to_push.push(format!("E-NP-{}", number));
                } else {
                    to_push.push(format!("I-NP-{}", number));
                }
            } else {
                to_push.push(chunk.into())
            }

            chunks.push(to_push);
        }

        // chunks with exactly the same char span as the input tokens get assigned to the token to match LT
        for token in sentence.iter_mut() {
            for (chunk, (_, char_span)) in chunks.iter().zip(internal_chunks.iter()) {
                if char_span == token.span().char() {
                    *token.chunks_mut() = (*chunk).clone();
                }
            }
        }
    }
}

// Explicit little-endian bytes avoid per-element Serde dispatch in large models.
// No struct padding or native-endian representations enter the asset.
mod packed_slots {
    use super::FeatureSlot;
    use serde::{Deserialize, Deserializer, Serializer, de::Error};
    pub(super) fn serialize<S: Serializer>(
        values: &[FeatureSlot],
        s: S,
    ) -> Result<S::Ok, S::Error> {
        let mut bytes = Vec::with_capacity(values.len() * 13);
        for v in values {
            bytes.extend_from_slice(&v.key.to_le_bytes());
            bytes.extend_from_slice(&v.offset.to_le_bytes());
            bytes.push(v.length);
        }
        s.serialize_bytes(&bytes)
    }
    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Vec<FeatureSlot>, D::Error> {
        let bytes = <&[u8]>::deserialize(d)?;
        if bytes.len() % 13 != 0 {
            return Err(D::Error::custom("invalid feature slots"));
        }
        Ok(bytes
            .chunks_exact(13)
            .map(|b| FeatureSlot {
                key: u64::from_le_bytes(b[..8].try_into().unwrap()),
                offset: u32::from_le_bytes(b[8..12].try_into().unwrap()),
                length: b[12],
            })
            .collect())
    }
}
mod packed_outcomes {
    use serde::{Deserialize, Deserializer, Serializer};
    pub(super) fn serialize<S: Serializer>(values: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(values)
    }
    pub(super) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        Ok(<&[u8]>::deserialize(d)?.to_vec())
    }
}
mod packed_parameters {
    use serde::{Deserialize, Deserializer, Serializer, de::Error};
    pub(super) fn serialize<S: Serializer>(values: &[f32], s: S) -> Result<S::Ok, S::Error> {
        let mut bytes = Vec::with_capacity(values.len() * 4);
        for v in values {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        s.serialize_bytes(&bytes)
    }
    pub(super) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<f32>, D::Error> {
        let bytes = <&[u8]>::deserialize(d)?;
        if bytes.len() % 4 != 0 {
            return Err(D::Error::custom("invalid model parameters"));
        }
        Ok(bytes
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
            .collect())
    }
}

#[cfg(test)]
mod packed_tests {
    use super::*;
    #[test]
    fn parameters_preserve_every_float_bit_and_reject_partial_values() {
        #[derive(Serialize, Deserialize)]
        struct Parameters(#[serde(with = "packed_parameters")] Vec<f32>);
        let bits = [
            0, 0x80000000, 1, 0x7f800000, 0xff800000, 0x7fc12345, 0x3f800000,
        ];
        let original = Parameters(bits.map(f32::from_bits).to_vec());
        let bytes = bincode::serialize(&original).unwrap();
        let decoded: Parameters = bincode::deserialize(&bytes).unwrap();
        assert_eq!(
            decoded.0.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            bits
        );
        let partial = bincode::serialize(&vec![1u8, 2, 3]).unwrap();
        assert!(bincode::deserialize::<Parameters>(&partial).is_err());
        assert!(bincode::deserialize::<FeatureMap>(&partial).is_err());
    }
}
