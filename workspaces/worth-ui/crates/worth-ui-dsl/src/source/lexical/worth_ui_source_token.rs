use crate::source::WorthUiSourceSpan;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WorthUiSourceTokenKind {
    Identifier(String),
    StringLiteral(String),
    KeywordImport,
    KeywordComponent,
    KeywordControl,
    KeywordIntent,
    KeywordSurface,
    KeywordBinding,
    KeywordQueryScalar,
    KeywordQueryCollection,
    KeywordToken,
    KeywordAppearance,
    KeywordBackdrop,
    NumberLiteral(String),
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    Comma,
    Semicolon,
    Equals,
    Plus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorthUiSourceToken {
    kind: WorthUiSourceTokenKind,
    span: WorthUiSourceSpan,
}

impl WorthUiSourceToken {
    pub(crate) fn new(kind: WorthUiSourceTokenKind, span: WorthUiSourceSpan) -> Self {
        Self { kind, span }
    }

    pub(crate) fn kind(&self) -> &WorthUiSourceTokenKind {
        &self.kind
    }

    pub(crate) fn span(&self) -> &WorthUiSourceSpan {
        &self.span
    }
}
