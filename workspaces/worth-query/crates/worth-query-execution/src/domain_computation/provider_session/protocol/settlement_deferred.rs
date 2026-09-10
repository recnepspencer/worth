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
