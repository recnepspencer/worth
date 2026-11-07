use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceReportError {
    kind: EvidenceReportErrorKind,
    message: String,
}

impl EvidenceReportError {
    pub(crate) fn new(kind: EvidenceReportErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> EvidenceReportErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for EvidenceReportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for EvidenceReportError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceReportErrorKind {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    EmptyReportName,
    EmptyScope,
    InvalidScopeSegment,
    EmptyFieldName,
    InvalidFieldName,
    DuplicateFieldName,
    MissingParticipatingField,
    FieldNotFound,
    FieldKindMismatch,
}

impl From<IdentityDerivationDenial> for EvidenceReportError {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::new(
            EvidenceReportErrorKind::EvidenceIdentityDerivationDenied(cause),
            "evidence report identity exceeded its digest bounds",
        )
    }
}
