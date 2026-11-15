pub mod actions;
pub mod filter;
pub mod matcher;
pub mod pattern;
use crate::nlp::annotation::{
    rules::filter::Filter,
    types::{IncompleteSentence, Span},
};
use actions::{Disambiguation, Unification};
use matcher::Engine;
use pattern::{GraphId, MatchSentence};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisambiguationRule {
    pub id: String,
    pub engine: Engine,
    pub disambiguations: Disambiguation,
    pub filter: Option<Filter>,
    pub start: GraphId,
    pub end: GraphId,
    pub unification: Option<Unification>,
}
#[derive(Default, Debug)]
pub struct Changes(Vec<Vec<Vec<Span>>>);
impl Changes {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
impl DisambiguationRule {
    pub fn apply<'t>(&'t self, sentence: &MatchSentence<'t>) -> Changes {
        if matches!(self.disambiguations, Disambiguation::Nop) {
            return Changes::default();
        }
        Changes(
            self.engine
                .get_matches(sentence, self.start, self.end)
                .filter_map(|g| {
                    if self
                        .unification
                        .as_ref()
                        .is_some_and(|u| !u.keep(&g, sentence))
                        || self.filter.as_ref().is_some_and(|f| !f.keep(sentence, &g))
                    {
                        return None;
                    }
                    Some(
                        self.start
                            .range(&self.end)
                            .map(|id| {
                                g.by_id(id)
                                    .tokens(sentence)
                                    .map(|t| t.span().clone())
                                    .collect()
                            })
                            .collect(),
                    )
                })
                .collect(),
        )
    }
    pub fn change<'t>(&'t self, sentence: &mut IncompleteSentence<'t>, changes: Changes) {
        for spans in changes.0 {
            let mut groups: Vec<Vec<_>> = (0..spans.len()).map(|_| Vec::new()).collect();
            for token in sentence.iter_mut() {
                if let Some(i) = spans.iter().position(|s| s.contains(token.span())) {
                    groups[i].push(token);
                }
            }
            self.disambiguations.apply(groups);
        }
    }
}
