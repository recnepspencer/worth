//! Original access provenance is separate from the row's live Native custody.
use super::provider::{SealedIngressFrame, WorthQueryResourceProviderAttempt};
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandle;
use crate::domain_computation::primary_graph::application_query::CapturedApplicationQueryParameters;
use crate::domain_computation::primary_graph::entity_resolution::WorthQueryObservedScopeSelector;
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;
use worth_foundational::facade::AspectValue;
use worth_runtime_bridge::facade::{
    BridgeOwnedAsyncCommittedCompletion, BridgeOwnedAsyncManagedRequestAdmission,
    BridgeOwnedAsyncReleasedResource,
};

/// Issued selector data, never a cached action or disclosure permission. It
/// intentionally owns no Product observation or Native cleanup capability.
pub(super) struct OriginalResourceAccess {
    pub(super) branch: crate::basis::WorthQueryProductBranch,
    pub(super) scope: Arc<WorthQueryObservedScopeSelector>,
    pub(super) parameters: CapturedApplicationQueryParameters,
    #[expect(
        dead_code,
        reason = "issued selector retains its original principal provenance; fresh actions reauthorize"
    )]
    pub(super) birth_principal: worth_relational::facade::identity::EntityId,
    // The original parameter admission includes this shared wrapper backing.
}

pub(super) struct RetainedResourceValue {
    pub(super) value: AspectValue,
    // The value is dropped before its actual retained allocation reservation.
    #[expect(
        dead_code,
        reason = "reservation is retained until after the value drops"
    )]
    pub(super) funding: Option<ExecutionMemoryReservation>,
}

pub(super) enum ResourceLifecycle {
    /// Source clock progression can succeed before Native admission. That is
    /// real temporal progress, without a fabricated request or cleanup report.
    Reserved {
        clock: Option<worth_signal::facade::TemporalClockAdvanceSummary>,
    },
    Live {
        owner: BridgeOwnedAsyncManagedRequestAdmission,
        completion: Option<BridgeOwnedAsyncCommittedCompletion>,
    },
    Released(BridgeOwnedAsyncReleasedResource),
}

/// The immutable buffered frame remains owned while result capacity or a
/// Native handoff refuses. Actual queue admission is recorded separately from
/// unverified ingress; cleanup retires only this row's successful enqueue.
pub(super) struct ResourceCandidate {
    pub(super) frame: SealedIngressFrame,
    pub(super) verified: Option<(super::WorthQueryResourceFrameKind, RetainedResourceValue)>,
    pub(super) queue: ResourceCandidateQueue,
    pub(super) admission: Option<worth_runtime_bridge::facade::BridgeOwnedAsyncCompletionAdmission>,
    pub(super) validated: bool,
    pub(super) result_slot: Option<super::storage::ResourceResultReservation>,
}

#[expect(
    dead_code,
    reason = "actual queue mutation receipts remain under candidate custody"
)]
pub(super) enum ResourceCandidateQueue {
    Unadmitted,
    Admitted(worth_signal::facade::ResourceManagedQueueMutationReport),
    Drained(worth_signal::facade::ResourceManagedQueueMutationReport),
}

/// One row owns every unresolved obligation, including refused provider starts
/// and accepted clock-only progress. Public handle Drop cannot discard it.
pub(super) struct ResourceOccurrence {
    pub(super) access: Arc<OriginalResourceAccess>,
    pub(super) business_product: WorthQueryProductBranchLease,
    pub(super) input: RetainedResourceValue,
    pub(super) lifecycle: ResourceLifecycle,
    pub(super) last_clock: Option<worth_signal::facade::TemporalClockAdvanceSummary>,
    pub(super) native_slot: Option<usize>,
    pub(super) attempt: Option<WorthQueryResourceProviderAttempt>,
    pub(super) provider_closed: bool,
    pub(super) timeout: Option<worth_runtime_bridge::facade::BridgeOwnedAsyncTimeoutAdmission>,
    pub(super) retry: Option<worth_runtime_bridge::facade::BridgeOwnedAsyncRetrySchedule>,
    pub(super) candidate: Option<ResourceCandidate>,
    pub(super) result: Option<super::storage::RetainedResourceOutput>,
    pub(super) recovery: Option<WorthQueryRecoveryHandle>,
    pub(super) last_action:
        Option<crate::domain_computation::provider_session::WorthQueryResourceReadCompletion>,
}
