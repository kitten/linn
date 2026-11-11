// Portions derived from nlprule 0.6.4. See src/nlp/LICENSE.
//! Implementations related to single rules.

use crate::nlp::filter::{Filter, Filterable};
use crate::nlp::types::*;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub(crate) mod disambiguation;
pub(crate) mod engine;
pub mod id;

use engine::Engine;

pub(crate) use engine::composition::{MatchGraph, MatchSentence};

use self::{disambiguation::PosFilter, engine::composition::GraphId, id::Index};

/// A *Unification* makes an otherwise matching pattern invalid if no combination of its filters
/// matches all tokens marked with "unify".
/// Can also be negated.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Unification {
    pub(crate) mask: Vec<Option<bool>>,
    pub(crate) filters: Vec<Vec<PosFilter>>,
}

impl Unification {
    pub fn keep(&self, graph: &MatchGraph, sentence: &MatchSentence) -> bool {
        let filters: Vec<_> = self.filters.iter().multi_cartesian_product().collect();

        let mut filter_mask: Vec<_> = filters.iter().map(|_| true).collect();
        let negate = self.mask.iter().all(|x| x.is_none_or(|x| !x));

        for (group, maybe_mask_val) in graph.groups()[1..].iter().zip(self.mask.iter()) {
            if maybe_mask_val.is_some() {
                for token in group.tokens(sentence) {
                    for (mask_val, filter) in filter_mask.iter_mut().zip(filters.iter()) {
                        *mask_val = *mask_val && PosFilter::and(filter, token.word());
                    }
                }
            }
        }

        let result = filter_mask.iter().any(|x| *x);
        if negate { !result } else { result }
    }
}

/// A disambiguation rule.
/// Changes the information associcated with one or more tokens if it matches.
/// Sourced from LanguageTool. An example of how a simple rule might look in the original XML format:
///
/// ```xml
/// <rule id="NODT_HAVE" name="no determiner + have as verb/noun ->have/verb">
///    <pattern>
///    <token>
///         <exception postag="PRP$"></exception>
///         <exception regexp="yes">the|a</exception>
///    </token>
///    <marker>
///        <token case_sensitive="yes" regexp="yes">[Hh]ave|HAVE</token>
///    </marker>
///    </pattern>
///    <disambig action="replace"><wd lemma="have" pos="VB"></wd></disambig>
/// </rule>
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisambiguationRule {
    pub(crate) id: Index,
    pub(crate) engine: Engine,
    pub(crate) disambiguations: disambiguation::Disambiguation,
    pub(crate) filter: Option<Filter>,
    pub(crate) start: GraphId,
    pub(crate) end: GraphId,
    pub(crate) examples: Vec<disambiguation::DisambiguationExample>,
    pub(crate) unification: Option<Unification>,
}

#[derive(Default)]
pub(crate) struct Changes(Vec<Vec<HashSet<Span>>>);

// This is only used in tests at the moment.
// Could maybe be made generic.
impl Changes {}

impl Changes {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl DisambiguationRule {
    pub(crate) fn apply<'t>(&'t self, sentence: &MatchSentence<'t>) -> Changes {
        if matches!(self.disambiguations, disambiguation::Disambiguation::Nop) {
            return Changes::default();
        }

        let mut all_spans = Vec::new();

        for graph in self.engine.get_matches(sentence, self.start, self.end) {
            if let Some(unification) = &self.unification
                && !unification.keep(&graph, sentence)
            {
                continue;
            }

            if let Some(filter) = &self.filter
                && !filter.keep(sentence, &graph)
            {
                continue;
            }

            let mut spans = Vec::new();

            for group_idx in GraphId::range(&self.start, &self.end) {
                let group = graph.by_id(group_idx);

                let group_spans: HashSet<_> =
                    group.tokens(sentence).map(|x| x.span().clone()).collect();

                spans.push(group_spans);
            }

            all_spans.push(spans);
        }

        Changes(all_spans)
    }

    pub(crate) fn change<'t>(&'t self, sentence: &mut IncompleteSentence<'t>, changes: Changes) {
        log::info!("applying {}", self.id);

        for spans in changes.0 {
            let mut groups = Vec::new();
            let mut refs = sentence.iter_mut().collect::<Vec<_>>();

            for group_spans in spans {
                let mut group = Vec::new();

                while let Some(i) = refs.iter().position(|x| group_spans.contains(x.span())) {
                    group.push(refs.remove(i));
                }

                groups.push(group);
            }

            self.disambiguations.apply(groups);
        }
    }
}
