pub(crate) use crate::nlp::annotation::dictionary::WordIdInt;
pub use crate::nlp::annotation::dictionary::{PosId, WordId, owned};
use crate::nlp::annotation::dictionary::{SpecialPos, Tagger};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt,
    ops::{Add, AddAssign, Range, Sub},
};
pub(crate) type DefaultHashMap<K, V> = std::collections::HashMap<K, V>;
#[derive(Clone, Copy, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub byte: usize,
    pub char: usize,
}
impl PartialOrd for Position {
    fn partial_cmp(&self, b: &Self) -> Option<Ordering> {
        let x = self.byte.cmp(&b.byte);
        (x == self.char.cmp(&b.char)).then_some(x)
    }
}
impl Add for Position {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self {
            byte: self.byte + b.byte,
            char: self.char + b.char,
        }
    }
}
impl AddAssign for Position {
    fn add_assign(&mut self, b: Self) {
        *self = *self + b;
    }
}
impl Sub for Position {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        Self {
            byte: self.byte.saturating_sub(b.byte),
            char: self.char.saturating_sub(b.char),
        }
    }
}
#[derive(Clone, Default, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    byte: Range<usize>,
    char: Range<usize>,
}
impl Span {
    pub fn new(byte: Range<usize>, char: Range<usize>) -> Self {
        Self { byte, char }
    }
    pub fn from_positions(start: Position, end: Position) -> Self {
        assert!(start <= end);
        Self::new(start.byte..end.byte, start.char..end.char)
    }
    pub fn start(&self) -> Position {
        Position {
            byte: self.byte.start,
            char: self.char.start,
        }
    }
    pub fn end(&self) -> Position {
        Position {
            byte: self.byte.end,
            char: self.char.end,
        }
    }
    pub fn byte(&self) -> &Range<usize> {
        &self.byte
    }
    pub fn char(&self) -> &Range<usize> {
        &self.char
    }
    pub fn len(&self) -> Position {
        self.end() - self.start()
    }
    pub fn is_empty(&self) -> bool {
        self.start() == self.end()
    }
    pub fn set_start(&mut self, p: Position) {
        self.byte.start = p.byte;
        self.char.start = p.char;
    }
    pub fn set_end(&mut self, p: Position) {
        self.byte.end = p.byte;
        self.char.end = p.char;
    }
    pub fn rshift(mut self, p: Position) -> Self {
        self.set_start(self.start() + p);
        self.set_end(self.end() + p);
        self
    }
    pub fn lshift(mut self, p: Position) -> Self {
        self.set_start(self.start() - p);
        self.set_end(self.end() - p);
        self
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct WordData<'t> {
    lemma: WordId<'t>,
    pos: PosId<'t>,
    frozen: bool,
}
impl<'t> WordData<'t> {
    pub fn new(lemma: WordId<'t>, pos: PosId<'t>) -> Self {
        Self {
            lemma,
            pos,
            frozen: false,
        }
    }
    pub fn lemma(&self) -> &WordId<'t> {
        &self.lemma
    }
    pub fn pos(&self) -> &PosId<'t> {
        &self.pos
    }
    pub fn freeze(&mut self) {
        self.frozen = true;
    }
    pub fn frozen(&self) -> bool {
        self.frozen
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Word<'t> {
    text: WordId<'t>,
    tags: Vec<WordData<'t>>,
}
impl<'t> Word<'t> {
    pub fn new(text: WordId<'t>, tags: Vec<WordData<'t>>) -> Self {
        Self { text, tags }
    }
    pub fn text(&self) -> &WordId<'t> {
        &self.text
    }
    pub fn tags(&self) -> &[WordData<'t>] {
        &self.tags
    }
    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }
    pub fn clear(&mut self) {
        self.retain(|_| false);
    }
    pub fn retain<F: FnMut(&WordData<'t>) -> bool>(&mut self, mut f: F) {
        self.tags.retain(|x| x.frozen || f(x));
    }
    pub fn push(&mut self, d: WordData<'t>) {
        self.tags.push(d);
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct IncompleteToken<'t> {
    word: Word<'t>,
    span: Span,
    is_sentence_end: bool,
    has_space_before: bool,
    chunks: Vec<String>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Token<'t> {
    word: Word<'t>,
    span: Span,
    has_space_before: bool,
    chunks: Vec<String>,
}
impl<'t> IncompleteToken<'t> {
    pub(crate) fn new(
        word: Word<'t>,
        span: Span,
        is_sentence_end: bool,
        has_space_before: bool,
        chunks: Vec<String>,
    ) -> Self {
        Self {
            word,
            span,
            is_sentence_end,
            has_space_before,
            chunks,
        }
    }
    pub fn word_mut(&mut self) -> &mut Word<'t> {
        &mut self.word
    }
    pub fn is_sentence_end(&self) -> bool {
        self.is_sentence_end
    }
    pub fn is_sentence_end_mut(&mut self) -> &mut bool {
        &mut self.is_sentence_end
    }
    pub fn chunks_mut(&mut self) -> &mut Vec<String> {
        &mut self.chunks
    }
    pub fn into_token(mut self) -> Token<'t> {
        self.word.push(WordData::new(
            self.word.text.clone(),
            PosId::special(SpecialPos::Empty),
        ));
        if self.word.tags.iter().all(|x| x.pos.as_str().is_empty()) {
            self.word.push(WordData::new(
                self.word.text.clone(),
                PosId::special(SpecialPos::Unknown),
            ));
        }
        if self.is_sentence_end {
            self.word.push(WordData::new(
                self.word.text.clone(),
                PosId::special(SpecialPos::SentEnd),
            ));
        }
        Token {
            word: self.word,
            span: self.span,
            has_space_before: self.has_space_before,
            chunks: self.chunks,
        }
    }
}
impl Token<'_> {
    pub(crate) fn sent_start() -> &'static Token<'static> {
        static START: std::sync::LazyLock<Token<'static>> = std::sync::LazyLock::new(|| Token {
            word: Word::new(
                WordId::empty(),
                vec![WordData::new(
                    WordId::empty(),
                    PosId::special(SpecialPos::SentStart),
                )],
            ),
            span: Span::default(),
            has_space_before: false,
            chunks: vec![],
        });
        &START
    }
}
impl<'t> Token<'t> {
    pub fn word(&self) -> &Word<'t> {
        &self.word
    }
    pub fn span(&self) -> &Span {
        &self.span
    }
    pub fn has_space_before(&self) -> bool {
        self.has_space_before
    }
    pub fn chunks(&self) -> &[String] {
        &self.chunks
    }
    pub fn rshift(mut self, p: Position) -> Self {
        self.span = self.span.rshift(p);
        self
    }
}
impl<'t> IncompleteToken<'t> {
    pub fn word(&self) -> &Word<'t> {
        &self.word
    }
    pub fn span(&self) -> &Span {
        &self.span
    }
    pub fn has_space_before(&self) -> bool {
        self.has_space_before
    }
    pub fn chunks(&self) -> &[String] {
        &self.chunks
    }
    pub fn rshift(mut self, p: Position) -> Self {
        self.span = self.span.rshift(p);
        self
    }
}
#[derive(Clone)]
pub struct IncompleteSentence<'t> {
    text: &'t str,
    tokens: Vec<IncompleteToken<'t>>,
    span: Span,
    tagger: &'t Tagger,
}
impl fmt::Debug for IncompleteSentence<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IncompleteSentence")
            .field("text", &self.text)
            .field("tokens", &self.tokens)
            .field("span", &self.span)
            .finish()
    }
}
impl PartialEq for IncompleteSentence<'_> {
    fn eq(&self, b: &Self) -> bool {
        self.text == b.text && self.tokens == b.tokens && self.span == b.span
    }
}
impl<'t> IntoIterator for IncompleteSentence<'t> {
    type Item = IncompleteToken<'t>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.tokens.into_iter()
    }
}
impl<'t> IncompleteSentence<'t> {
    pub fn text(&self) -> &'t str {
        self.text
    }
    pub fn len(&self) -> usize {
        self.tokens.len()
    }
    pub fn tagger(&self) -> &'t Tagger {
        self.tagger
    }
    pub fn span(&self) -> &Span {
        &self.span
    }
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &IncompleteToken<'t>> {
        self.tokens.iter()
    }
}
#[derive(Clone)]
pub struct Sentence<'t> {
    text: &'t str,
    tokens: Vec<Token<'t>>,
    span: Span,
    tagger: &'t Tagger,
}
impl fmt::Debug for Sentence<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sentence")
            .field("text", &self.text)
            .field("tokens", &self.tokens)
            .field("span", &self.span)
            .finish()
    }
}
impl PartialEq for Sentence<'_> {
    fn eq(&self, b: &Self) -> bool {
        self.text == b.text && self.tokens == b.tokens && self.span == b.span
    }
}
impl<'t> IntoIterator for Sentence<'t> {
    type Item = Token<'t>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.tokens.into_iter()
    }
}
impl<'t> Sentence<'t> {
    pub fn text(&self) -> &'t str {
        self.text
    }
    pub fn len(&self) -> usize {
        self.tokens.len()
    }
    pub fn tagger(&self) -> &'t Tagger {
        self.tagger
    }
    pub fn span(&self) -> &Span {
        &self.span
    }
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Token<'t>> {
        self.tokens.iter()
    }
}
impl<'t> IncompleteSentence<'t> {
    pub(crate) fn new(tokens: Vec<IncompleteToken<'t>>, text: &'t str, tagger: &'t Tagger) -> Self {
        Self {
            text,
            tokens,
            tagger,
            span: Span::new(0..text.len(), 0..text.chars().count()),
        }
    }
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut IncompleteToken<'t>> {
        self.tokens.iter_mut()
    }
    pub fn into_sentence(self) -> Sentence<'t> {
        Sentence {
            text: self.text,
            tokens: self
                .tokens
                .into_iter()
                .map(IncompleteToken::into_token)
                .collect(),
            span: self.span,
            tagger: self.tagger,
        }
    }
    pub fn rshift(mut self, p: Position) -> Self {
        self.span = self.span.rshift(p);
        self.tokens = self.tokens.into_iter().map(|t| t.rshift(p)).collect();
        self
    }
}
impl<'t> Sentence<'t> {
    pub fn tokens(&self) -> &[Token<'t>] {
        &self.tokens
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frozen_entries_skip_predicate_and_survive_clear() {
        let mut frozen = WordData::new(WordId::empty(), PosId::special(SpecialPos::Unknown));
        frozen.freeze();
        frozen.freeze();
        let mut w = Word::new(
            WordId::empty(),
            vec![
                frozen,
                WordData::new(WordId::empty(), PosId::special(SpecialPos::Empty)),
            ],
        );
        let mut calls = 0;
        w.retain(|_| {
            calls += 1;
            false
        });
        assert_eq!(calls, 1);
        w.clear();
        assert_eq!(w.tags().len(), 1);
        assert!(w.tags()[0].frozen());
    }
    #[test]
    #[allow(clippy::reversed_empty_ranges)] // Reversed coordinate ranges are valid data.
    fn coordinates_preserve_independent_saturation() {
        let s = Span::new(7..2, 1..9);
        assert_eq!(s.len(), Position { byte: 0, char: 8 });
        let s = s.lshift(Position { byte: 3, char: 5 });
        assert_eq!(s.byte(), &(4..0));
        assert_eq!(s.char(), &(0..4));
        assert!(
            Position { byte: 1, char: 2 }
                .partial_cmp(&Position { byte: 2, char: 1 })
                .is_none()
        );
    }
}
