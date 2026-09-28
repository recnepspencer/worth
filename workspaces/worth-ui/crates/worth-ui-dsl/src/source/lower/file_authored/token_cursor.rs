//! The one cursor every file-authored declaration body is read with. It says
//! what the body holds next; each grammar names the words it reads and the
//! denials it gives.
use crate::source::{WorthUiParsedBlockBody, WorthUiSourceSpan, WorthUiSourceTokenKind};

/// The tokens a grammar reads as words.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UiDeclarationWords {
    /// Identifiers only; a grammar that reads numbers reads them through
    /// `number()`.
    Identifiers,
    /// Identifiers, numbers, and the `token` keyword.
    Values,
    /// Values, and the `appearance` and `backdrop` keywords as well.
    ValuesAndDeclarationKeywords,
}

pub(super) struct Cursor<'a> {
    tokens: &'a [WorthUiSourceTokenKind],
    spans: &'a [WorthUiSourceSpan],
    whole: &'a WorthUiSourceSpan,
    words: UiDeclarationWords,
    grammar: &'static str,
    index: usize,
}

impl<'a> Cursor<'a> {
    /// Reads `body`, whose declaration spans `whole`, as `grammar` does.
    pub(super) fn new(
        body: &'a WorthUiParsedBlockBody,
        whole: &'a WorthUiSourceSpan,
        words: UiDeclarationWords,
        grammar: &'static str,
    ) -> Self {
        Self {
            tokens: body.tokens(),
            spans: body.token_spans(),
            whole,
            words,
            grammar,
            index: 0,
        }
    }

    pub(super) fn eof(&self) -> bool {
        self.index >= self.tokens.len()
    }

    pub(super) fn advance(&mut self) {
        self.index += 1;
    }

    /// The current token's span, or the whole declaration's at the end.
    pub(super) fn span(&self) -> &'a WorthUiSourceSpan {
        self.spans.get(self.index).unwrap_or(self.whole)
    }

    pub(super) fn word(&self) -> Result<&'a str, String> {
        use UiDeclarationWords::{Values, ValuesAndDeclarationKeywords};
        use WorthUiSourceTokenKind as Token;
        match (self.tokens.get(self.index), self.words) {
            (Some(Token::Identifier(word)), _) => Ok(word),
            (Some(Token::NumberLiteral(word)), Values | ValuesAndDeclarationKeywords) => Ok(word),
            (Some(Token::KeywordToken), Values | ValuesAndDeclarationKeywords) => Ok("token"),
            (Some(Token::KeywordAppearance), ValuesAndDeclarationKeywords) => Ok("appearance"),
            (Some(Token::KeywordBackdrop), ValuesAndDeclarationKeywords) => Ok("backdrop"),
            _ => Err(format!("{} expected a word", self.grammar)),
        }
    }

    pub(super) fn peek_word(&self, expected: &str) -> bool {
        self.word().is_ok_and(|word| word == expected)
    }

    pub(super) fn take_word(&mut self, expected: &str) -> bool {
        let found = self.peek_word(expected);
        if found {
            self.advance();
        }
        found
    }

    pub(super) fn expect_word(&mut self, expected: &str) -> Result<(), String> {
        self.take_word(expected)
            .then_some(())
            .ok_or_else(|| format!("{} expected '{expected}'", self.grammar))
    }

    pub(super) fn take_symbol(&mut self, expected: WorthUiSourceTokenKind) -> bool {
        let found = self.tokens.get(self.index) == Some(&expected);
        if found {
            self.advance();
        }
        found
    }

    pub(super) fn expect_symbol(&mut self, expected: WorthUiSourceTokenKind) -> Result<(), String> {
        self.take_symbol(expected)
            .then_some(())
            .ok_or_else(|| format!("{} has malformed punctuation", self.grammar))
    }

    pub(super) fn skip(&mut self, expected: WorthUiSourceTokenKind) {
        while self.take_symbol(expected.clone()) {}
    }

    /// The current number, taken when it parses as `T`. The lexer admits
    /// digits only, so a sign, a fraction, or a nonfinite value never reaches
    /// here as a number.
    pub(super) fn number<T: std::str::FromStr>(&mut self) -> Option<T> {
        let Some(WorthUiSourceTokenKind::NumberLiteral(text)) = self.tokens.get(self.index) else {
            return None;
        };
        let value = text.parse().ok()?;
        self.advance();
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{Cursor, UiDeclarationWords};
    use crate::source::{
        tokenize_module_source, WorthUiParsedBlockBody, WorthUiSourceModuleId, WorthUiSourceSpan,
    };

    /// `text` lexed as one declaration body, with the span it covers.
    fn body(text: &str) -> (WorthUiParsedBlockBody, WorthUiSourceSpan) {
        let module =
            WorthUiSourceModuleId::from_relative_path(std::path::Path::new("app/main.wui"))
                .unwrap();
        let whole = WorthUiSourceSpan::new(module.clone(), 0, text.len());
        let tokens = tokenize_module_source(&module, text).unwrap();
        let body = WorthUiParsedBlockBody::new_with_spans(whole.clone(), tokens);
        (body, whole)
    }

    /// Each token of `text` as the word `words` reads, or `None` where it is
    /// no word.
    fn words(text: &str, words: UiDeclarationWords) -> Vec<Option<String>> {
        let (body, whole) = body(text);
        let mut cursor = Cursor::new(&body, &whole, words, "grammar");
        let mut read = Vec::new();
        while !cursor.eof() {
            read.push(cursor.word().ok().map(str::to_owned));
            cursor.advance();
        }
        read
    }

    #[test]
    fn each_grammar_reads_only_its_own_words() {
        let text = "gap 12 token appearance backdrop";
        let word = |word: &str| Some(word.to_owned());
        assert_eq!(
            words(text, UiDeclarationWords::Identifiers),
            [word("gap"), None, None, None, None]
        );
        assert_eq!(
            words(text, UiDeclarationWords::Values),
            [word("gap"), word("12"), word("token"), None, None]
        );
        assert_eq!(
            words(text, UiDeclarationWords::ValuesAndDeclarationKeywords),
            [
                word("gap"),
                word("12"),
                word("token"),
                word("appearance"),
                word("backdrop")
            ]
        );
    }

    #[test]
    fn a_number_is_no_identifier_and_the_end_falls_back_to_the_whole_span() {
        let (body, whole) = body(" 12 ");
        let mut cursor = Cursor::new(&body, &whole, UiDeclarationWords::Identifiers, "layout");
        assert_eq!(cursor.word(), Err("layout expected a word".to_owned()));
        assert_ne!(cursor.span(), &whole);
        assert_eq!(cursor.number::<u16>(), Some(12));
        assert!(cursor.eof());
        assert_eq!(cursor.span(), &whole);
    }
}
