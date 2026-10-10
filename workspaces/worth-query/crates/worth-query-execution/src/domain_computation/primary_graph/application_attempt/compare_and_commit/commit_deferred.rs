//! Application-owned deferred commit evidence.

/// Evidence that a commit attempt stopped at a capacity, lifetime or prerequisite boundary, carried
/// by the `Deferred` commit outcome.
///
/// Nothing was committed. The [`kind`](Self::kind) names the boundary that
/// deferred publication; retry when that boundary can admit the attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationCommitDeferred {
    kind: WorthQueryApplicationCommitDeferredKind,
    stage: crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolStage,
    detail: String,
    counters:
        crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolCounters,
    prerequisite_denial:
        Option<crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial>,
}

/// The limit that deferred a commit attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationCommitDeferredKind {
    /// A required native publication companion deferred the attempt.
    RelationalDeferred(worth_relational::facade::mvcc::RelationalPublicationDeferred),
    /// The owner had no room to retain another basis for the attempt.
    RetentionCapacityExhausted,
    /// Another attempt held the reservation this commit needed.
    PatchPositionReservationContended,
    /// The prepared candidate outlived its maximum lifetime before it could commit.
    CandidateLifetimeExpired { maximum_lifetime_millis: u64 },
    /// The owner already held its maximum number of prepared candidates.
    CandidateCapacityExhausted { maximum_candidates: usize },
    /// The owner already held its maximum number of published snapshot handles.
    PublishedSnapshotCapacityExhausted { maximum_handles: usize },
    /// Source marking changed during exact same-image revalidation; retry the commit.
    SourceCurrentnessRaced(worth_relational::facade::mvcc::CompanionCellEditStop),
    /// Required upstream custody or its bounded preparation is unavailable.
    RequiredPrerequisitePending(
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind,
    ),
}

impl From<crate::domain_computation::provider_session::WorthQueryProviderSessionCommitDeferredKind>
    for WorthQueryApplicationCommitDeferredKind
{
    fn from(
        kind: crate::domain_computation::provider_session::WorthQueryProviderSessionCommitDeferredKind,
    ) -> Self {
        use crate::domain_computation::provider_session::WorthQueryProviderSessionCommitDeferredKind as Provider;
        match kind {
            Provider::RelationalDeferred(deferred) => Self::RelationalDeferred(deferred),
            Provider::RetentionCapacityExhausted => Self::RetentionCapacityExhausted,
            Provider::PatchPositionReservationContended => Self::PatchPositionReservationContended,
            Provider::CandidateLifetimeExpired {
                maximum_lifetime_millis,
            } => Self::CandidateLifetimeExpired {
                maximum_lifetime_millis,
            },
            Provider::CandidateCapacityExhausted { maximum_candidates } => {
                Self::CandidateCapacityExhausted { maximum_candidates }
            }
            Provider::PublishedSnapshotCapacityExhausted { maximum_handles } => {
                Self::PublishedSnapshotCapacityExhausted { maximum_handles }
            }
            Provider::SourceCurrentnessRaced(stop) => Self::SourceCurrentnessRaced(stop),
            Provider::RequiredPrerequisitePending(kind) => Self::RequiredPrerequisitePending(kind),
        }
    }
}

impl WorthQueryApplicationCommitDeferred {
    pub(in crate::domain_computation::primary_graph) fn from_provider_session(
        deferred: crate::domain_computation::provider_session::WorthQueryProviderSessionCommitDeferred,
    ) -> Self {
        Self {
            kind: deferred.kind().into(),
            stage: deferred.stage(),
            detail: deferred.detail().to_owned(),
            counters: deferred.counters(),
            prerequisite_denial: deferred.into_prerequisite_denial(),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn into_prerequisite_denial(
        self,
    ) -> Option<crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial> {
        self.prerequisite_denial
    }

    pub fn kind(&self) -> WorthQueryApplicationCommitDeferredKind {
        self.kind.clone()
    }

    pub const fn stage(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolStage {
        self.stage
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub const fn counters(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolCounters
    {
        self.counters
    }
}
