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
    pub(super) fn new(
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
