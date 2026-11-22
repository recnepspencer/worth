//! Pure temporal principal failure types.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryTemporalPrincipalFailureKind {
    SourceUnavailable,
    AdmissionRejected,
    SourcePanicked,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryTemporalPrincipalFailure {
    kind: WorthQueryTemporalPrincipalFailureKind,
    detail: String,
}

impl WorthQueryTemporalPrincipalFailure {
    pub fn new(kind: WorthQueryTemporalPrincipalFailureKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub fn kind(&self) -> WorthQueryTemporalPrincipalFailureKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}
