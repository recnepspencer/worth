//! Moved whole from `domain_computation::provider_session::graph_provider::bounded_step::denial`: the items below and every impl they own.
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphProviderStepDenialKind {
    WorkBudgetExceeded,
    UnexpectedEffect,
    EffectPostureDenied,
    UnexpectedProjection,
    MissingProjectionChunk,
    MultipleProjectionChunks,
    ChunkWidthExceeded,
    ScratchBudgetExceeded,
    RetainedBudgetExceeded,
    MemoryAllocationFailed,
    ForeignRetainedMemory,
    MultipleExecutionAdmissions,
    ForeignExecutionAdmission,
    ArtifactAdmissionDenied,
    MultipleCheckpoints,
    NoProgress,
    ProviderFailureLatched,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphProviderStepDenial {
    kind: WorthQueryGraphProviderStepDenialKind,
    detail: Arc<str>,
}

impl WorthQueryGraphProviderStepDenial {
    pub(in crate::domain_computation) fn new(
        kind: WorthQueryGraphProviderStepDenialKind,
        detail: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub const fn kind(&self) -> WorthQueryGraphProviderStepDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}
