use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQuerySessionLabelError {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    EmptyNamespace,
    EmptyNameSegment,
    MissingNameSegments,
}

impl std::fmt::Display for WorthQuerySessionLabelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::EmptyNamespace => "session label namespace may not be empty",
            Self::EmptyNameSegment => "session label name segment may not be empty",
            Self::MissingNameSegments => "session label must contain at least one name segment",
        })
    }
}

impl std::error::Error for WorthQuerySessionLabelError {}

impl From<IdentityDerivationDenial> for WorthQuerySessionLabelError {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}
