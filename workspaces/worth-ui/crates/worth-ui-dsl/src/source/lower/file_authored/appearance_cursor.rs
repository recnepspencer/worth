use crate::source::{WorthUiSourceSpan, WorthUiSourceTokenKind};

pub(super) struct Cursor<'a> {
    tokens: &'a [WorthUiSourceTokenKind],
    spans: &'a [WorthUiSourceSpan],
    index: usize,
}

impl<'a> Cursor<'a> {
    pub(super) fn with_spans(
        tokens: &'a [WorthUiSourceTokenKind],
        spans: &'a [WorthUiSourceSpan],
    ) -> Self {
        Self {
            tokens,
            spans,
            index: 0,
        }
    }

    pub(super) fn current_span(&self) -> Option<&WorthUiSourceSpan> {
        self.spans.get(self.index)
    }

    pub(super) fn eof(&self) -> bool {
        self.index >= self.tokens.len()
    }

    pub(super) fn advance(&mut self) {
        self.index += 1;
    }

    pub(super) fn word(&self) -> Result<&str, String> {
        match self.tokens.get(self.index) {
            Some(WorthUiSourceTokenKind::Identifier(value)) => Ok(value),
            Some(WorthUiSourceTokenKind::KeywordToken) => Ok("token"),
            Some(WorthUiSourceTokenKind::NumberLiteral(value)) => Ok(value),
            _ => Err("appearance declaration expected a word".to_owned()),
        }
    }

    pub(super) fn peek_word(&self, expected: &str) -> bool {
        self.word().is_ok_and(|word| word == expected)
    }

    pub(super) fn take_word(&mut self, expected: &str) -> bool {
        if self.peek_word(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub(super) fn expect_word(&mut self, expected: &str) -> Result<(), String> {
        if self.take_word(expected) {
            Ok(())
        } else {
            Err(format!("appearance declaration expected '{expected}'"))
        }
    }

    pub(super) fn take_symbol(&mut self, expected: WorthUiSourceTokenKind) -> bool {
        if self.tokens.get(self.index) == Some(&expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub(super) fn expect_symbol(&mut self, expected: WorthUiSourceTokenKind) -> Result<(), String> {
        if self.take_symbol(expected) {
            Ok(())
        } else {
            Err("appearance declaration has malformed punctuation".to_owned())
        }
    }

    pub(super) fn skip(&mut self, expected: WorthUiSourceTokenKind) {
        while self.take_symbol(expected.clone()) {}
    }
}
