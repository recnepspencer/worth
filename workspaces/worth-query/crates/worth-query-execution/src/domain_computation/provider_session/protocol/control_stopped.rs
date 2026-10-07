use super::{WorthQueryProviderSessionProtocolCounters, WorthQueryProviderSessionProtocolStage};

/// The request control condition that stopped provider execution before publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderSessionControlStopKind {
    Cancelled,
    TimedOut,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProviderSessionCommitControlStopped {
    kind: WorthQueryProviderSessionControlStopKind,
    stage: WorthQueryProviderSessionProtocolStage,
    detail: String,
    counters: WorthQueryProviderSessionProtocolCounters,
    execution_preparation: bool,
}

impl WorthQueryProviderSessionCommitControlStopped {
    pub(in crate::domain_computation) fn new(
        kind: WorthQueryProviderSessionControlStopKind,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            stage: WorthQueryProviderSessionProtocolStage::Commit,
            detail: detail.into(),
            counters: WorthQueryProviderSessionProtocolCounters::default(),
            execution_preparation: false,
        }
    }

    pub(in crate::domain_computation) fn execution_preparation(
        kind: WorthQueryProviderSessionControlStopKind,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            execution_preparation: true,
            ..Self::new(kind, detail)
        }
    }

    pub(in crate::domain_computation) const fn is_execution_preparation(&self) -> bool {
        self.execution_preparation
    }

    pub const fn kind(&self) -> WorthQueryProviderSessionControlStopKind {
        self.kind
    }

    pub const fn stage(&self) -> WorthQueryProviderSessionProtocolStage {
        self.stage
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub const fn counters(&self) -> WorthQueryProviderSessionProtocolCounters {
        self.counters
    }

    pub(in crate::domain_computation) fn at_stage(
        mut self,
        stage: WorthQueryProviderSessionProtocolStage,
        counters: WorthQueryProviderSessionProtocolCounters,
    ) -> Self {
        self.stage = stage;
        self.counters = counters;
        self
    }
}
