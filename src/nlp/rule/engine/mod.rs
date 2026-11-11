// Portions derived from nlprule 0.6.4. See src/nlp/LICENSE.
use crate::nlp::{
    types::*,
    utils::regex::{CaptureMatches, Regex},
};
use serde::{Deserialize, Serialize};
pub mod composition;

use composition::{Composition, GraphId, Group, MatchGraph, MatchSentence};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenEngine {
    pub(crate) composition: Composition,
    pub(crate) antipatterns: Vec<Composition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Engine {
    Token(TokenEngine),
    // regex with the `fancy_regex` backend is large on the stack
    Text(
        Box<Regex>,
        #[serde(serialize_with = "crate::nlp::utils::asset::map")] DefaultHashMap<GraphId, usize>,
    ),
}

struct TokenMatches<'a> {
    engine: &'a TokenEngine,
    index: usize,
    mask: Vec<bool>,
    antipattern_ranges: Option<Vec<(usize, usize)>>,
}

impl<'a> TokenMatches<'a> {
    fn get_match<'t>(&mut self, sentence: &'t MatchSentence, i: usize) -> Option<MatchGraph<'t>>
    where
        'a: 't,
    {
        let graph = self.engine.composition.apply(sentence, i)?;
        let ranges = self.antipattern_ranges.get_or_insert_with(|| {
            let mut ranges = Vec::new();
            for position in 0..sentence.len() {
                for pattern in &self.engine.antipatterns {
                    if let Some(g) = pattern.apply(sentence, position) {
                        ranges.push((
                            g.by_index(0).span.char().start,
                            g.by_index(g.groups().len() - 1).span.char().end,
                        ));
                    }
                }
            }
            ranges.sort_unstable_by_key(|r| r.0);
            let mut end = 0;
            for range in &mut ranges {
                end = end.max(range.1);
                range.1 = end;
            }
            ranges
        });
        let start = graph.by_index(0).span.char().start;
        let end = graph.by_index(graph.groups().len() - 1).span.char().end;
        let cut = ranges.partition_point(|range| range.0 <= end);
        if cut > 0 && ranges[cut - 1].1 >= start {
            None
        } else {
            Some(graph)
        }
    }
}

struct TextMatches<'a, 't> {
    byte_idx_to_char_idx: DefaultHashMap<usize, usize>,
    id_to_idx: &'a DefaultHashMap<GraphId, usize>,
    captures: CaptureMatches<'a, 't>,
}

enum InnerMatches<'a: 't, 't> {
    Token(TokenMatches<'a>),
    Text(TextMatches<'a, 't>),
}

pub struct EngineMatches<'a, 't> {
    sentence: &'t MatchSentence<'t>,
    start: GraphId,
    end: GraphId,
    inner: InnerMatches<'a, 't>,
}

impl<'a, 't> Iterator for EngineMatches<'a, 't> {
    type Item = MatchGraph<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        let sentence = self.sentence;
        let start_id = self.start;
        let end_id = self.end;

        match &mut self.inner {
            InnerMatches::Token(inner) => (inner.index..sentence.len()).find_map(|i| {
                inner.get_match(sentence, i).and_then(|graph| {
                    let start_group = graph.by_id(start_id);
                    let end_group = graph.by_id(end_id);

                    let start = start_group.span.char().start - sentence.span().char().start;
                    let end = end_group.span.char().end - sentence.span().char().start;

                    if inner.mask[start..end].iter().all(|x| !x) {
                        inner.mask[start..end].iter_mut().for_each(|x| *x = true);

                        // Earlier positions failed or overlap the monotonically
                        // growing mask. A nonempty accepted span also masks itself.
                        // Preserve the repeated-zero-width behavior explicitly.
                        inner.index = if start < end { i + 1 } else { inner.index + 1 };
                        Some(graph)
                    } else {
                        None
                    }
                })
            }),
            InnerMatches::Text(inner) => inner.captures.next().map(|captures| {
                let bi_to_ci = &inner.byte_idx_to_char_idx;
                let mut groups = Vec::new();

                for group in captures.iter() {
                    if let Some(group) = group {
                        let byte_span = group.start()..group.end();

                        let char_start = *bi_to_ci
                            .get(&byte_span.start)
                            .expect("byte index is at char boundary");
                        let char_end = *bi_to_ci
                            .get(&byte_span.end)
                            .expect("byte index is at char boundary");

                        groups.push(Group::new(
                            Span::new(byte_span, char_start..char_end)
                                .rshift(sentence.span().start()),
                        ));
                    } else {
                        groups.push(Group::new(Span::default()));
                    }
                }

                MatchGraph::new(groups, inner.id_to_idx)
            }),
        }
    }
}

impl Engine {
    pub fn get_matches<'a, 't>(
        &'a self,
        sentence: &'t MatchSentence,
        start: GraphId,
        end: GraphId,
    ) -> EngineMatches<'a, 't> {
        let inner = match &self {
            Engine::Token(engine) => InnerMatches::Token(TokenMatches {
                engine,
                index: 0,
                mask: vec![false; sentence.text().chars().count()],
                antipattern_ranges: None,
            }),
            Engine::Text(regex, id_to_idx) => {
                let mut bi_to_ci: DefaultHashMap<usize, usize> = sentence
                    .text()
                    .char_indices()
                    .enumerate()
                    .map(|(ci, (bi, _))| (bi, ci))
                    .collect();
                bi_to_ci.insert(sentence.text().len(), bi_to_ci.len());

                InnerMatches::Text(TextMatches {
                    byte_idx_to_char_idx: bi_to_ci,
                    id_to_idx,
                    captures: regex.captures_iter(sentence.text()),
                })
            }
        };

        EngineMatches {
            sentence,
            start,
            end,
            inner,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use composition::{Atom, Matcher, Part, Quantifier, TextMatcher, concrete::TextAtom};

    fn literal(text: &str) -> Composition {
        Composition {
            parts: vec![Part {
                atom: Atom::TextAtom(TextAtom {
                    matcher: TextMatcher {
                        matcher: Matcher {
                            matcher: either::Left(either::Left(text.into())),
                            negate: false,
                            case_sensitive: true,
                            empty_always_false: true,
                        },
                        set: None,
                    },
                }),
                quantifier: Quantifier { min: 1, max: 1 },
                greedy: false,
                visible: true,
                unify: None,
            }],
            id_to_idx: [(GraphId(0), 0), (GraphId(1), 1)].into(),
            can_stop_mask: vec![false, true],
        }
    }

    #[test]
    fn cursor_and_exclusion_semantics() {
        let tokenizer = crate::nlp::Tokenizer::from_compiled(include_bytes!(concat!(
            env!("OUT_DIR"),
            "/english.nlp"
        )))
        .unwrap();
        let sentence = tokenizer.pipe("one two cache three cache").next().unwrap();
        let sentence = MatchSentence::new(&sentence);
        let engine = Engine::Token(TokenEngine {
            composition: literal("cache"),
            antipatterns: vec![],
        });
        let spans: Vec<_> = engine
            .get_matches(&sentence, GraphId(1), GraphId(1))
            .map(|g| g.by_id(GraphId(1)).span.char().clone())
            .collect();
        assert_eq!(spans, [8..13, 20..25]);
        let zeroes: Vec<_> = engine
            .get_matches(&sentence, GraphId(0), GraphId(0))
            .map(|g| g.by_id(GraphId(0)).span.char().clone())
            .collect();
        assert_eq!(zeroes, [8..8, 8..8, 8..8, 8..8, 20..20, 20..20]);

        let sentence = tokenizer.pipe("cache,cache").next().unwrap();
        let sentence = MatchSentence::new(&sentence);
        let engine = Engine::Token(TokenEngine {
            composition: literal(","),
            antipatterns: vec![literal("cache")],
        });
        assert_eq!(
            engine
                .get_matches(&sentence, GraphId(1), GraphId(1))
                .count(),
            0
        );
    }
}
