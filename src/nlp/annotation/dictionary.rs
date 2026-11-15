use crate::nlp::annotation::types::WordData;
use bimap::BiMap;
use fst::Map;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};
macro_rules! integer {
    ($n:ident,$t:ty) => {
        #[derive(
            Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        pub(crate) struct $n($t);
        impl $n {
            #[allow(dead_code)] // Asset compiler and synthetic fixture constructor.
            pub(crate) fn from_value_unchecked(value: $t) -> Self {
                Self(value)
            }
        }
    };
}
integer!(WordIdInt, u32);
integer!(PosIdInt, u16);
impl PosIdInt {
    pub fn value(&self) -> u16 {
        self.0
    }
}
#[derive(Clone, PartialEq)]
pub struct WordId<'t>(pub(crate) Cow<'t, str>, pub(crate) Option<WordIdInt>);
impl fmt::Debug for WordId<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.1 {
            Some(i) => write!(f, "{:?}<id={}>", self.0, i.0),
            None => write!(f, "{:?}<id=none>", self.0),
        }
    }
}
impl<'t> WordId<'t> {
    pub(crate) fn id(&self) -> &Option<WordIdInt> {
        &self.1
    }
    pub(crate) fn empty() -> Self {
        Self(Cow::Borrowed(""), Some(WordIdInt(0)))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpecialPos {
    Empty,
    Unknown,
    SentStart,
    SentEnd,
}
impl SpecialPos {
    pub fn as_str(&self) -> &'static str {
        ["", "UNKNOWN", "SENT_START", "SENT_END"][*self as usize]
    }
}
#[derive(Clone, Copy, PartialEq)]
pub struct PosId<'t> {
    text: &'t str,
    id: PosIdInt,
    special: bool,
}
impl fmt::Debug for PosId<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}<id={}>", self.text, self.id.0)
    }
}
impl<'t> PosId<'t> {
    pub(crate) fn regular(text: &'t str, id: PosIdInt) -> Self {
        Self {
            text,
            id,
            special: false,
        }
    }
    pub(crate) fn special(s: SpecialPos) -> Self {
        Self {
            text: s.as_str(),
            id: PosIdInt(s as u16),
            special: true,
        }
    }
    pub(crate) fn id(&self) -> PosIdInt {
        self.id
    }
    pub fn as_str(&self) -> &'t str {
        self.text
    }
}
pub mod owned {
    use super::*;
    #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct WordId(pub(crate) String, pub(crate) Option<WordIdInt>);
    #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct PosId(pub(crate) String, pub(crate) PosIdInt);
    #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct WordData {
        pub lemma: WordId,
        pub pos: PosId,
    }
    impl From<super::WordId<'_>> for WordId {
        fn from(x: super::WordId<'_>) -> Self {
            Self(x.0.into_owned(), x.1)
        }
    }
    impl From<super::PosId<'_>> for PosId {
        fn from(x: super::PosId<'_>) -> Self {
            Self(x.text.into(), x.id)
        }
    }
    impl<'a> From<&'a WordId> for super::WordId<'a> {
        fn from(x: &'a WordId) -> Self {
            Self(Cow::Borrowed(&x.0), x.1)
        }
    }
    impl<'a> From<&'a PosId> for super::PosId<'a> {
        fn from(x: &'a PosId) -> Self {
            Self::regular(&x.0, x.1)
        }
    }
}
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub(crate) struct TaggerLangOptions {
    pub use_compound_split_heuristic: bool,
    pub always_add_lower_tags: bool,
    pub extra_tags: Vec<String>,
    pub retain_last: bool,
}
mod fst_wire {
    use super::*;
    pub fn serialize<S: serde::Serializer>(m: &Map<Vec<u8>>, s: S) -> Result<S::Ok, S::Error> {
        m.as_fst().as_bytes().serialize(s)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Map<Vec<u8>>, D::Error> {
        Map::new(Vec::<u8>::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}
mod labels_wire {
    use super::*;
    pub fn serialize<S: serde::Serializer>(
        m: &BiMap<String, PosIdInt>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        m.iter()
            .collect::<std::collections::BTreeMap<_, _>>()
            .serialize(s)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        d: D,
    ) -> Result<BiMap<String, PosIdInt>, D::Error> {
        Ok(
            std::collections::BTreeMap::<String, PosIdInt>::deserialize(d)?
                .into_iter()
                .collect(),
        )
    }
}
#[derive(Serialize, Deserialize)]
pub struct Tagger {
    #[serde(with = "fst_wire")]
    words: Map<Vec<u8>>,
    word_bytes: String,
    word_ranges: Vec<(u32, u32)>,
    tags: Vec<(WordIdInt, PosIdInt)>,
    tag_ranges: Vec<(u32, u32)>,
    #[serde(with = "labels_wire")]
    pub(crate) tag_store: BiMap<String, PosIdInt>,
    pub(crate) lang_options: TaggerLangOptions,
}
pub type TagIter<'a> = std::vec::IntoIter<WordData<'a>>;
impl Tagger {
    #[allow(dead_code)] // Build-only asset constructor; runtime deserializes prepared fields.
    pub(crate) fn from_parts(
        words: Map<Vec<u8>>,
        word_bytes: String,
        word_ranges: Vec<(u32, u32)>,
        tags: Vec<(WordIdInt, PosIdInt)>,
        tag_ranges: Vec<(u32, u32)>,
        labels: Vec<(String, PosIdInt)>,
        lang_options: TaggerLangOptions,
    ) -> Self {
        Self {
            words,
            word_bytes,
            word_ranges,
            tags,
            tag_ranges,
            tag_store: labels.into_iter().collect(),
            lang_options,
        }
    }
    pub fn id_word<'a>(&self, text: Cow<'a, str>) -> WordId<'a> {
        let id = self.words.get(text.as_bytes()).map(|i| WordIdInt(i as u32));
        WordId(text, id)
    }
    pub fn id_tag<'a>(&self, tag: &'a str) -> PosId<'a> {
        PosId::regular(
            tag,
            *self
                .tag_store
                .get_by_left(tag)
                .unwrap_or_else(|| self.tag_store.get_by_left("UNKNOWN").unwrap()),
        )
    }
    fn exact<'a>(&'a self, w: &str) -> Vec<WordData<'a>> {
        let Some(i) = self.words.get(w) else {
            return vec![];
        };
        let (a, b) = self.tag_ranges[i as usize];
        self.tags[a as usize..b as usize]
            .iter()
            .map(|(w, p)| {
                let (a, b) = self.word_ranges[w.0 as usize];
                let text = &self.word_bytes[a as usize..b as usize];
                WordData::new(
                    self.id_word(Cow::Borrowed(text)),
                    self.id_tag(self.tag_store.get_by_right(p).unwrap()),
                )
            })
            .collect()
    }
    fn ordinary<'a>(&'a self, w: &str, lower: bool, fallback: bool) -> Vec<WordData<'a>> {
        let mut v = self.exact(w);
        let lo = w.to_lowercase();
        let title = w.chars().next().is_some_and(char::is_uppercase)
            && !w.chars().skip(1).any(char::is_uppercase);
        let upper = !w.chars().any(char::is_lowercase);
        if lo != w && (title || upper) && (lower || (fallback && v.is_empty())) {
            v.extend(self.exact(&lo));
        }
        v
    }
    pub fn get_tags_with_options<'a>(
        &'a self,
        word: &'a str,
        add_lower: Option<bool>,
        compound: Option<bool>,
    ) -> TagIter<'a> {
        let lower = add_lower.unwrap_or(self.lang_options.always_add_lower_tags);
        let mut v = self.ordinary(word, lower, true);
        let n = word.chars().count();
        if v.is_empty()
            && compound.unwrap_or(self.lang_options.use_compound_split_heuristic)
            && n >= 7
        {
            for (i, _) in word.char_indices().skip(1).take(n - 5) {
                let suffix = &word[i..];
                let query = if word.chars().next().is_some_and(char::is_uppercase) {
                    let mut c = suffix.chars();
                    c.next().unwrap().to_uppercase().collect::<String>() + c.as_str()
                } else {
                    suffix.into()
                };
                let found = self.ordinary(&query, lower, false);
                if !found.is_empty() {
                    v = found
                        .into_iter()
                        .map(|d| {
                            let lemma = word[..i].to_owned() + &d.lemma().as_str().to_lowercase();
                            WordData::new(self.id_word(Cow::Owned(lemma)), *d.pos())
                        })
                        .collect();
                    break;
                }
            }
        }
        v.into_iter()
    }
    pub fn get_tags<'a>(&'a self, word: &'a str) -> TagIter<'a> {
        self.get_tags_with_options(word, None, None)
    }
}
impl From<&WordId<'_>> for owned::WordId {
    fn from(x: &WordId<'_>) -> Self {
        Self(x.0.to_string(), x.1)
    }
}
impl From<&PosId<'_>> for owned::PosId {
    fn from(x: &PosId<'_>) -> Self {
        Self(x.text.into(), x.id)
    }
}
impl From<WordData<'_>> for owned::WordData {
    fn from(x: WordData<'_>) -> Self {
        Self {
            lemma: x.lemma().into(),
            pos: x.pos().into(),
        }
    }
}
impl<'a> From<&'a owned::WordData> for WordData<'a> {
    fn from(x: &'a owned::WordData) -> Self {
        Self::new((&x.lemma).into(), (&x.pos).into())
    }
}
impl owned::WordId {
    pub fn as_ref_id(&self) -> WordId<'_> {
        self.into()
    }
}
impl owned::PosId {
    pub fn as_ref_id(&self) -> PosId<'_> {
        self.into()
    }
}
impl AsRef<str> for owned::WordId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl AsRef<str> for owned::PosId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_fst_rejected() {
        let encoded = bincode::serialize(&vec![0u8; 8]).unwrap();
        let result: Result<FstProbe, _> = bincode::deserialize(&encoded);
        assert!(result.is_err());
    }
    #[derive(Deserialize)]
    struct FstProbe {
        #[serde(with = "fst_wire")]
        _map: Map<Vec<u8>>,
    }
    #[test]
    fn special_identity_is_lost_on_owned_conversion() {
        for s in [
            SpecialPos::Empty,
            SpecialPos::Unknown,
            SpecialPos::SentStart,
            SpecialPos::SentEnd,
        ] {
            let p = PosId::special(s);
            let o = owned::PosId::from(p);
            let regular = PosId::from(&o);
            assert_ne!(p, regular);
            assert_eq!(p.id(), regular.id());
            assert_eq!(p.as_str(), regular.as_str());
        }
    }
}
