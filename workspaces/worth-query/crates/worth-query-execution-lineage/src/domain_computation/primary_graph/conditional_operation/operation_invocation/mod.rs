//! Pure temporal invocation failure types.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryTemporalInvocationFailureKind {
    ProjectionRejected,
    InvocationRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryTemporalInvocationFailure {
    kind: WorthQueryTemporalInvocationFailureKind,
    detail: String,
}

impl WorthQueryTemporalInvocationFailure {
    pub fn new(kind: WorthQueryTemporalInvocationFailureKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub fn kind(&self) -> WorthQueryTemporalInvocationFailureKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}
