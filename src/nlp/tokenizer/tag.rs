// Portions derived from nlprule 0.6.4. See src/nlp/LICENSE.
//! A dictionary-based tagger.

use crate::nlp::types::*;
use bimap::BiMap;
use fst::{Map, Streamer};
use log::error;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt, iter::FusedIterator};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub(crate) struct WordIdInt(u32);

#[allow(dead_code)] // used in compile module
impl WordIdInt {
    pub(crate) fn from_value_unchecked(value: u32) -> Self {
        WordIdInt(value)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub(crate) struct PosIdInt(u16);

impl PosIdInt {
    #[allow(dead_code)] // used in compile module
    pub(crate) fn from_value_unchecked(value: u16) -> Self {
        PosIdInt(value)
    }

    pub fn value(&self) -> u16 {
        self.0
    }
}

/// A potentially identified word. If it is identified as a known word, many optimizations can be applied.
#[derive(Clone, PartialEq)]
pub struct WordId<'t>(pub(crate) Cow<'t, str>, pub(crate) Option<WordIdInt>);

impl<'t> fmt::Debug for WordId<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id_str = if let Some(id) = self.1 {
            id.0.to_string()
        } else {
            "none".into()
        };

        write!(f, "{:?}<id={}>", self.0, id_str)
    }
}

impl<'t> WordId<'t> {
    pub(crate) fn to_owned_id(&self) -> owned::WordId {
        owned::WordId(self.0.to_string(), self.1)
    }

    pub(crate) fn id(&self) -> &Option<WordIdInt> {
        &self.1
    }

    pub(crate) fn empty() -> Self {
        WordId("".into(), Some(WordIdInt(0)))
    }

    /// Gets the word as string.
    pub fn as_str(&'t self) -> &'t str {
        self.0.as_ref()
    }
}

#[derive(Copy, Clone, PartialEq)]
pub(crate) enum SpecialPos {
    None = 0,
    Unknown = 1,
    SentStart = 2,
    SentEnd = 3,
}

impl SpecialPos {
    pub fn as_str(&self) -> &'static str {
        match &self {
            SpecialPos::None => "",
            SpecialPos::Unknown => "UNKNOWN",
            SpecialPos::SentStart => "SENT_START",
            SpecialPos::SentEnd => "SENT_END",
        }
    }

    #[allow(dead_code)] // used in compile module
    pub fn iter() -> impl Iterator<Item = &'static str> + 'static {
        [
            SpecialPos::None,
            SpecialPos::Unknown,
            SpecialPos::SentStart,
            SpecialPos::SentEnd,
        ]
        .iter()
        .map(|pos| pos.as_str())
    }
}

#[derive(Clone, Copy, PartialEq)]
enum InnerPosId<'t> {
    Normal(&'t str, PosIdInt),
    Special(SpecialPos),
}

/// An identified part-of-speech tag. POS tags are treated as a closed set so every POS tag is identified.
#[derive(Clone, Copy, PartialEq)]
pub struct PosId<'t> {
    inner: InnerPosId<'t>,
}

impl<'t> fmt::Debug for PosId<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}<id={}>", self.as_str(), self.id().0)
    }
}

impl<'t> PosId<'t> {
    pub(crate) fn regular(text: &'t str, id: PosIdInt) -> Self {
        PosId {
            inner: InnerPosId::Normal(text, id),
        }
    }

    pub(crate) fn special(special: SpecialPos) -> Self {
        PosId {
            inner: InnerPosId::Special(special),
        }
    }

    /// Converts this ID to an owned ID.
    pub fn to_owned_id(self) -> owned::PosId {
        owned::PosId(self.as_str().to_string(), self.id())
    }

    pub(crate) fn id(&self) -> PosIdInt {
        match &self.inner {
            InnerPosId::Normal(_, id) => *id,
            InnerPosId::Special(special) => PosIdInt(*special as u16),
        }
    }

    /// Gets the part-of-speech as string.
    pub fn as_str(&self) -> &'t str {
        match &self.inner {
            InnerPosId::Normal(text, _) => text,
            InnerPosId::Special(special) => special.as_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct TaggerLangOptions {
    /// Whether to use a heuristic to split potential compound words.
    pub use_compound_split_heuristic: bool,
    /// Whether to always add tags for a lowercase version of the word when assigning part-of-speech tags.
    pub always_add_lower_tags: bool,
    /// Used part-of-speech tags which are not in the tagger dictionary.
    pub extra_tags: Vec<String>,
    /// Whether to retain the last tag if disambiguation leads to an empty tag.
    /// Language-specific in LT so it has to be an option.
    pub retain_last: bool,
}

#[derive(Serialize, Deserialize)]
pub(super) struct TaggerFields {
    tag_fst: Vec<u8>,
    word_store_fst: Vec<u8>,
    tag_store: BiMap<String, PosIdInt>,
    lang_options: TaggerLangOptions,
    anchors: Vec<String>,
}

// Convert the upstream wire dictionary once during the asset build. Runtime
// lookup uses the FST and flat arrays without rebuilding hash maps.
impl From<TaggerFields> for Tagger {
    fn from(data: TaggerFields) -> Self {
        let words = Map::new(data.word_store_fst).expect("valid dictionary FST");
        let mut stream = words.stream();
        let mut word_bytes = String::new();
        let mut word_ranges = vec![(0_u32, 0_u32); words.len()];
        while let Some((key, id)) = stream.next() {
            let start = word_bytes.len() as u32;
            word_bytes.push_str(std::str::from_utf8(key).expect("UTF-8 dictionary word"));
            word_ranges[id as usize] = (start, word_bytes.len() as u32);
        }
        drop(stream);
        let tags_fst = Map::new(data.tag_fst).expect("valid tag FST");
        let mut tags = Vec::new();
        let mut tag_ranges = vec![(0_u32, 0_u32); words.len()];
        let mut stream = tags_fst.stream();
        let mut previous = None;
        while let Some((key, value)) = stream.next() {
            let id = words.get(&key[..key.len() - 1]).expect("tag word exists") as usize;
            if previous != Some(id) {
                tag_ranges[id].0 = tags.len() as u32;
                previous = Some(id);
            }
            let bytes = value.to_be_bytes();
            tags.push((
                WordIdInt(u32::from_be_bytes(bytes[..4].try_into().unwrap())),
                PosIdInt(u16::from_be_bytes(bytes[6..].try_into().unwrap())),
            ));
            tag_ranges[id].1 = tags.len() as u32;
        }
        Self {
            words,
            word_bytes,
            word_ranges,
            tags,
            tag_ranges,
            tag_store: data.tag_store,
            lang_options: data.lang_options,
        }
    }
}

mod dictionary_bytes {
    use fst::Map;
    use serde::{
        Deserializer, Serializer,
        de::{Error, Visitor},
    };
    pub fn serialize<S: Serializer>(map: &Map<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(map.as_fst().as_bytes())
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Map<Vec<u8>>, D::Error> {
        struct Bytes;
        impl<'de> Visitor<'de> for Bytes {
            type Value = Map<Vec<u8>>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("dictionary FST bytes")
            }
            fn visit_bytes<E: Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
                Map::new(bytes.to_vec()).map_err(E::custom)
            }
            fn visit_byte_buf<E: Error>(self, bytes: Vec<u8>) -> Result<Self::Value, E> {
                Map::new(bytes).map_err(E::custom)
            }
        }
        deserializer.deserialize_bytes(Bytes)
    }
}

enum AddLower {
    Always,
    IfEmpty,
    Never,
}

struct RawTagIter<'a> {
    tagger: &'a Tagger,
    slice_iter: std::slice::Iter<'a, (WordIdInt, PosIdInt)>,
}

impl<'a> Iterator for RawTagIter<'a> {
    type Item = WordData<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.slice_iter.next().map(move |(lemma_id, pos_id)| {
            WordData::new(
                self.tagger
                    .id_word(self.tagger.str_for_word_id(lemma_id).into()),
                self.tagger.id_tag(self.tagger.str_for_pos_id(pos_id)),
            )
        })
    }
}

impl<'a> FusedIterator for RawTagIter<'a> {}
impl<'a> ExactSizeIterator for RawTagIter<'a> {
    fn len(&self) -> usize {
        self.slice_iter.len()
    }
}

struct StrictTagIter<'a> {
    tag_iter: RawTagIter<'a>,
    lower_tag_iter: Option<RawTagIter<'a>>,
}

impl<'a> Iterator for StrictTagIter<'a> {
    type Item = WordData<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tag_iter.next().or_else(|| {
            self.lower_tag_iter
                .as_mut()
                .and_then(|lower_iter| lower_iter.next())
        })
    }
}

impl<'a> FusedIterator for StrictTagIter<'a> {}
impl<'a> ExactSizeIterator for StrictTagIter<'a> {
    fn len(&self) -> usize {
        self.tag_iter.len()
            + self
                .lower_tag_iter
                .as_ref()
                .map_or(0, |lower_iter| lower_iter.len())
    }
}

/// An iterator over [WordData].
pub struct TagIter<'a> {
    tagger: &'a Tagger,
    tag_iter: StrictTagIter<'a>,
    prefix: Option<&'a str>,
}

impl<'a> Iterator for TagIter<'a> {
    type Item = WordData<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tag_iter.next().map(|data| {
            let lemma = if let Some(prefix) = &self.prefix {
                self.tagger
                    .id_word(format!("{}{}", prefix, data.lemma().as_str().to_lowercase()).into())
            } else {
                data.lemma().clone()
            };

            WordData::new(lemma, *data.pos())
        })
    }
}

impl<'a> FusedIterator for TagIter<'a> {}
impl<'a> ExactSizeIterator for TagIter<'a> {
    fn len(&self) -> usize {
        self.tag_iter.len()
    }
}

/// The lexical tagger.
/// Created from a dictionary that looks like this:
///
/// ```raw
/// actualize   actualize   VB
/// actualize   actualize   VBP
/// actualized  actualize   VBD
/// actualized  actualize   VBN
/// actualizes  actualize   VBZ
/// actualizing actualize   VBG
/// actually    actually    RB
/// ```
///
/// i.e. one word (left) associated with one or more pairs of lemma (middle) and POS (part-of-speech) tag (right).
/// From this structure, the tagger must be able to look up:
/// 1. lemma and pos by word: all lemmas and POS tags associated with a given word.
///
/// (1) is called extensively (at least once for every word) so it has to be as fast as possible.
///
/// ## Implementation
///
/// The tagger stores two bidirectional maps:
/// A POS bimap: A bimap assigning each POS tag a 16-bit ID. POS tags are a closed set, so there is an entry
/// for every tag in the bimap. This allows e.g. storing a set of IDs of matching tags for a regex instead of actually
/// evaluating it in the matcher logic.
/// A word bimap: A bimap assigning each known word a 32-bit ID. Words are not a closed set, so if an entry in this map
/// does not exist it only means that the word is not known to nlprule. Still, the map can be used for optimizations similar
/// to the POS bimap. The word bimap also stores lemmas since there is often a large overlap between known words
/// and known lemmas.
///
/// These two maps can be used to relatively cheaply in terms of memory allow (1) while retaining fast lookup.
///
/// There is a `tags` map which associates a Word ID with *multiple* pairs of
/// `(lemma_id, pos_id)` where the ID for the lemma is a regular 32-bit Word ID.
#[derive(Serialize, Deserialize, Clone)]
pub struct Tagger {
    #[serde(with = "dictionary_bytes")]
    words: Map<Vec<u8>>,
    word_bytes: String,
    word_ranges: Vec<(u32, u32)>,
    tags: Vec<(WordIdInt, PosIdInt)>,
    tag_ranges: Vec<(u32, u32)>,
    #[serde(serialize_with = "serialize_tag_store")]
    pub(crate) tag_store: BiMap<String, PosIdInt>,
    pub(crate) lang_options: TaggerLangOptions,
}

fn serialize_tag_store<S: serde::Serializer>(
    values: &BiMap<String, PosIdInt>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut entries: Vec<_> = values.iter().collect();
    entries.sort_unstable_by(|a, b| a.0.cmp(b.0));
    serializer.collect_map(entries)
}

impl Tagger {
    /// Directly looks up the given word in the `tags` map and returns
    /// corresponding [WordData].
    fn get_raw(&self, word: &str) -> RawTagIter<'_> {
        let slice = self.words.get(word).map_or(&[][..], |id| {
            let (start, end) = self.tag_ranges[id as usize];
            &self.tags[start as usize..end as usize]
        });

        RawTagIter {
            tagger: self,
            slice_iter: slice.iter(),
        }
    }

    #[allow(dead_code)] // used in compile module
    pub(crate) fn lang_options(&self) -> &TaggerLangOptions {
        &self.lang_options
    }

    /// Same as [get_tags] but optionally:
    /// Adds tags for the lower variant of the word (if `add_lower` is `AddLower::Always`).
    /// Adds tags for the lower variant of the word if no [WordData] is found otherwise.
    /// (if `add_lower` is `AddLower::IfEmpty`).
    fn get_strict_tags(&self, word: &str, add_lower: AddLower) -> StrictTagIter<'_> {
        let tag_iter = self.get_raw(word);
        let lower_tag_iter = if (matches!(add_lower, AddLower::Always)
            || (matches!(add_lower, AddLower::IfEmpty) && tag_iter.len() == 0))
        {
            let lower = word.to_lowercase();

            if (word != lower)
                && (crate::nlp::utils::is_title_case(word) || crate::nlp::utils::is_uppercase(word))
            {
                Some(self.get_raw(&lower))
            } else {
                None
            }
        } else {
            None
        };

        StrictTagIter {
            tag_iter,
            lower_tag_iter,
        }
    }

    #[allow(dead_code)] // used by compile module
    pub(crate) fn tag_store(&self) -> &BiMap<String, PosIdInt> {
        &self.tag_store
    }

    /// Gets the original dictionary spelling for a stable word id.
    fn str_for_word_id(&self, id: &WordIdInt) -> &str {
        let (start, end) = self.word_ranges[id.0 as usize];
        &self.word_bytes[start as usize..end as usize]
    }

    /// Gets the string associated with a POS ID.
    fn str_for_pos_id(&self, id: &PosIdInt) -> &str {
        self.tag_store
            .get_by_right(id)
            .expect("only valid pos ids are created")
    }

    /// Tags the given string representation of a part-of-speech tag.
    /// Part-of-speech tags are treated as a closed set so each valid part-of-speech tag will get a numerical id.
    pub fn id_tag<'a>(&self, tag: &'a str) -> PosId<'a> {
        PosId::regular(
            tag,
            *self.tag_store.get_by_left(tag).unwrap_or_else(|| {
                error!(
                    "'{}' not found in tag store, please add it to the `extra_tags`. Using UNKNOWN instead.",
                    tag
                );
                self.tag_store.get_by_left("UNKNOWN").expect("UNKNOWN tag must exist in tag store")
            }),
        )
    }

    /// Tags the given text.
    /// Unknown words will not get a numerical id.
    pub fn id_word<'a>(&self, text: Cow<'a, str>) -> WordId<'a> {
        let id = self.words.get(text.as_ref()).map(|id| WordIdInt(id as u32));
        WordId(text, id)
    }

    /// Get the tags and lemmas (as [WordData]) for the given word.
    ///
    /// # Arguments
    /// `word`: The word to lookup data for.
    /// `add_lower`: Whether to add data for the lowercase variant of the word.
    /// If `None`, will be set according to the language options.
    /// `use_compound_split_heuristic`: Whether to use a heuristic to split compound words.
    /// If `None`, will be set according to the language options.
    /// If true, will attempt to find tags for words which are longer than some cutoff and unknown by looking up tags
    /// for substrings from left to right until tags are found or a minimum length reached.
    pub fn get_tags_with_options<'a>(
        &'a self,
        word: &'a str,
        add_lower: Option<bool>,
        use_compound_split_heuristic: Option<bool>,
    ) -> TagIter<'a> {
        let add_lower = add_lower.unwrap_or(self.lang_options.always_add_lower_tags);
        let use_compound_split_heuristic =
            use_compound_split_heuristic.unwrap_or(self.lang_options.use_compound_split_heuristic);

        let mut tags = TagIter {
            tagger: self,
            tag_iter: self.get_strict_tags(
                word,
                if add_lower {
                    AddLower::Always
                } else {
                    AddLower::IfEmpty
                },
            ),
            prefix: None,
        };

        // compound splitting heuristic, seems to work reasonably well
        if use_compound_split_heuristic && tags.len() == 0 {
            let n_chars = word.chars().count() as isize;

            if n_chars >= 7 {
                let indices = word
                    .char_indices()
                    .take(std::cmp::max(n_chars - 4, 0) as usize)
                    .skip(1)
                    .map(|x| x.0);
                // the word always has at least one char if the above condition is satisfied
                // but semantically this is false if no char exists
                let starts_with_uppercase = word.chars().next().is_some_and(|x| x.is_uppercase());

                for i in indices {
                    let next = if starts_with_uppercase {
                        crate::nlp::utils::apply_to_first(&word[i..], |c| {
                            c.to_uppercase().collect()
                        })
                    } else {
                        word[i..].to_string()
                    };

                    let next_tags = self.get_strict_tags(
                        &next,
                        if add_lower {
                            AddLower::Always
                        } else {
                            AddLower::Never
                        },
                    );

                    if next_tags.len() != 0 {
                        tags = TagIter {
                            tagger: self,
                            tag_iter: next_tags,
                            prefix: Some(&word[..i]),
                        };
                        break;
                    }
                }
            }
        }

        tags
    }

    /// Get the tags and lemmas (as [WordData][crate::nlp::types::WordData]) for the given word
    /// using the default options of the tagger.
    ///
    /// # Arguments
    /// `word`: The word to lookup data for.
    pub fn get_tags<'a>(&'a self, word: &'a str) -> TagIter<'a> {
        self.get_tags_with_options(word, None, None)
    }
}
