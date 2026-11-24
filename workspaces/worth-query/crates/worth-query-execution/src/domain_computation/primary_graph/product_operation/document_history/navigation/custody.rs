//! Every progression posture owns its actual created branch or World continuation.
use super::*;
use crate::{
    basis::WorthQueryProductObservationLease,
    domain_computation::{
        execution_runtime::product_world::WorthQueryPreparedProductPublication,
        primary_graph::{
            product_operation::DetachedExpectedProductBranch,
            WorthQueryApplicationIdempotencyBinding,
        },
        WorthQueryProductUnpublishedRecovery,
    },
};
use std::ops::Deref;
use std::sync::{Mutex, MutexGuard};
use worth_execution::ExecutionMemoryReservation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHistoryNavigationPosture {
    Advancing,
    Prepared,
    CreatingCandidate,
    PreparingCandidate,
    PublishingCandidate,
    RecoveringPublication,
    AwaitingOutputs,
    AwaitingSelection,
    Selected,
    Discarding,
    Discarded,
}

pub(super) enum ForkCleanup {
    Creation(crate::basis::WorthQueryProductBranchCreationRecovery),
    Cleanup(crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupFailure),
}

pub(super) enum NavigationPhase {
    Prepared,
    ForkRecovery(ForkCleanup),
    Candidate(DetachedExpectedProductBranch),
    Publication {
        candidate: DetachedExpectedProductBranch,
        publication: WorthQueryPreparedProductPublication,
    },
    Unpublished {
        candidate: DetachedExpectedProductBranch,
        recovery: WorthQueryProductUnpublishedRecovery,
    },
    Selecting {
        candidate: DetachedExpectedProductBranch,
        exact: WorthQueryProductObservationLease,
        _publication: worth_runtime_world::facade::ConsumedCompositePublication,
        _outputs: Option<WorthQueryCompletedHistoryOutputs>,
    },
    AwaitingOutputs {
        candidate: DetachedExpectedProductBranch,
        exact: WorthQueryProductObservationLease,
        _publication: worth_runtime_world::facade::ConsumedCompositePublication,
    },
    Selected(WorthQueryHistoryNavigationCompletion),
    Discarding(super::disposal::NavigationCleanup),
    Discarded,
}

pub(super) struct NavigationProgress {
    pub(super) pending_slot: Option<super::pending_capacity::PendingNavigationSlot>,
    pub(super) meaning: WorthQueryApplicationIdempotencyBinding,
    pub(super) output_profile: Option<WorthQueryHistoryOutputProfile>,
    pub(super) output_source: Option<super::output_authority::HistoryRestoredOutputBasis>,
    pub(super) output_basis_memory: Option<ExecutionMemoryReservation>,
    pub(super) output_source_memory: Option<ExecutionMemoryReservation>,
    pub(super) expected: WorthQueryHistorySelection,
    pub(super) model: super::super::HistoryModelIdentity,
    /// The installed ownership recipe's identity when the model is an owned graph.
    pub(super) ownership: Option<&'static str>,
    pub(super) source: Option<super::super::WorthQueryHistoryPreview>,
    pub(super) requested: super::super::WorthQueryHistoryNodeId,
    pub(super) intent: WorthQueryHistoryNavigationIntent,
    pub(super) phase: NavigationPhase,
    /// Restored producer custody and its rows, from preparation until World.
    pub(super) output_transport: Option<super::output_custody::HistoryOutputTransport>,
    // Preallocated in the attempt before fork; filled in the adoption transition.
    pub(super) head_slot: Option<u64>,
    pub(super) prior_cleanup:
        Option<crate::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure>,
    pub(super) selection_pause: Option<Arc<super::super::receiver_pause::ReceiverPause>>,
    pub(super) delivery_pause: Option<Arc<super::super::receiver_pause::ReceiverPause>>,
}

impl NavigationProgress {
    pub(super) fn require_resolvable(&self) -> Result<(), WorthQueryHistoryNavigationDenial> {
        match self.phase {
            NavigationPhase::Discarded => Err(WorthQueryHistoryNavigationDenial::Discarded),
            NavigationPhase::Discarding(_) => {
                Err(WorthQueryHistoryNavigationDenial::DisposalInProgress)
            }
            _ => Ok(()),
        }
    }

    pub(super) fn posture(&self) -> WorthQueryHistoryNavigationPosture {
        use WorthQueryHistoryNavigationPosture as P;
        match self.phase {
            NavigationPhase::Prepared => P::Prepared,
            NavigationPhase::ForkRecovery(_) => P::CreatingCandidate,
            NavigationPhase::Candidate(_) => P::PreparingCandidate,
            NavigationPhase::Publication { .. } => P::PublishingCandidate,
            NavigationPhase::Unpublished { .. } => P::RecoveringPublication,
            NavigationPhase::Selecting { .. } => P::AwaitingSelection,
            NavigationPhase::AwaitingOutputs { .. } => P::AwaitingOutputs,
            NavigationPhase::Selected(_) => P::Selected,
            NavigationPhase::Discarding(_) => P::Discarding,
            NavigationPhase::Discarded => P::Discarded,
        }
    }
}

trait NavigationAttemptBacking: Send + Sync {
    fn as_attempt(&self) -> &NavigationAttempt;
}

impl NavigationAttemptBacking for NavigationAttempt {
    fn as_attempt(&self) -> &NavigationAttempt {
        self
    }
}

#[derive(Clone)]
pub(super) struct SharedNavigationAttempt(Arc<dyn NavigationAttemptBacking>);

impl SharedNavigationAttempt {
    pub(super) fn from_arc(attempt: Arc<NavigationAttempt>) -> Self {
        Self(attempt)
    }
}

impl Deref for SharedNavigationAttempt {
    type Target = NavigationAttempt;

    fn deref(&self) -> &Self::Target {
        self.0.as_attempt()
    }
}

pub(super) struct NavigationAttempt {
    pub(super) progress: Mutex<NavigationProgress>,
    pub(super) _allocation: Option<ExecutionMemoryReservation>,
}

impl NavigationAttempt {
    pub(super) fn lock(&self) -> MutexGuard<'_, NavigationProgress> {
        self.progress
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}
