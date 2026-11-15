//! Ordered sentence splitting, linguistic annotation and disambiguation.
use super::{chunker::Chunker, dictionary::Tagger, multiword::MultiwordTagger};
use crate::nlp::annotation::{
    Error,
    regex::Regex,
    rules::DisambiguationRule,
    types::{IncompleteSentence, IncompleteToken, Position, Sentence, Span, Word},
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, ops::Range, sync::Arc};

// Token-separator compatibility data; Unicode whitespace is handled separately.
const SPLITTING_CHARACTERS: &str = r##"¡¿«»'’`´‘],.:;!?/\()<=>„“”"+#…*"##;
const MARKER: &[u8] = b"LINN-NLP-4\0";
#[derive(Clone, Serialize, Deserialize)]
pub struct Tokenizer {
    pub(crate) rules: Vec<DisambiguationRule>,
    pub(crate) chunker: Option<Chunker>,
    pub(crate) sentencizer: srx::Rules,
    pub(crate) multiword_tagger: Option<MultiwordTagger>,
    pub(crate) tagger: Arc<Tagger>,
    pub(crate) lang_options: TokenizerLangOptions,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct TokenizerLangOptions {
    allow_errors: bool,
    #[serde(default)]
    ids: Vec<String>,
    #[serde(default)]
    ignore_ids: Vec<String>,
    #[serde(default)]
    known_failures: Vec<String>,
    #[serde(default)]
    extra_split_chars: Vec<char>,
    #[serde(default)]
    extra_join_regexes: Vec<Regex>,
}
fn primary(c: char) -> bool {
    c.is_whitespace() || SPLITTING_CHARACTERS.contains(c)
}
fn separate(text: &str, base: usize, extra: &[char]) -> Vec<Range<usize>> {
    let mut result = Vec::new();
    let mut start = 0;
    for (i, c) in text.char_indices() {
        if primary(c) || extra.contains(&c) {
            if start < i {
                result.push(base + start..base + i);
            }
            result.push(base + i..base + i + c.len_utf8());
            start = i + c.len_utf8();
        }
    }
    if start < text.len() {
        result.push(base + start..base + text.len());
    }
    result
}
impl Tokenizer {
    pub fn from_compiled(bytes: &[u8]) -> Result<Self, Error> {
        let payload = bytes.strip_prefix(MARKER).ok_or_else(|| {
            Error::Serialization(Box::new(bincode::ErrorKind::Custom(
                "unsupported NLP asset marker".into(),
            )))
        })?;
        Ok(bincode::deserialize(payload)?)
    }
    pub fn compile_asset(&self) -> Result<Vec<u8>, Error> {
        let mut out = MARKER.to_vec();
        out.extend(bincode::serialize(self)?);
        Ok(out)
    }
    fn split_gap(&self, text: &str, range: Range<usize>, out: &mut Vec<Range<usize>>) {
        for piece in separate(&text[range.clone()], range.start, &[]) {
            if self
                .tagger
                .id_word(Cow::Borrowed(&text[piece.clone()]))
                .id()
                .is_some()
            {
                out.push(piece);
            } else {
                out.extend(separate(
                    &text[piece.clone()],
                    piece.start,
                    &self.lang_options.extra_split_chars,
                ));
            }
        }
    }
    fn pieces(&self, text: &str) -> Vec<Range<usize>> {
        let mut claimed = vec![false; text.len()];
        let mut joins = Vec::new();
        for regex in &self.lang_options.extra_join_regexes {
            for m in regex.find_iter(text) {
                let r = m.range();
                if claimed[r.clone()].iter().all(|x| !*x) {
                    claimed[r.clone()].fill(true);
                    joins.push(r);
                }
            }
        }
        joins.sort_by_key(|r| r.start);
        let mut out = Vec::new();
        let mut cursor = 0;
        for r in joins {
            if cursor < r.start {
                self.split_gap(text, cursor..r.start, &mut out);
            }
            cursor = cursor.max(r.end);
            out.push(r);
        }
        self.split_gap(text, cursor..text.len(), &mut out);
        out
    }
    pub fn tokenize<'t>(&'t self, text: &'t str) -> Option<IncompleteSentence<'t>> {
        let pieces = self.pieces(text);
        let count = pieces.len();
        let mut tokens = Vec::new();
        let mut positions = text.char_indices().peekable();
        let mut chars = 0;
        for (i, r) in pieces.into_iter().enumerate() {
            let spelling = text[r.clone()].trim();
            if spelling.is_empty() {
                continue;
            }
            while positions.peek().is_some_and(|(byte, _)| *byte < r.start) {
                positions.next();
                chars += 1;
            }
            let word = Word::new(
                self.tagger.id_word(Cow::Borrowed(spelling)),
                self.tagger
                    .get_tags_with_options(spelling, if i == 0 { Some(true) } else { None }, None)
                    .collect(),
            );
            tokens.push(IncompleteToken::new(
                word,
                Span::new(
                    r.start..r.start + spelling.len(),
                    chars..chars + spelling.chars().count(),
                ),
                i + 1 == count,
                text[..r.start]
                    .chars()
                    .next_back()
                    .is_some_and(char::is_whitespace),
                Vec::new(),
            ));
        }
        let last = tokens.last_mut()?;
        *last.is_sentence_end_mut() = true;
        let mut sentence = IncompleteSentence::new(tokens, text, &self.tagger);
        if let Some(c) = &self.chunker {
            c.apply(&mut sentence);
        }
        if let Some(m) = &self.multiword_tagger {
            m.apply(&mut sentence);
        }
        Some(sentence)
    }
    pub fn disambiguate<'t>(
        &'t self,
        mut sentence: IncompleteSentence<'t>,
    ) -> IncompleteSentence<'t> {
        let mut remaining = self.rules.iter();
        loop {
            let complete = sentence.clone().into_sentence();
            let matching = crate::nlp::annotation::rules::pattern::MatchSentence::new(&complete);
            let Some((rule, changes)) = remaining.find_map(|rule| {
                let changes = rule.apply(&matching);
                (!changes.is_empty()).then_some((rule, changes))
            }) else {
                break;
            };
            rule.change(&mut sentence, changes);
        }
        sentence
    }
    pub fn sentencize<'t>(&'t self, text: &'t str) -> IncompleteSentenceIter<'t> {
        IncompleteSentenceIter {
            tokenizer: self,
            text,
            ranges: self.sentencizer.split_ranges(text).into_iter(),
            position: Position::default(),
        }
    }
    pub fn pipe<'t>(&'t self, text: &'t str) -> SentenceIter<'t> {
        SentenceIter(self.sentencize(text))
    }
}
pub struct IncompleteSentenceIter<'t> {
    tokenizer: &'t Tokenizer,
    text: &'t str,
    ranges: std::vec::IntoIter<Range<usize>>,
    position: Position,
}
impl<'t> Iterator for IncompleteSentenceIter<'t> {
    type Item = IncompleteSentence<'t>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut range = self.ranges.next()?;
        while self.text[range.clone()].trim().is_empty() {
            if let Some(next) = self.ranges.next() {
                range.end = next.end;
            } else {
                break;
            }
        }
        let text = &self.text[range];
        let shift = self.position;
        self.position += Position {
            byte: text.len(),
            char: text.chars().count(),
        };
        self.tokenizer.tokenize(text).map(|s| s.rshift(shift))
    }
}
pub struct SentenceIter<'t>(IncompleteSentenceIter<'t>);
impl<'t> Iterator for SentenceIter<'t> {
    type Item = Sentence<'t>;
    fn next(&mut self) -> Option<Self::Item> {
        let s = self.0.next()?;
        Some(self.0.tokenizer.disambiguate(s).into_sentence())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::nlp::annotation::dictionary::{PosIdInt, TaggerLangOptions, WordIdInt};
    fn tokenizer() -> Tokenizer {
        let mut builder = fst::MapBuilder::memory();
        for (word, id) in [("HELLO", 0), ("a-b", 1), ("hello", 2)] {
            builder.insert(word, id).unwrap();
        }
        let tagger = Tagger::from_parts(
            fst::Map::new(builder.into_inner().unwrap()).unwrap(),
            "HELLOa-bhello".into(),
            vec![(0, 5), (5, 8), (8, 13)],
            vec![
                (
                    WordIdInt::from_value_unchecked(0),
                    PosIdInt::from_value_unchecked(4),
                ),
                (
                    WordIdInt::from_value_unchecked(2),
                    PosIdInt::from_value_unchecked(5),
                ),
            ],
            vec![(0, 1), (0, 0), (1, 2)],
            vec![
                ("UP".into(), PosIdInt::from_value_unchecked(4)),
                ("LOW".into(), PosIdInt::from_value_unchecked(5)),
                ("UNKNOWN".into(), PosIdInt::from_value_unchecked(1)),
            ],
            TaggerLangOptions::default(),
        );
        Tokenizer {
            rules: vec![],
            chunker: None,
            multiword_tagger: None,
            sentencizer: srx::Rules::default(),
            tagger: Arc::new(tagger),
            lang_options: TokenizerLangOptions::default(),
        }
    }
    #[test]
    fn first_required_graph_reference_uses_empty_context() {
        use crate::nlp::annotation::rules::pattern::{Composition, MatchSentence};
        let tokenizer = tokenizer();
        let sentence = tokenizer.tokenize("hello").unwrap().into_sentence();
        let matching = MatchSentence::new(&sentence);
        let composition: Composition = serde_json::from_value(serde_json::json!({
            "parts": [{
                "atom": {"TextAtom": {"matcher": {"matcher": {
                    "matcher": {"Left": {"Right": 1}}, "negate": false,
                    "case_sensitive": true, "empty_always_false": true
                }, "set": null}}},
                "quantifier": {"min": 1, "max": 1}, "greedy": false,
                "visible": true, "unify": null
            }],
            "id_to_idx": {"1": 1}, "can_stop_mask": [false, false]
        }))
        .unwrap();
        // The mapped capture exists only after the required first-atom precheck.
        // An initialized context would silently reject instead of panicking.
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || composition.apply(&matching, 1)
            ))
            .is_err()
        );
    }
    #[test]
    fn changed_rules_refresh_snapshot_and_continue_in_order() {
        use crate::nlp::annotation::rules::{
            DisambiguationRule,
            actions::Disambiguation,
            matcher::{Engine, TokenEngine},
            pattern::*,
        };
        let mut t = tokenizer();
        let make_rule = |from: usize, to: &str| {
            let mut mask = vec![false; 6];
            mask[from] = true;
            DisambiguationRule {
                id: "synthetic".into(),
                engine: Engine::Token(TokenEngine {
                    composition: Composition {
                        parts: vec![Part {
                            atom: Atom::WordDataAtom(WordDataAtom {
                                matcher: WordDataMatcher {
                                    pos_matcher: Some(PosMatcher { mask }),
                                    inflect_matcher: None,
                                },
                                case_sensitive: true,
                            }),
                            quantifier: Quantifier { min: 1, max: 1 },
                            greedy: false,
                            visible: true,
                            unify: None,
                        }],
                        id_to_idx: [(GraphId(1), 1)].into(),
                        can_stop_mask: vec![false, false],
                    },
                    antipatterns: vec![],
                }),
                disambiguations: Disambiguation::Replace(vec![
                    crate::nlp::annotation::types::owned::WordData {
                        lemma: t.tagger.id_word(Cow::Borrowed("hello")).into(),
                        pos: t.tagger.id_tag(to).into(),
                    },
                ]),
                start: GraphId(1),
                end: GraphId(1),
                filter: None,
                unification: None,
            }
        };
        t.rules = vec![
            make_rule(4, "LOW"),
            make_rule(5, "UP"),
            make_rule(5, "LOW"),
            make_rule(4, "LOW"),
        ];
        let s = t.disambiguate(t.tokenize("hello").unwrap());
        let tags = s.iter().next().unwrap().word().tags();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].pos().as_str(), "LOW");
    }
    #[test]
    fn engine_matches_are_lazy_and_not_capped_at_fixture_limit() {
        use crate::nlp::annotation::rules::{
            matcher::{Engine, TokenEngine},
            pattern::*,
        };
        let t = tokenizer();
        let text = "hello ".repeat(1205);
        let sentence = t.tokenize(&text).unwrap().into_sentence();
        let matching = MatchSentence::new(&sentence);
        let engine = Engine::Token(TokenEngine {
            composition: Composition {
                parts: vec![Part {
                    atom: Atom::OffsetAtom(OffsetAtom {
                        atom: Box::new(Atom::TrueAtom(TrueAtom {})),
                        offset: -1,
                    }),
                    quantifier: Quantifier { min: 1, max: 1 },
                    greedy: false,
                    visible: true,
                    unify: None,
                }],
                id_to_idx: [(GraphId(1), 1)].into(),
                can_stop_mask: vec![false, false],
            },
            antipatterns: vec![],
        });
        assert_eq!(
            engine
                .get_matches(&matching, GraphId(1), GraphId(1))
                .count(),
            1205
        );
    }
    fn surfaces<'a>(s: &IncompleteSentence<'a>) -> Vec<String> {
        s.iter().map(|t| t.word().as_str().to_owned()).collect()
    }
    #[test]
    fn unicode_separators_and_coordinates() {
        let t = tokenizer();
        let s = t.tokenize("\u{a0}é\u{301}!\u{2003}猫\u{200b}x").unwrap();
        assert_eq!(surfaces(&s), ["é\u{301}", "!", "猫\u{200b}x"]);
        let tokens = s.iter().collect::<Vec<_>>();
        assert_eq!(tokens[0].span(), &Span::new(2..6, 1..3));
        assert_eq!(tokens[1].span(), &Span::new(6..7, 3..4));
        assert_eq!(tokens[2].span(), &Span::new(10..17, 5..8));
        assert!(tokens[0].has_space_before());
        assert!(tokens[2].has_space_before());
        assert!(tokens[2].is_sentence_end());
    }
    #[test]
    fn dictionary_protects_only_secondary_split() {
        let mut t = tokenizer();
        t.lang_options.extra_split_chars = vec!['-'];
        assert_eq!(
            surfaces(&t.tokenize("a-b c-d a/b").unwrap()),
            ["a-b", "c", "-", "d", "a", "/", "b"]
        );
    }
    #[test]
    fn lowercase_depends_on_original_piece_index() {
        let t = tokenizer();
        assert_eq!(
            t.tokenize("HELLO")
                .unwrap()
                .iter()
                .next()
                .unwrap()
                .word()
                .tags()
                .len(),
            2
        );
        assert_eq!(
            t.tokenize(" HELLO")
                .unwrap()
                .iter()
                .next()
                .unwrap()
                .word()
                .tags()
                .len(),
            1
        );
    }
    #[test]
    fn joins_claim_bytes_in_configuration_order() {
        let mut t = tokenizer();
        t.lang_options.extra_join_regexes =
            vec![Regex::new("b c".into()), Regex::new("a b".into())];
        assert_eq!(surfaces(&t.tokenize("a b c").unwrap()), ["a", "b c"]);
    }
    #[test]
    fn empty_joins_claim_nothing_and_keep_source_gaps() {
        let mut t = tokenizer();
        t.lang_options.extra_join_regexes = vec![Regex::new("".into()), Regex::new("a b".into())];
        assert_eq!(surfaces(&t.tokenize("a b").unwrap()), ["a b"]);
    }
    #[test]
    fn trimmed_join_retains_untrimmed_start() {
        let mut t = tokenizer();
        t.lang_options.extra_join_regexes = vec![Regex::new(" +é +".into())];
        let s = t.tokenize("  é  ").unwrap();
        let token = s.iter().next().unwrap();
        assert_eq!(token.word().as_str(), "é");
        assert_eq!(token.span(), &Span::new(0..2, 0..1));
        assert!(!token.has_space_before());
    }
    #[test]
    fn sentence_offsets_include_preceding_whitespace() {
        let mut t = tokenizer();
        #[derive(Serialize)]
        struct Srx {
            rules: Vec<SrxRule>,
        }
        #[derive(Serialize)]
        struct SrxRule {
            regex: String,
            do_break: bool,
        }
        t.sentencizer = bincode::deserialize(
            &bincode::serialize(&Srx {
                rules: vec![SrxRule {
                    regex: "[.!]( +)".into(),
                    do_break: true,
                }],
            })
            .unwrap(),
        )
        .unwrap();
        let sentences = t.sentencize("é.  猫!").collect::<Vec<_>>();
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].text(), "é.");
        assert_eq!(sentences[1].text(), "  猫!");
        assert_eq!(sentences[1].span(), &Span::new(3..9, 2..6));
        assert_eq!(
            sentences[1].iter().next().unwrap().span(),
            &Span::new(5..8, 4..5)
        );
    }
    #[test]
    fn iterator_none_does_not_fuse() {
        let t = tokenizer();
        let text = "  猫";
        let mut iter = IncompleteSentenceIter {
            tokenizer: &t,
            text,
            ranges: vec![0..1, 1..2, 2..2, 2..5].into_iter(),
            position: Position::default(),
        }; // whitespace merges through zero-width, then nonwhite
        assert_eq!(iter.next().unwrap().text(), text);
        assert!(iter.next().is_none());
        // Exercise an empty final range followed by another call using a manually supplied range sequence.
        let mut iter = IncompleteSentenceIter {
            tokenizer: &t,
            text: "猫",
            ranges: vec![0..0, 0..0].into_iter(),
            position: Position::default(),
        };
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }
    #[test]
    fn finalization_and_native_only_marker() {
        let t = tokenizer();
        let bytes = t.compile_asset().unwrap();
        let loaded = Tokenizer::from_compiled(&bytes).unwrap();
        assert_eq!(
            loaded
                .pipe("?")
                .next()
                .unwrap()
                .iter()
                .next()
                .unwrap()
                .word()
                .tags()
                .iter()
                .map(|d| d.pos().as_str())
                .collect::<Vec<_>>(),
            ["", "UNKNOWN", "SENT_END"]
        );
        for marker in [b"LINN-NLP-3\0".as_slice(), b"LINN-NLP-4", b""] {
            assert!(Tokenizer::from_compiled(marker).is_err());
        }
    }
}
