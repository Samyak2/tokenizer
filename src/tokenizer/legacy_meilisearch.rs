use std::borrow::Cow;

use slice_group_by::StrGroupBy;

use super::{TokenStream, Tokenizer};
use crate::detection::classify_separator;
use crate::detection::is_cj;
use crate::processors::ProcessedText;
use crate::token::SeparatorKind;
use crate::{Token, TokenKind};

pub struct LegacyMeilisearch;

impl Tokenizer for LegacyMeilisearch {
    fn tokenize<'a>(&self, s: &'a ProcessedText<'a>) -> super::TokenStream<'a> {
        TokenStream { inner: Box::new(LegacyTokenizer::new(s)) }
    }
}

pub struct LegacyTokenizer<'a> {
    inner: &'a str,
    char_index: usize,
    byte_index: usize,
}

impl<'a> LegacyTokenizer<'a> {
    pub fn new(s: &'a ProcessedText<'a>) -> Self {
        // skip every separator and set `char_index`
        // to the number of char trimmed
        Self { inner: s.processed.as_ref(), char_index: 0, byte_index: 0 }
    }
}

impl<'a> Iterator for LegacyTokenizer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut iter = self.inner.linear_group_by(same_group_category);
        let word = iter.next()?;

        let token = Some(Token {
            kind: TokenKind::Unknown,
            word: Cow::Borrowed(word),
            char_index: self.char_index,
            byte_start: self.byte_index,
            byte_end: self.byte_index + word.len(),
            char_map: None,
        });

        self.char_index += word.chars().count();
        self.byte_index += word.len();
        self.inner = &self.inner[word.len()..];

        token
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum CharCategory {
    Separator(SeparatorKind),
    Cj,
    Other,
}

fn classify_char(c: char) -> CharCategory {
    if let Some(category) = classify_separator(c) {
        CharCategory::Separator(category)
    } else if is_cj(c) {
        CharCategory::Cj
    } else {
        CharCategory::Other
    }
}

fn same_group_category(a: char, b: char) -> bool {
    match (classify_char(a), classify_char(b)) {
        (CharCategory::Cj, _) | (_, CharCategory::Cj) => false,
        (CharCategory::Separator(_), CharCategory::Separator(_)) => true,
        (a, b) => a == b,
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_byte_indices() {
        let tokenizer = LegacyMeilisearch;
        let orig = "The quick (\"brown\") fox can't jump 32.3 feet, right? Brr, it's 29.3°F!";
        let processed = ProcessedText { original: orig, processed: Cow::Borrowed(orig) };
        let tokens = tokenizer.tokenize(&processed);
        assert_eq!(orig, tokens.map(|t| &orig[t.byte_start..t.byte_end]).collect::<String>());

        let orig = "為一包含一千多萬目詞的帶標記平衡語料庫";
        let processed = ProcessedText { original: orig, processed: Cow::Borrowed(orig) };
        let tokens = tokenizer.tokenize(&processed).collect::<Vec<_>>();
        assert_eq!("為", tokens.first().unwrap().text());
        assert_eq!(
            orig,
            tokens.iter().map(|t| &orig[t.byte_start..t.byte_end]).collect::<String>()
        );
    }
}
