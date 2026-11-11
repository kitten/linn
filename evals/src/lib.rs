//! Corpus-free experimental occurrence analysis; never consulted by product search.
//! @ref LLP 0005#experimental-occurrence-audit
use nlprule::Tokenizer;
use serde::Serialize;
use std::ops::Range;

#[derive(Debug, Serialize)]
pub struct Occurrence {
    pub sentence: usize,
    pub span: Range<usize>,
    pub analyses: Vec<(String, String)>,
    pub chunks: Vec<String>,
}
#[derive(Debug, Serialize)]
pub struct Group {
    pub alternatives: Vec<String>,
    pub span: Range<usize>,
    pub tokens: Range<usize>,
    pub kind: String,
}
#[derive(Debug, Serialize)]
pub struct Analysis {
    pub occurrences: Vec<Occurrence>,
    pub groups: Vec<Group>,
}

/// Analyze raw bytes. Sentences and caller-supplied source units are hard boundaries.
/// Enumerated groups retain punctuation and every intervening original byte.
pub fn analyze(tokenizer: &Tokenizer, text: &str) -> Analysis {
    let mut out = Analysis {
        occurrences: vec![],
        groups: vec![],
    };
    for (sentence_id, sentence) in tokenizer.pipe(text).enumerate() {
        let base = out.occurrences.len();
        for token in sentence.tokens() {
            let span = token.span().byte().clone();
            assert_eq!(text.get(span.clone()), Some(token.word().text().as_str()));
            out.occurrences.push(Occurrence {
                sentence: sentence_id,
                span,
                analyses: token
                    .word()
                    .tags()
                    .iter()
                    .map(|a| (a.lemma().as_str().to_owned(), a.pos().as_str().to_owned()))
                    .collect(),
                chunks: token.chunks().iter().map(|s| s.to_string()).collect(),
            });
        }
        let end = out.occurrences.len();
        // Linear alternatives only: singleton or adjacent subgroup. Whole chunks
        // below are alternate segmentations, never additional obligations.
        for a in base..end {
            for b in a + 1..=(a + 2).min(end) {
                out.groups.push(Group {
                    alternatives: vec!["contiguous".into()],
                    span: out.occurrences[a].span.start..out.occurrences[b - 1].span.end,
                    tokens: a..b,
                    kind: "contiguous".into(),
                });
            }
        }
        // Preserve every begin/end alternative instead of inventing dependency heads.
        for a in base..end {
            for label in &out.occurrences[a].chunks {
                if let Some(kind) = label.strip_prefix("B-") {
                    let mut last = a;
                    while last + 1 < end
                        && out.occurrences[last + 1]
                            .chunks
                            .iter()
                            .any(|s| s == &format!("I-{kind}") || s == &format!("E-{kind}"))
                    {
                        last += 1;
                        if out.occurrences[last]
                            .chunks
                            .iter()
                            .any(|s| s == &format!("E-{kind}"))
                        {
                            break;
                        }
                    }
                    // NP labels have explicit ends; other chunk kinds use BIO.
                    if !kind.starts_with("NP")
                        || out.occurrences[last]
                            .chunks
                            .iter()
                            .any(|s| s == &format!("E-{kind}"))
                    {
                        out.groups.push(Group {
                            alternatives: vec![kind.into()],
                            span: out.occurrences[a].span.start..out.occurrences[last].span.end,
                            tokens: a..last + 1,
                            kind: kind.into(),
                        });
                    }
                }
            }
        }
    }
    let mut by_span = std::collections::HashMap::new();
    let mut deduped: Vec<Group> = Vec::new();
    for g in out.groups {
        let key = (g.span.start, g.span.end);
        if let Some(&i) = by_span.get(&key) {
            let previous: &mut Group = &mut deduped[i];
            if previous.kind == "contiguous" && g.kind != "contiguous" {
                previous.kind = g.kind.clone();
            }
            for kind in g.alternatives {
                if !previous.alternatives.contains(&kind) {
                    previous.alternatives.push(kind);
                }
            }
        } else {
            by_span.insert(key, deduped.len());
            deduped.push(g);
        }
    }
    out.groups = deduped;
    out
}

/// Ordered, injective, exact whole-group witnesses. Type compatibility adds no
/// synonym or entailment license; unmatched obligations remain unverified.
pub fn witnesses(
    query: &str,
    q: &Analysis,
    source: &str,
    s: &Analysis,
    typed: bool,
) -> Vec<(usize, usize)> {
    let mut pairs = vec![];
    let mut after = 0;
    for (qi, group) in q.groups.iter().enumerate().filter(|(_, g)| {
        if typed {
            g.kind != "contiguous"
        } else {
            g.tokens.len() == 2
        }
    }) {
        if let Some((si, matched)) = s.groups.iter().enumerate().find(|(_, g)| {
            g.span.start >= after
                && (!typed || group.kind == g.kind)
                && query[group.span.clone()].eq_ignore_ascii_case(&source[g.span.clone()])
        }) {
            pairs.push((qi, si));
            after = matched.span.end;
        }
    }
    pairs
}

/// Whole original NP identity, never a head alias. Returns separate source spans;
/// callers must still check lifecycle, section identity and legal card budgets.
pub fn shared_np_anchors(
    left: &str,
    l: &Analysis,
    right: &str,
    r: &Analysis,
) -> Vec<(Range<usize>, Range<usize>)> {
    l.groups
        .iter()
        .filter(|g| g.kind.starts_with("NP") && g.tokens.len() >= 2)
        .flat_map(|a| {
            r.groups
                .iter()
                .filter(move |b| {
                    b.kind.starts_with("NP")
                        && left[a.span.clone()].eq_ignore_ascii_case(&right[b.span.clone()])
                })
                .map(move |b| (a.span.clone(), b.span.clone()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn group(span: Range<usize>) -> Group {
        Group {
            alternatives: vec!["NP-singular".into()],
            span,
            tokens: 0..2,
            kind: "NP-singular".into(),
        }
    }
    #[test]
    fn whole_modifiers_required() {
        let a = Analysis {
            occurrences: vec![],
            groups: vec![group(0..17)],
        };
        let b = Analysis {
            occurrences: vec![],
            groups: vec![group(0..18)],
        };
        assert!(shared_np_anchors("local build cache", &a, "remote build cache", &b).is_empty());
    }
    #[test]
    fn source_occurrence_cannot_satisfy_two_groups() {
        let a = Analysis {
            occurrences: vec![],
            groups: vec![group(0..11), group(12..23)],
        };
        let b = Analysis {
            occurrences: vec![],
            groups: vec![group(0..11)],
        };
        assert_eq!(
            witnesses("build cache build cache", &a, "build cache", &b, true).len(),
            1
        );
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum Support {
    /// Every raw query occurrence has an ordered, distinct literal witness.
    Literal,
    /// Partial matches and vector proposals cannot certify missing obligations.
    Unverified,
}

/// Sentence-bounded occurrence coverage retaining every query occurrence, including
/// operators, values and punctuation. No POS label releases an obligation.
pub fn literal_support(query: &str, q: &Analysis, source: &str, s: &Analysis) -> Support {
    if q.occurrences.is_empty() {
        return Support::Unverified;
    }
    let sentences: std::collections::BTreeSet<_> =
        s.occurrences.iter().map(|t| t.sentence).collect();
    for sentence in sentences {
        let candidates: Vec<_> = s
            .occurrences
            .iter()
            .filter(|t| t.sentence == sentence)
            .collect();
        let mut next = 0;
        let mut complete = true;
        for obligation in &q.occurrences {
            let Some(offset) = candidates[next..].iter().position(|candidate| {
                query[obligation.span.clone()].eq_ignore_ascii_case(&source[candidate.span.clone()])
            }) else {
                complete = false;
                break;
            };
            next += offset + 1;
        }
        if complete {
            return Support::Literal;
        }
    }
    Support::Unverified
}

#[cfg(test)]
mod pipeline_tests {
    use super::*;
    #[test]
    #[ignore = "requires explicitly supplied pinned LGPL tokenizer asset"]
    fn actual_pipeline_offsets_ambiguity_and_role_controls() {
        let tokenizer =
            Tokenizer::new(std::env::var("LINN_POS_TOKENIZER").expect("supply audited tokenizer"))
                .unwrap();
        assert!(tokenizer.chunker().is_some());
        assert_eq!(tokenizer.rules().len(), 843);
        for text in [
            "record changes",
            "worker’s café\r\nsettings",
            "cafe\u{301} naïve",
            "E_INDEX-42 --no-cache",
            "`cache`\n```rust\nlet cache = 1;\n```",
            "minimum maximum increase decrease not no without from to",
            "\"local build cache\"",
            "cache invalidation policy",
        ] {
            let a = analyze(&tokenizer, text);
            assert!(!a.occurrences.is_empty());
            for g in a.groups {
                assert!(text.get(g.span).is_some());
            }
        }
        assert_eq!(
            literal_support(
                "cache policy",
                &analyze(&tokenizer, "cache policy"),
                "cache. Unrelated policy.",
                &analyze(&tokenizer, "cache. Unrelated policy.")
            ),
            Support::Unverified
        );
        let query = "Alice follows Bob";
        let source = "Bob follows Alice";
        assert_eq!(
            literal_support(
                query,
                &analyze(&tokenizer, query),
                source,
                &analyze(&tokenizer, source)
            ),
            Support::Unverified
        );
        let a = analyze(&tokenizer, "record changes");
        assert!(a.occurrences.iter().any(|t| t.analyses.len() > 1));
        assert!(
            analyze(&tokenizer, "Alice follows Bob")
                .groups
                .iter()
                .any(|g| g.kind == "VP")
        );
    }
}
