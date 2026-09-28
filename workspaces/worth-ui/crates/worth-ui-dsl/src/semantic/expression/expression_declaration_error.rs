use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WorthUiExpressionDeclarationErrorKind {
    EmptyIdentity,
    MissingBody,
    WrongIntroducer,
    DuplicateBody,
    MisplacedBody,
    MalformedClause,
    UnknownClause,
    InvalidOperandName,
    DuplicateOperand,
    UnknownOperandSource,
    MissingResult,
    ExtraResult,
    DuplicateResult,
    BooleanResult,
    UnknownResultType,
}

/// Why an expression declaration is not well formed, before any kernel
/// admission runs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiExpressionDeclarationError {
    kind: WorthUiExpressionDeclarationErrorKind,
    detail: String,
}

impl WorthUiExpressionDeclarationError {
    pub(crate) fn new(
        kind: WorthUiExpressionDeclarationErrorKind,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub fn kind(&self) -> WorthUiExpressionDeclarationErrorKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for WorthUiExpressionDeclarationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for WorthUiExpressionDeclarationError {}
