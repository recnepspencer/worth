use crate::source::WorthUiSourceSpan;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorthUiParseDiagnosticCode {
    InvalidCharacter,
    UnterminatedStringLiteral,
    UnexpectedToken,
    MissingIdentifier,
    MissingStringLiteral,
    MissingEquals,
    MissingSemicolon,
    MissingBlockStart,
    UnterminatedBlock,
    UnterminatedExpressionBody,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorthUiParseDiagnostic {
    code: WorthUiParseDiagnosticCode,
    message: String,
    span: WorthUiSourceSpan,
}

impl WorthUiParseDiagnostic {
    pub(crate) fn new(
        code: WorthUiParseDiagnosticCode,
        message: impl Into<String>,
        span: WorthUiSourceSpan,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            span,
        }
    }

    pub(crate) fn code(&self) -> WorthUiParseDiagnosticCode {
        self.code
    }

    pub(crate) fn span(&self) -> &WorthUiSourceSpan {
        &self.span
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}
