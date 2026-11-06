//! Every progression posture owns its actual created branch or World continuation.
use super::*;
use std::sync::{Mutex, MutexGuard};
use worth_execution::ExecutionMemoryReservation;
use {
    crate::basis::WorthQueryProductObservationLease,
    crate::domain_computation::execution_runtime::product_world::WorthQueryPreparedProductPublication,
    crate::domain_computation::primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding,
    crate::domain_computation::primary_graph::WorthQueryExpectedProductBranch,
    crate::domain_computation::WorthQueryProductUnpublishedRecovery,
};

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

pub(super) enum NavigationPhase<'runtime, S> {
    Prepared,
    ForkRecovery(ForkCleanup),
    Candidate(WorthQueryExpectedProductBranch<'runtime, S>),
    Publication {
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
        publication: WorthQueryPreparedProductPublication,
    },
    Unpublished {
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
        recovery: WorthQueryProductUnpublishedRecovery,
    },
    Selecting {
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
        exact: WorthQueryProductObservationLease,
        _publication: worth_runtime_world::facade::ConsumedCompositePublication,
        _outputs: Option<WorthQueryCompletedHistoryOutputs>,
    },
    AwaitingOutputs {
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
        exact: WorthQueryProductObservationLease,
        _publication: worth_runtime_world::facade::ConsumedCompositePublication,
    },
    Selected(WorthQueryHistoryNavigationCompletion),
    Discarding(super::disposal::NavigationCleanup<'runtime, S>),
    Discarded,
}

pub(super) struct NavigationProgress<'runtime, S> {
    pub(super) pending_slot: Option<super::pending_capacity::PendingNavigationSlot>,
    pub(super) application: &'runtime dyn crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<S>,
    pub(super) meaning: WorthQueryApplicationIdempotencyBinding,
    pub(super) output_profile: Option<WorthQueryHistoryOutputProfile>,
    pub(super) output_source:
        Option<Arc<crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation>>,
    pub(super) output_source_memory: Option<ExecutionMemoryReservation>,
    pub(super) expected: WorthQueryHistorySelection,
    pub(super) model: super::super::HistoryModelIdentity,
    pub(super) source: Option<super::super::WorthQueryHistoryPreview>,
    pub(super) requested: super::super::WorthQueryHistoryNodeId,
    pub(super) intent: WorthQueryHistoryNavigationIntent,
    pub(super) phase: NavigationPhase<'runtime, S>,
    // Preallocated in the attempt before fork; filled in the adoption transition.
    pub(super) head_slot: Option<u64>,
    pub(super) prior_cleanup:
        Option<crate::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure>,
    pub(super) selection_pause: Option<Arc<super::super::receiver_pause::ReceiverPause>>,
    pub(super) delivery_pause: Option<Arc<super::super::receiver_pause::ReceiverPause>>,
}

impl<S> NavigationProgress<'_, S> {
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

pub(super) struct NavigationAttempt<'runtime, S> {
    pub(super) progress: Mutex<NavigationProgress<'runtime, S>>,
    pub(super) _allocation: Option<ExecutionMemoryReservation>,
}

impl<'runtime, S> NavigationAttempt<'runtime, S> {
    pub(super) fn lock(&self) -> MutexGuard<'_, NavigationProgress<'runtime, S>> {
        self.progress
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}
