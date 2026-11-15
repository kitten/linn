use crate::nlp::annotation::types::{IncompleteSentence, WordData, owned};
use aho_corasick::AhoCorasick;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
#[derive(Clone, Deserialize)]
#[serde(from = "MultiwordTaggerFields")]
pub struct MultiwordTagger {
    multiwords: Vec<(String, owned::PosId)>,
    matcher: AhoCorasick,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct MultiwordTaggerFields {
    pub(crate) multiwords: Vec<(String, owned::PosId)>,
}
impl From<MultiwordTaggerFields> for MultiwordTagger {
    fn from(fields: MultiwordTaggerFields) -> Self {
        let matcher = AhoCorasick::new_auto_configured(
            &fields
                .multiwords
                .iter()
                .map(|x| x.0.as_str())
                .collect::<Vec<_>>(),
        );
        Self {
            multiwords: fields.multiwords,
            matcher,
        }
    }
}
impl Serialize for MultiwordTagger {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut fields = serializer.serialize_struct("MultiwordTaggerFields", 1)?;
        fields.serialize_field("multiwords", &self.multiwords)?;
        fields.end()
    }
}
impl MultiwordTagger {
    pub fn apply<'t>(&'t self, sentence: &mut IncompleteSentence<'t>) {
        let count = sentence.len();
        let bytes = sentence
            .iter()
            .map(|t| t.word().as_str().len())
            .sum::<usize>()
            + count.saturating_sub(1);
        let mut joined = String::with_capacity(bytes);
        let mut starts = Vec::with_capacity(count);
        let mut ends = Vec::with_capacity(count);
        for (i, token) in sentence.iter().enumerate() {
            if i != 0 {
                joined.push(' ');
            }
            starts.push(joined.len());
            joined.push_str(token.word().as_str());
            ends.push(joined.len());
        }
        let tagger = sentence.tagger();
        for found in self.matcher.find_iter(&joined) {
            if let (Ok(start), Ok(end)) = (
                starts.binary_search(&found.start()),
                ends.binary_search(&found.end()),
            ) {
                let (phrase, pos) = &self.multiwords[found.pattern()];
                let mut data =
                    WordData::new(tagger.id_word(Cow::Borrowed(phrase)), pos.as_ref_id());
                data.freeze();
                for token in sentence.iter_mut().skip(start).take(end + 1 - start) {
                    token.word_mut().push(data.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn multiword_boundaries_and_wire() {
        use crate::nlp::annotation::{
            dictionary::{PosIdInt, Tagger, TaggerLangOptions, owned},
            multiword::{MultiwordTagger, MultiwordTaggerFields},
            types::{IncompleteSentence, IncompleteToken, Span, Word, WordData},
        };
        use std::borrow::Cow;
        let tagger = Tagger::from_parts(
            fst::Map::from_iter(std::iter::empty::<(&str, u64)>()).unwrap(),
            String::new(),
            vec![],
            vec![],
            vec![],
            vec![("OLD".into(), PosIdInt::from_value_unchecked(20))],
            TaggerLangOptions::default(),
        );
        type Scenario<'a> = (Vec<&'a str>, Vec<&'a str>, Vec<Vec<&'a str>>);
        let scenarios: Vec<Scenario<'_>> = vec![
            (
                vec!["a b", "a", "b"],
                vec!["a", "b"],
                vec![vec!["a"], vec!["b"]],
            ),
            (
                vec!["a b", "b c"],
                vec!["a", "b", "c"],
                vec![vec!["a b"], vec!["a b"], vec![]],
            ),
            (
                vec!["é 中", "中"],
                vec!["é", "中"],
                vec![vec!["é 中"], vec!["é 中"]],
            ),
            (vec!["a"], vec!["cat"], vec![vec![]]),
            (
                vec!["a  b"],
                vec!["a", "", "b"],
                vec![vec!["a  b"], vec!["a  b"], vec!["a  b"]],
            ),
            (vec!["a", "a"], vec!["a"], vec![vec!["a"]]),
        ];
        for (phrases, surfaces, expected) in scenarios {
            let fields = MultiwordTaggerFields {
                multiwords: phrases
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        (
                            p.to_string(),
                            owned::PosId(format!("P{i}"), PosIdInt::from_value_unchecked(i as u16)),
                        )
                    })
                    .collect(),
            };
            let bytes = bincode::serialize(&fields).unwrap();
            let matcher: MultiwordTagger = bincode::deserialize(&bytes).unwrap();
            assert_eq!(bytes, bincode::serialize(&matcher).unwrap());
            let tokens = surfaces
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let mut old =
                        WordData::new(tagger.id_word(Cow::Borrowed("old")), tagger.id_tag("OLD"));
                    old.freeze();
                    IncompleteToken::new(
                        Word::new(tagger.id_word(Cow::Borrowed(s)), vec![old.clone(), old]),
                        Span::new(i..i + 1, i..i + 1),
                        i == 0,
                        i % 2 == 0,
                        vec!["NP".into()],
                    )
                })
                .collect();
            let mut sentence = IncompleteSentence::new(tokens, "unrelated source spacing", &tagger);
            let before: Vec<_> = sentence.iter().cloned().collect();
            matcher.apply(&mut sentence);
            for ((token, original), want) in sentence.iter().zip(&before).zip(expected) {
                assert_eq!(token.span(), original.span());
                assert_eq!(token.chunks(), original.chunks());
                assert_eq!(token.is_sentence_end(), original.is_sentence_end());
                assert_eq!(token.has_space_before(), original.has_space_before());
                assert_eq!(token.word().text(), original.word().text());
                assert_eq!(&token.word().tags()[..2], original.word().tags());
                assert_eq!(
                    token.word().tags()[2..]
                        .iter()
                        .map(|t| t.lemma().as_str())
                        .collect::<Vec<_>>(),
                    want
                );
                assert!(token.word().tags().iter().all(|t| t.frozen()));
                for t in &token.word().tags()[2..] {
                    assert!(t.pos().as_str().starts_with('P'));
                }
            }
        }
    }
}
