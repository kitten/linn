use crate::nlp::annotation::{
    regex::Regex,
    rules::pattern::{GraphId, MatchGraph, MatchSentence},
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Filter {
    NoDisambiguationEnglishPartialPosTagFilter {
        id: GraphId,
        regexp: Regex,
        postag_regexp: Regex,
    },
}
impl Filter {
    pub fn keep(&self, sentence: &MatchSentence, graph: &MatchGraph) -> bool {
        let Self::NoDisambiguationEnglishPartialPosTagFilter {
            id,
            regexp,
            postag_regexp,
        } = self;
        graph.by_id(*id).tokens(sentence).all(|t| {
            let Some(captures) = regexp.captures(t.word().as_str()) else {
                return false;
            };
            let text = captures.get(1).expect("capture one").as_str();
            sentence
                .tagger()
                .get_tags(text)
                .any(|d| postag_regexp.is_match(d.pos().as_str()))
        })
    }
}
