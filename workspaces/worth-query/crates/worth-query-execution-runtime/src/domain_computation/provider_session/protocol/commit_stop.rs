use super::{
    WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolCounters,
    WorthQueryProviderSessionProtocolStage,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProviderSessionCommitDeferred {
    kind: WorthQueryProviderSessionCommitDeferredKind,
    stage: WorthQueryProviderSessionProtocolStage,
    detail: String,
    counters: WorthQueryProviderSessionProtocolCounters,
    prerequisite_denial: Option<crate::ports::OpaquePrerequisiteDenial>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderSessionCommitDeferredKind {
    RelationalDeferred(worth_relational::facade::mvcc::RelationalPublicationDeferred),
    RetentionCapacityExhausted,
    PatchPositionReservationContended,
    CandidateLifetimeExpired {
        maximum_lifetime_millis: u64,
    },
    CandidateCapacityExhausted {
        maximum_candidates: usize,
    },
    PublishedSnapshotCapacityExhausted {
        maximum_handles: usize,
    },
    SourceCurrentnessRaced(worth_relational::facade::mvcc::CompanionCellEditStop),
    RequiredPrerequisitePending(crate::ports::OpaquePrerequisiteDenialKind),
}

impl WorthQueryProviderSessionCommitDeferred {
    pub fn new(
        kind: WorthQueryProviderSessionCommitDeferredKind,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            prerequisite_denial: None,
            stage: WorthQueryProviderSessionProtocolStage::Commit,
            detail: detail.into(),
            counters: WorthQueryProviderSessionProtocolCounters::default(),
        }
    }

    pub fn required_prerequisite_opaque(
        kind: crate::ports::OpaquePrerequisiteDenialKind,
        denial: crate::ports::OpaquePrerequisiteDenial,
        detail: impl Into<String>,
    ) -> Self {
        let mut deferred = Self::new(
            WorthQueryProviderSessionCommitDeferredKind::RequiredPrerequisitePending(kind),
            detail,
        );
        deferred.prerequisite_denial = Some(denial);
        deferred
    }

    pub fn into_prerequisite_denial_opaque(self) -> Option<crate::ports::OpaquePrerequisiteDenial> {
        self.prerequisite_denial
    }

    pub fn kind(&self) -> WorthQueryProviderSessionCommitDeferredKind {
        self.kind.clone()
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

    pub fn at_stage(
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
    ProductUnpublished(crate::ports::OpaqueProductPublicationOutcome),
    ProductPreparationPending(crate::ports::OpaqueProductPublicationOutcome),
    ProductPreparationRefused(crate::ports::OpaqueProductPublicationOutcome),

    NoEffect(worth_runtime_world::facade::NoEffectCompositePublication),
    PreEffectDenied(WorthQueryProviderSessionFailure),
    Denied(WorthQueryProviderSessionFailure),
    Deferred(WorthQueryProviderSessionCommitDeferred),
    ControlStopped(super::WorthQueryProviderSessionCommitControlStopped),
}

impl From<WorthQueryProviderSessionFailure> for WorthQueryProviderSessionCommitStop {
    fn from(failure: WorthQueryProviderSessionFailure) -> Self {
        Self::Denied(failure)
    }
}

mod court_doors_for_test;
