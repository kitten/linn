use super::pattern::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEngine {
    pub composition: Composition,
    pub antipatterns: Vec<Composition>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Engine {
    Token(TokenEngine),
}
pub struct EngineMatches<'a, 't>
where
    'a: 't,
{
    engine: &'a TokenEngine,
    sentence: &'t MatchSentence<'t>,
    start: GraphId,
    end: GraphId,
    mask: Vec<bool>,
    cursor: usize,
    anti: Option<Vec<(usize, usize)>>,
}
impl<'a, 't> Iterator for EngineMatches<'a, 't>
where
    'a: 't,
{
    type Item = MatchGraph<'t>;
    fn next(&mut self) -> Option<Self::Item> {
        let s = self.sentence;
        let anti = self.anti.get_or_insert_with(|| {
            let mut ranges: Vec<_> = self
                .engine
                .antipatterns
                .iter()
                .flat_map(|a| (0..s.len()).filter_map(|p| a.apply(s, p)))
                .map(|g| {
                    (
                        g.groups()[0].span.char().start,
                        g.groups().last().unwrap().span.char().end,
                    )
                })
                .collect();
            ranges.sort_unstable_by_key(|r| r.0);
            let mut max_end = 0;
            for (_, end) in &mut ranges {
                max_end = max_end.max(*end);
                *end = max_end;
            }
            ranges
        });
        for p in self.cursor..s.len() {
            if let Some(g) = self.engine.composition.apply(s, p) {
                let a = g.by_id(self.start).span.char().start;
                let b = g.by_id(self.end).span.char().end;
                let full_a = g.groups()[0].span.char().start;
                let full_b = g.groups().last().unwrap().span.char().end;
                if (a..b).any(|i| self.mask[i - s.span().char().start]) || {
                    let i = anti.partition_point(|(start, _)| *start <= full_b);
                    i > 0 && anti[i - 1].1 >= full_a
                } {
                    continue;
                }
                for i in a..b {
                    self.mask[i - s.span().char().start] = true;
                }
                self.cursor = if a < b { p + 1 } else { self.cursor + 1 };
                return Some(g);
            }
        }
        None
    }
}
impl Engine {
    pub fn get_matches<'a, 't>(
        &'a self,
        s: &'t MatchSentence<'t>,
        start: GraphId,
        end: GraphId,
    ) -> EngineMatches<'a, 't>
    where
        'a: 't,
    {
        let Self::Token(e) = self;
        EngineMatches {
            engine: e,
            sentence: s,
            start,
            end,
            mask: vec![false; s.span().char().end - s.span().char().start],
            cursor: 0,
            anti: None,
        }
    }
}
