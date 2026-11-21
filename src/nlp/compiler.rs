//! Build-time decoding and conversion of the pinned linguistic resource.
#[path = "annotation/mod.rs"]
mod annotation;
pub use annotation::Error;
use annotation::{
    Tokenizer,
    chunker::{Chunker, MaxentChunker, MaxentPosTagger, MaxentTokenizer, Model},
    dictionary::{PosIdInt, Tagger, TaggerLangOptions, WordIdInt},
    multiword::MultiwordTagger,
    regex::Regex,
    rules::{
        DisambiguationRule, Filter,
        actions::{Disambiguation, Unification},
        matcher::{Engine, TokenEngine},
        pattern::GraphId,
    },
    tokenizer::TokenizerLangOptions,
    types::{DefaultHashMap, owned},
};
use fst::Streamer;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, ops::Range, sync::Arc};
#[derive(Serialize, Deserialize)]
struct Category(String);
#[derive(Serialize, Deserialize)]
struct Group {
    parent: Category,
    inner: String,
}
#[derive(Serialize, Deserialize)]
struct Index {
    parent: Group,
    inner: usize,
}
impl std::fmt::Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}/{}",
            self.parent.parent.0, self.parent.inner, self.inner
        )
    }
}
#[derive(Serialize, Deserialize)]
enum WireEngine {
    Token(TokenEngine),
    Text(Box<Regex>, BTreeMap<GraphId, usize>),
}
#[derive(Serialize, Deserialize)]
struct WireWord {
    text: owned::WordId,
    tags: Vec<owned::WordData>,
}
#[derive(Serialize, Deserialize)]
struct Change {
    text: String,
    char_span: Range<usize>,
    before: WireWord,
    after: WireWord,
}
#[derive(Serialize, Deserialize)]
enum Example {
    Unchanged(String),
    Changed(Change),
}
#[derive(Serialize, Deserialize)]
enum WireFilter {
    NoDisambiguationEnglishPartialPosTagFilter {
        id: GraphId,
        regexp: Regex,
        postag_regexp: Regex,
        negate_postag: bool,
    },
}
#[derive(Serialize, Deserialize)]
struct WireRule {
    id: Index,
    engine: WireEngine,
    disambiguations: Disambiguation,
    filter: Option<WireFilter>,
    start: GraphId,
    end: GraphId,
    examples: Vec<Example>,
    unification: Option<Unification>,
}
#[derive(Serialize, Deserialize)]
struct ModelFields {
    outcome_labels: Vec<String>,
    pmap: Vec<(u64, u8)>,
    outcomes: Vec<u8>,
    parameters: Vec<half::bf16>,
}
impl From<ModelFields> for Model {
    fn from(m: ModelFields) -> Self {
        Model::from_parts(
            m.outcome_labels,
            m.pmap,
            m.outcomes,
            m.parameters.into_iter().map(|p| p.to_f32()).collect(),
        )
    }
}
#[derive(Serialize, Deserialize)]
struct Single {
    model: ModelFields,
}
#[derive(Serialize, Deserialize)]
struct Pos {
    model: ModelFields,
    tagdict: DefaultHashMap<String, Vec<String>>,
}
#[derive(Serialize, Deserialize)]
struct WireChunker {
    token_model: Single,
    pos_model: Pos,
    chunk_model: Single,
}
#[derive(Serialize, Deserialize)]
struct TaggerFields {
    tag_fst: Vec<u8>,
    word_store_fst: Vec<u8>,
    tag_store: bimap::BiMap<String, PosIdInt>,
    lang_options: TaggerLangOptions,
    anchors: Vec<String>,
}
#[derive(Serialize, Deserialize)]
struct WireTokenizer {
    rules: Vec<WireRule>,
    chunker: Option<WireChunker>,
    sentencizer: srx::Rules,
    multiword_tagger: Option<MultiwordTagger>,
    tagger: TaggerFields,
    lang_options: TokenizerLangOptions,
}
fn invalid(e: impl std::fmt::Display) -> Error {
    Error::Serialization(Box::new(bincode::ErrorKind::Custom(e.to_string())))
}
fn tagger(t: TaggerFields) -> Result<Tagger, Error> {
    let words = fst::Map::new(t.word_store_fst).map_err(invalid)?;
    let mut ranges = vec![(0, 0); words.len()];
    let mut text = String::new();
    let mut stream = words.stream();
    while let Some((key, id)) = stream.next() {
        let start = text.len() as u32;
        text.push_str(std::str::from_utf8(key).map_err(invalid)?);
        ranges[id as usize] = (start, text.len() as u32);
    }
    let tags_fst = fst::Map::new(t.tag_fst).map_err(invalid)?;
    let mut tags = Vec::new();
    let mut tag_ranges = vec![(0, 0); words.len()];
    let mut seen = vec![false; words.len()];
    let mut stream = tags_fst.stream();
    while let Some((key, value)) = stream.next() {
        let id = words
            .get(&key[..key.len() - 1])
            .ok_or_else(|| invalid("missing dictionary word"))? as usize;
        if !seen[id] {
            tag_ranges[id].0 = tags.len() as u32;
            seen[id] = true;
        }
        let bytes = value.to_be_bytes();
        tags.push((
            WordIdInt::from_value_unchecked(u32::from_be_bytes(bytes[..4].try_into().unwrap())),
            PosIdInt::from_value_unchecked(u16::from_be_bytes(bytes[6..].try_into().unwrap())),
        ));
        tag_ranges[id].1 = tags.len() as u32;
    }
    Ok(Tagger::from_parts(
        words,
        text,
        ranges,
        tags,
        tag_ranges,
        t.tag_store.into_iter().collect(),
        t.lang_options,
    ))
}
pub fn compile_asset(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let wire: WireTokenizer = bincode::deserialize(bytes)?;
    let mut rules = Vec::new();
    for r in wire.rules {
        let engine = match r.engine {
            WireEngine::Token(t) => Engine::Token(t),
            WireEngine::Text(..) => return Err(Error::UnsupportedTextEngine(r.id.to_string())),
        };
        if matches!(r.disambiguations, Disambiguation::Nop) {
            continue;
        }
        let filter = r.filter.map(|f| match f {
            WireFilter::NoDisambiguationEnglishPartialPosTagFilter {
                id,
                regexp,
                postag_regexp,
                ..
            } => Filter::NoDisambiguationEnglishPartialPosTagFilter {
                id,
                regexp,
                postag_regexp,
            },
        });
        rules.push(DisambiguationRule {
            id: r.id.to_string(),
            engine,
            disambiguations: r.disambiguations,
            filter,
            start: r.start,
            end: r.end,
            unification: r.unification,
        });
    }
    let chunker = wire.chunker.map(|c| Chunker {
        token_model: MaxentTokenizer {
            model: c.token_model.model.into(),
        },
        pos_model: MaxentPosTagger {
            model: c.pos_model.model.into(),
            tagdict: c.pos_model.tagdict,
        },
        chunk_model: MaxentChunker {
            model: c.chunk_model.model.into(),
        },
    });
    Tokenizer {
        rules,
        chunker,
        sentencizer: wire.sentencizer,
        multiword_tagger: wire.multiword_tagger,
        tagger: Arc::new(tagger(wire.tagger)?),
        lang_options: wire.lang_options,
    }
    .compile_asset()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn resource() -> &'static [u8] {
        include_bytes!(env!("LINN_POS_TOKENIZER_BUILD"))
    }
    #[test]
    fn full_resource_bytes_and_determinism() {
        let raw = resource();
        let out = compile_asset(raw).unwrap();
        use sha2::{Digest, Sha256};
        assert_eq!(
            format!("{:x}", Sha256::digest(&out)),
            "08917743286aeefa2c162fd855b59136555adbbcf8552fe0a81e42fdcb873e5a"
        );
        assert!(
            out == compile_asset(raw).unwrap(),
            "compiled asset must be deterministic"
        );
        assert_eq!(
            out,
            Tokenizer::from_compiled(&out)
                .unwrap()
                .compile_asset()
                .unwrap()
        );
    }
    #[test]
    fn rejects_text_even_with_nop() {
        let mut raw: WireTokenizer = bincode::deserialize(resource()).unwrap();
        let rule = raw.rules.first_mut().unwrap();
        rule.engine = WireEngine::Text(Box::new(Regex::new("x".into())), BTreeMap::new());
        rule.disambiguations = Disambiguation::Nop;
        assert!(matches!(
            compile_asset(&bincode::serialize(&raw).unwrap()),
            Err(Error::UnsupportedTextEngine(_))
        ));
    }
    #[test]
    fn slash_ids_preserve_empty_components() {
        let id = Index {
            parent: Group {
                parent: Category("".into()),
                inner: "a/b".into(),
            },
            inner: 0,
        };
        assert_eq!(id.to_string(), "/a/b/0");
    }
}
