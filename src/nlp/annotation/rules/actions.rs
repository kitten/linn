use super::pattern::{MatchGraph, MatchSentence, PosMatcher};
use crate::nlp::annotation::types::{IncompleteToken, Word, WordData, owned};
use either::Either;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosFilter {
    pub matcher: PosMatcher,
}
impl PosFilter {
    pub fn and(filters: &[&Self], word: &Word) -> bool {
        word.tags()
            .iter()
            .any(|d| filters.iter().all(|f| f.matcher.is_match(d.pos())))
    }
    pub fn apply(filters: &[Vec<&Self>], word: &mut Word) {
        word.retain(|d| {
            filters
                .iter()
                .any(|fs| fs.iter().all(|f| f.matcher.is_match(d.pos())))
        });
    }
}
pub(crate) fn products(filters: &[Vec<PosFilter>]) -> Vec<Vec<&PosFilter>> {
    if filters.is_empty() {
        return vec![];
    }
    let mut out = vec![vec![]];
    for dimension in filters {
        out = out
            .into_iter()
            .flat_map(|prefix| {
                dimension.iter().map(move |f| {
                    let mut next = prefix.clone();
                    next.push(f);
                    next
                })
            })
            .collect();
    }
    out
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unification {
    pub mask: Vec<Option<bool>>,
    pub filters: Vec<Vec<PosFilter>>,
}
impl Unification {
    pub fn keep(&self, graph: &MatchGraph, sentence: &MatchSentence) -> bool {
        let mut viable = products(&self.filters);
        for (group, mask) in graph.groups().iter().skip(1).zip(&self.mask) {
            if mask.is_some() {
                for token in group.tokens(sentence) {
                    viable.retain(|fs| PosFilter::and(fs, token.word()));
                }
            }
        }
        !viable.is_empty() != self.mask.iter().all(|m| !m.unwrap_or(false))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Disambiguation {
    Remove(Vec<Either<owned::WordData, PosFilter>>),
    Add(Vec<owned::WordData>),
    Replace(Vec<owned::WordData>),
    Filter(Vec<Option<Either<owned::WordData, PosFilter>>>, bool),
    Unify(Vec<Vec<PosFilter>>, Vec<Option<PosFilter>>, Vec<bool>),
    Nop,
}
fn data<'t>(d: &'t owned::WordData, w: &Word<'t>) -> WordData<'t> {
    WordData::new(
        if d.lemma.as_ref().is_empty() {
            w.text().clone()
        } else {
            (&d.lemma).into()
        },
        (&d.pos).into(),
    )
}
impl Disambiguation {
    pub fn apply<'t>(&'t self, groups: Vec<Vec<&mut IncompleteToken<'t>>>) {
        match self {
            Self::Nop => {}
            Self::Add(ds) | Self::Replace(ds) => {
                for (ts, d) in groups.into_iter().zip(ds) {
                    for t in ts {
                        let w = t.word_mut();
                        let next = data(d, w);
                        if matches!(self, Self::Replace(_)) {
                            w.clear();
                        }
                        w.push(next);
                        if matches!(self, Self::Add(_)) {
                            w.retain(|x| !x.pos().as_str().is_empty());
                        }
                    }
                }
            }
            Self::Remove(ds) => {
                for (ts, d) in groups.into_iter().zip(ds) {
                    for t in ts {
                        t.word_mut().retain(|x| !match d {
                            Either::Left(d) => {
                                x.pos().id() == d.pos.as_ref_id().id()
                                    && (d.lemma.as_ref().is_empty()
                                        || x.lemma() == &d.lemma.as_ref_id())
                            }
                            Either::Right(f) => f.matcher.is_match(x.pos()),
                        });
                    }
                }
            }
            Self::Filter(ds, last) => {
                for (ts, d) in groups.into_iter().zip(ds) {
                    if let Some(d) = d {
                        for t in ts {
                            let w = t.word_mut();
                            match d {
                                Either::Right(f) => w.retain(|x| f.matcher.is_match(x.pos())),
                                Either::Left(d) => {
                                    let lemma = if *last {
                                        w.tags()
                                            .first()
                                            .map(|d| d.lemma())
                                            .unwrap_or(w.text())
                                            .clone()
                                    } else {
                                        w.text().clone()
                                    };
                                    w.retain(|x| x.pos().id() == d.pos.as_ref_id().id());
                                    if w.tags().is_empty() {
                                        w.push(WordData::new(lemma, (&d.pos).into()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Self::Unify(fs, ds, mask) => {
                let mut viable = products(fs);
                for (ts, active) in groups.iter().zip(mask) {
                    if *active {
                        for t in ts {
                            let complete = (**t).clone().into_token();
                            viable.retain(|fs| PosFilter::and(fs, complete.word()));
                        }
                    }
                }
                if viable.is_empty() {
                    return;
                }
                for ((ts, d), active) in groups.into_iter().zip(ds).zip(mask) {
                    if *active {
                        for t in ts {
                            let w = t.word_mut();
                            let old = w.clone();
                            PosFilter::apply(&viable, w);
                            if let Some(f) = d {
                                w.retain(|x| f.matcher.is_match(x.pos()));
                            }
                            if w.tags().is_empty() {
                                *w = old;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn products_with_no_dimensions_or_an_empty_dimension_are_empty() {
        assert!(products(&[]).is_empty());
        let filter = || PosFilter {
            matcher: PosMatcher { mask: vec![true] },
        };
        assert!(products(&[vec![filter()], vec![], vec![filter()]]).is_empty());
        assert_eq!(
            products(&[vec![filter(), filter()], vec![filter()]]).len(),
            2
        );
    }
}
