use super::{
    WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolCounters,
    WorthQueryProviderSessionProtocolStage,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProviderSessionSettlementDeferred {
    stage: WorthQueryProviderSessionProtocolStage,
    detail: String,
    counters: WorthQueryProviderSessionProtocolCounters,
    settlement: worth_relational::facade::publication::DeferredPublicationSettlement,
    publication_failure: Option<WorthQueryPerformedPublicationFailure>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPerformedPublicationFailure {
    kind: super::WorthQueryProviderSessionDenialKind,
    stage: WorthQueryProviderSessionProtocolStage,
    detail: String,
    counters: WorthQueryProviderSessionProtocolCounters,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProviderSessionCommitDeferred {
    kind: WorthQueryProviderSessionCommitDeferredKind,
    stage: WorthQueryProviderSessionProtocolStage,
    detail: String,
    counters: WorthQueryProviderSessionProtocolCounters,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderSessionCommitDeferredKind {
    RetentionCapacityExhausted,
    PatchPositionReservationContended,
    CandidateLifetimeExpired { maximum_lifetime_millis: u64 },
    CandidateCapacityExhausted { maximum_candidates: usize },
    PublishedSnapshotCapacityExhausted { maximum_handles: usize },
}

impl WorthQueryProviderSessionSettlementDeferred {
    pub(in crate::domain_computation) fn new(
        detail: impl Into<String>,
        settlement: worth_relational::facade::publication::DeferredPublicationSettlement,
    ) -> Self {
        Self {
            stage: WorthQueryProviderSessionProtocolStage::Commit,
            detail: detail.into(),
            counters: WorthQueryProviderSessionProtocolCounters::default(),
            settlement,
            publication_failure: None,
        }
    }

    pub fn stage(&self) -> WorthQueryProviderSessionProtocolStage {
        self.stage
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn counters(&self) -> WorthQueryProviderSessionProtocolCounters {
        self.counters
    }

    pub(in crate::domain_computation) fn settlement(
        &self,
    ) -> &worth_relational::facade::publication::DeferredPublicationSettlement {
        &self.settlement
    }

    pub fn publication_failure(&self) -> Option<&WorthQueryPerformedPublicationFailure> {
        self.publication_failure.as_ref()
    }

    pub(in crate::domain_computation) fn with_publication_failure(
        mut self,
        failure: &WorthQueryProviderSessionFailure,
    ) -> Self {
        self.publication_failure = Some(WorthQueryPerformedPublicationFailure {
            kind: failure.kind(),
            stage: failure.stage(),
            detail: failure.detail().to_owned(),
            counters: failure.counters(),
        });
        self
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

impl WorthQueryPerformedPublicationFailure {
    pub const fn kind(&self) -> super::WorthQueryProviderSessionDenialKind {
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
}

impl WorthQueryProviderSessionCommitDeferred {
    pub(in crate::domain_computation) fn new(
        kind: WorthQueryProviderSessionCommitDeferredKind,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            stage: WorthQueryProviderSessionProtocolStage::Commit,
            detail: detail.into(),
            counters: WorthQueryProviderSessionProtocolCounters::default(),
        }
    }

    pub const fn kind(&self) -> WorthQueryProviderSessionCommitDeferredKind {
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

#[derive(Debug)]
pub enum WorthQueryProviderSessionCommitStop {
    ProductStale(crate::domain_computation::WorthQueryProductStaleApplication),
    ProductUnpublished(crate::domain_computation::WorthQueryProductUnpublishedApplication),
    NoEffect(worth_runtime_world::facade::NoEffectCompositePublication),
    Denied(WorthQueryProviderSessionFailure),
    Deferred(WorthQueryProviderSessionCommitDeferred),
    ControlStopped(super::WorthQueryProviderSessionCommitControlStopped),
    SettlementDeferred(WorthQueryProviderSessionSettlementDeferred),
}

impl From<WorthQueryProviderSessionFailure> for WorthQueryProviderSessionCommitStop {
    fn from(failure: WorthQueryProviderSessionFailure) -> Self {
        Self::Denied(failure)
    }
}
