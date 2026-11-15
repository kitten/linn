use crate::nlp::annotation::{
    dictionary::Tagger,
    regex::Regex,
    types::{DefaultHashMap, PosId, Sentence, Span, Token, WordData, WordId, WordIdInt},
};
use either::Either;
use serde::{Deserialize, Serialize};
pub type Context<'a, 't> = (&'a MatchSentence<'t>, &'a MatchGraph<'t>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GraphId(pub usize);
impl GraphId {
    pub fn range(&self, end: &Self) -> impl DoubleEndedIterator<Item = Self> {
        (self.0..=end.0).map(Self)
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Matcher {
    pub matcher: Either<Either<String, GraphId>, Regex>,
    pub negate: bool,
    pub case_sensitive: bool,
    pub empty_always_false: bool,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct TextMatcher {
    pub matcher: Matcher,
    pub set: Option<WordSet>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PosMatcher {
    pub mask: Vec<bool>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WordDataMatcher {
    pub pos_matcher: Option<PosMatcher>,
    pub(crate) inflect_matcher: Option<TextMatcher>,
}
pub mod concrete {
    use super::*;
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ChunkAtom {
        pub(crate) matcher: Matcher,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SpaceBeforeAtom {
        pub(crate) value: bool,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TextAtom {
        pub(crate) matcher: TextMatcher,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WordDataAtom {
        pub(crate) matcher: WordDataMatcher,
        pub(crate) case_sensitive: bool,
    }
}
pub use concrete::*;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrueAtom {}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalseAtom {}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndAtom {
    pub atoms: Vec<Atom>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrAtom {
    pub atoms: Vec<Atom>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotAtom {
    pub atom: Box<Atom>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffsetAtom {
    pub atom: Box<Atom>,
    pub offset: isize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::enum_variant_names)] // Variant names are part of the serialized compatibility schema.
pub enum Atom {
    ChunkAtom(ChunkAtom),
    SpaceBeforeAtom(SpaceBeforeAtom),
    TextAtom(TextAtom),
    WordDataAtom(WordDataAtom),
    TrueAtom(TrueAtom),
    FalseAtom(FalseAtom),
    AndAtom(AndAtom),
    OrAtom(OrAtom),
    NotAtom(NotAtom),
    OffsetAtom(OffsetAtom),
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Quantifier {
    pub min: usize,
    pub max: usize,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Part {
    pub atom: Atom,
    pub quantifier: Quantifier,
    pub greedy: bool,
    pub visible: bool,
    pub unify: Option<bool>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Composition {
    pub parts: Vec<Part>,
    #[serde(serialize_with = "crate::nlp::annotation::serialization::map")]
    pub id_to_idx: DefaultHashMap<GraphId, usize>,
    pub can_stop_mask: Vec<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub(crate) struct WordSet(Vec<WordIdInt>);
impl<'de> Deserialize<'de> for WordSet {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let mut v = Vec::<WordIdInt>::deserialize(d)?;
        v.sort_unstable();
        v.dedup();
        Ok(Self(v))
    }
}
#[derive(Debug, Default, Clone)]
pub struct Group {
    pub span: Span,
}
impl Group {
    pub fn tokens<'t>(
        &self,
        sentence: &'t MatchSentence,
    ) -> impl DoubleEndedIterator<Item = &'t Token<'t>> {
        let range = self.span.char().clone();
        let tokens = sentence.sentence.tokens();
        let first = tokens.partition_point(|t| t.span().char().start < range.start);
        let end = tokens.partition_point(|t| t.span().char().end <= range.end);
        tokens[first..end.max(first)].iter().filter(move |t| {
            t.span().char().start < t.span().char().end
                && t.span().char().start >= range.start
                && t.span().char().end <= range.end
        })
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct MatchSentence<'t> {
    sentence: &'t Sentence<'t>,
    sent_start: &'t Token<'t>,
}
impl<'t> MatchSentence<'t> {
    pub fn new(sentence: &'t Sentence<'t>) -> Self {
        Self {
            sentence,
            sent_start: Token::sent_start(),
        }
    }
    #[allow(clippy::should_implement_trait)] // Required compatibility method name.
    pub fn index(&self, index: usize) -> &Token<'_> {
        if index == 0 {
            self.sent_start
        } else {
            &self.sentence.tokens()[index - 1]
        }
    }
    #[allow(clippy::len_without_is_empty)] // Includes a mandatory synthetic start token.
    pub fn len(&self) -> usize {
        self.sentence.len() + 1
    }
    pub fn tagger(&self) -> &'t Tagger {
        self.sentence.tagger()
    }
    pub fn span(&self) -> &Span {
        self.sentence.span()
    }
}
#[derive(Debug, Clone)]
pub struct MatchGraph<'t> {
    groups: Vec<Group>,
    id_to_idx: &'t DefaultHashMap<GraphId, usize>,
}
impl Default for MatchGraph<'_> {
    fn default() -> Self {
        static EMPTY: std::sync::LazyLock<DefaultHashMap<GraphId, usize>> =
            std::sync::LazyLock::new(DefaultHashMap::new);
        Self {
            groups: vec![],
            id_to_idx: &EMPTY,
        }
    }
}
impl<'t> MatchGraph<'t> {
    pub fn new(groups: Vec<Group>, id_to_idx: &'t DefaultHashMap<GraphId, usize>) -> Self {
        Self { groups, id_to_idx }
    }
    pub fn by_index(&self, index: usize) -> &Group {
        &self.groups[index]
    }
    pub fn by_id(&self, id: GraphId) -> &Group {
        self.by_index(self.get_index(id))
    }
    pub fn get_index(&self, id: GraphId) -> usize {
        self.id_to_idx[&id]
    }
    pub fn groups(&self) -> &[Group] {
        &self.groups
    }
    pub fn fill_empty(&mut self, s: &MatchSentence) {
        let bounds: Vec<_> = self
            .groups
            .iter()
            .map(|g| {
                g.tokens(s)
                    .next()
                    .zip(g.tokens(s).next_back())
                    .map(|(a, b)| (a.span().start(), b.span().end()))
            })
            .collect();
        let mut left = bounds.iter().flatten().next().expect("empty graph").0;
        let mut right = bounds.iter().flatten().next_back().unwrap().1;
        for (g, bound) in self.groups.iter_mut().zip(&bounds) {
            if let Some((a, b)) = bound {
                g.span = Span::new(a.byte..b.byte, a.char..b.char);
                left = *b;
            } else {
                g.span.set_end(left);
            }
        }
        for (g, bound) in self.groups.iter_mut().zip(&bounds).rev() {
            if bound.is_some() {
                right = g.span.start();
            } else {
                g.span.set_start(right);
            }
        }
    }
}
impl Matcher {
    pub fn is_match(&self, text: &str, ctx: Option<Context>, case: Option<bool>) -> bool {
        if text.is_empty() {
            return !self.empty_always_false && self.negate;
        }
        let equal = |a: &str, b: &str| {
            if case.unwrap_or(self.case_sensitive) {
                a == b
            } else {
                unicase::UniCase::new(a) == unicase::UniCase::new(b)
            }
        };
        let yes = match &self.matcher {
            Either::Left(Either::Left(lit)) => equal(text, lit),
            Either::Right(re) => re.is_match(text),
            Either::Left(Either::Right(id)) => {
                let (s, g) = ctx.expect("graph context");
                g.by_id(*id)
                    .tokens(s)
                    .next()
                    .is_some_and(|t| equal(text, t.word().as_str()))
            }
        };
        yes != self.negate
    }
    pub fn is_slice_match<S: AsRef<str>>(
        &self,
        text: &[S],
        ctx: Option<Context>,
        case: Option<bool>,
    ) -> bool {
        text.iter().any(|t| self.is_match(t.as_ref(), ctx, case))
    }
}
impl TextMatcher {
    pub fn is_match(&self, word_id: &WordId, ctx: Option<Context>, case: Option<bool>) -> bool {
        if let (Some(set), Some(id)) = (&self.set, word_id.id()) {
            set.0.binary_search(id).is_ok()
        } else {
            self.matcher.is_match(word_id.as_str(), ctx, case)
        }
    }
}
impl PosMatcher {
    pub fn is_match(&self, pos: &PosId) -> bool {
        self.mask[pos.id().value() as usize]
    }
}
impl WordDataMatcher {
    pub fn is_match(&self, input: &[WordData], ctx: Option<Context>, case: Option<bool>) -> bool {
        input.iter().any(|tag| {
            self.pos_matcher
                .as_ref()
                .is_none_or(|m| m.is_match(tag.pos()))
                && self
                    .inflect_matcher
                    .as_ref()
                    .is_none_or(|m| m.is_match(tag.lemma(), ctx, case))
        })
    }
}
pub trait Atomable: Send + Sync {
    fn is_match(&self, ctx: Context, position: usize) -> bool;
}
impl Atomable for Atom {
    fn is_match(&self, ctx: Context, p: usize) -> bool {
        match self {
            Self::ChunkAtom(a) => a.is_match(ctx, p),
            Self::SpaceBeforeAtom(a) => a.is_match(ctx, p),
            Self::TextAtom(a) => a.is_match(ctx, p),
            Self::WordDataAtom(a) => a.is_match(ctx, p),
            Self::TrueAtom(a) => a.is_match(ctx, p),
            Self::FalseAtom(a) => a.is_match(ctx, p),
            Self::AndAtom(a) => a.is_match(ctx, p),
            Self::OrAtom(a) => a.is_match(ctx, p),
            Self::NotAtom(a) => a.is_match(ctx, p),
            Self::OffsetAtom(a) => a.is_match(ctx, p),
        }
    }
}
impl Atomable for ChunkAtom {
    fn is_match(&self, ctx: Context, p: usize) -> bool {
        self.matcher
            .is_slice_match(ctx.0.index(p).chunks(), Some(ctx), None)
    }
}
impl Atomable for SpaceBeforeAtom {
    fn is_match(&self, ctx: Context, p: usize) -> bool {
        ctx.0.index(p).has_space_before() == self.value
    }
}
impl Atomable for TextAtom {
    fn is_match(&self, ctx: Context, p: usize) -> bool {
        self.matcher
            .is_match(ctx.0.index(p).word().text(), Some(ctx), None)
    }
}
impl Atomable for WordDataAtom {
    fn is_match(&self, ctx: Context, p: usize) -> bool {
        self.matcher.is_match(
            ctx.0.index(p).word().tags(),
            Some(ctx),
            Some(self.case_sensitive),
        )
    }
}
impl Atomable for TrueAtom {
    fn is_match(&self, ctx: Context, p: usize) -> bool {
        let _ = (ctx, p);
        true
    }
}
impl Atomable for FalseAtom {
    fn is_match(&self, ctx: Context, p: usize) -> bool {
        let _ = (ctx, p);
        false
    }
}
impl Atomable for AndAtom {
    fn is_match(&self, ctx: Context, p: usize) -> bool {
        self.atoms.iter().all(|a| a.is_match(ctx, p))
    }
}
impl Atomable for OrAtom {
    fn is_match(&self, ctx: Context, p: usize) -> bool {
        self.atoms.iter().any(|a| a.is_match(ctx, p))
    }
}
impl Atomable for NotAtom {
    fn is_match(&self, ctx: Context, p: usize) -> bool {
        !self.atom.is_match(ctx, p)
    }
}
impl Atomable for OffsetAtom {
    fn is_match(&self, ctx: Context, p: usize) -> bool {
        p.checked_add_signed(self.offset)
            .is_some_and(|i| i < ctx.0.len() && self.atom.is_match(ctx, i))
    }
}
impl Composition {
    pub fn apply<'t>(&'t self, s: &'t MatchSentence<'t>, start: usize) -> Option<MatchGraph<'t>> {
        if self.parts[0].quantifier.min > 0
            && !self.parts[0]
                .atom
                .is_match((s, &MatchGraph::default()), start)
        {
            return None;
        }
        let graph = MatchGraph::new(
            vec![Group::default(); self.parts.len() + 1],
            &self.id_to_idx,
        );
        self.walk(s, 0, start, graph)
    }
    fn walk<'t>(
        &'t self,
        s: &'t MatchSentence<'t>,
        mut j: usize,
        mut p: usize,
        mut g: MatchGraph<'t>,
    ) -> Option<MatchGraph<'t>> {
        let mut c = 0;
        let positive = loop {
            if j >= self.parts.len() {
                break true;
            }
            let part = &self.parts[j];
            if c >= part.quantifier.max {
                j += 1;
                c = 0;
                if j == self.parts.len() {
                    break false;
                }
                continue;
            }
            if p >= s.len() {
                break false;
            }
            if c >= part.quantifier.min && j + 1 < self.parts.len() {
                if part.greedy {
                    if let Some(result) = self.walk(s, j + 1, p, g.clone()) {
                        return Some(result);
                    }
                } else {
                    let mut matched = false;
                    for later in &self.parts[j + 1..] {
                        if later.atom.is_match((s, &g), p) {
                            matched = true;
                            break;
                        }
                        if later.quantifier.min > 0 {
                            break;
                        }
                    }
                    if matched {
                        j += 1;
                        c = 0;
                        continue;
                    }
                }
            }
            if !part.atom.is_match((s, &g), p) {
                break false;
            }
            let t = s.index(p);
            let h = &mut g.groups[j + 1];
            if h.span.char().end == 0 {
                h.span.set_start(t.span().start());
            }
            h.span.set_end(t.span().end());
            p += 1;
            c += 1;
        };
        if j < self.parts.len() && c >= self.parts[j].quantifier.min {
            j += 1;
        }
        if positive || j == self.parts.len() || self.can_stop_mask[j] {
            g.fill_empty(s);
            Some(g)
        } else {
            None
        }
    }
}
