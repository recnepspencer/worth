use std::cell::RefCell;

use worth_store::physical_runtime::{
    ClosedPhysicalRecoveryCleanup, StoreRecoveryCleanupAttempt,
    StoreRecoveryCleanupFreshnessFailure, StoreRecoveryCleanupPlan,
};

use crate::progression::ReopenedPhysicalRecovery;

use super::RecoveryCleanupEligibility;

/// Borrowed, proof-bearing Store inputs used to construct one exact cleanup
/// command. Runtime eligibility narrows the candidate set; the Store command
/// independently revalidates checkpoint coverage before accepting it.
pub(super) struct RecoveryCleanupCommandBasis {
    state: RecoveryCleanupCommandState,
}

enum RecoveryCleanupCommandState {
    Active(RefCell<StoreRecoveryCleanupPlan>),
    AdmissionDenied {
        closed: ClosedPhysicalRecoveryCleanup,
        failure: RefCell<Option<StoreRecoveryCleanupFreshnessFailure>>,
    },
    Deferred(ClosedPhysicalRecoveryCleanup),
}

pub(super) enum StoreCleanupCommandExecution {
    Attempt(StoreRecoveryCleanupAttempt),
    AdmissionDenied(StoreRecoveryCleanupFreshnessFailure),
    Unavailable,
}

impl RecoveryCleanupCommandBasis {
    pub(super) fn from_reopened(
        reopened: &mut ReopenedPhysicalRecovery,
        descriptive_plan_identity: [u8; 32],
        candidates: &[RecoveryCleanupEligibility],
    ) -> Self {
        let checkpoint = reopened
            .state
            .selection
            .checkpoint()
            .map(|checkpoint| checkpoint.share_checkpoint());
        let fresh_reopen = reopened.take_fresh_reopen();
        let admitted_wal = reopened
            .state
            .integrity
            .admitted_wal()
            .cleanup_segments(candidates.iter().map(RecoveryCleanupEligibility::artifact));
        let coordination = reopened.state.coordination.owner();
        let state = match (checkpoint, candidates.is_empty()) {
            (_, true) => RecoveryCleanupCommandState::Deferred(coordination.defer_cleanup(
                fresh_reopen,
                descriptive_plan_identity,
                0,
            )),
            (Some(checkpoint), false) => match coordination.admit_cleanup_plan(
                &reopened.state.authority.media,
                fresh_reopen,
                checkpoint,
                descriptive_plan_identity,
                admitted_wal,
            ) {
                Ok(plan) => RecoveryCleanupCommandState::Active(RefCell::new(plan)),
                Err(failure) => {
                    let (closed, failure) = failure.into_parts();
                    RecoveryCleanupCommandState::AdmissionDenied {
                        closed,
                        failure: RefCell::new(Some(failure)),
                    }
                }
            },
            (None, false) => RecoveryCleanupCommandState::Deferred(coordination.defer_cleanup(
                fresh_reopen,
                descriptive_plan_identity,
                0,
            )),
        };
        Self { state }
    }

    pub(super) fn plan_identity(&self) -> Option<[u8; 32]> {
        match &self.state {
            RecoveryCleanupCommandState::Active(plan) => Some(plan.borrow().identity()),
            RecoveryCleanupCommandState::AdmissionDenied { .. }
            | RecoveryCleanupCommandState::Deferred(_) => None,
        }
    }

    pub(super) fn descriptive_plan_identity(&self) -> [u8; 32] {
        match &self.state {
            RecoveryCleanupCommandState::Active(plan) => plan.borrow().descriptive_plan_identity(),
            RecoveryCleanupCommandState::AdmissionDenied { closed, .. } => {
                closed.descriptive_plan_identity()
            }
            RecoveryCleanupCommandState::Deferred(closed) => closed.descriptive_plan_identity(),
        }
    }

    pub(super) fn terminal_binding_evaluations(&self) -> u64 {
        match &self.state {
            RecoveryCleanupCommandState::Active(plan) => {
                plan.borrow().terminal_binding_evaluations()
            }
            RecoveryCleanupCommandState::AdmissionDenied { failure, .. } => {
                failure.borrow().as_ref().map_or(
                    0,
                    StoreRecoveryCleanupFreshnessFailure::terminal_binding_evaluations,
                )
            }
            RecoveryCleanupCommandState::Deferred(_) => 0,
        }
    }

    pub(super) fn execute(
        &self,
        coordination: &worth_store::physical_runtime::PhysicalRecoveryCoordination,
        media: &worth_store::physical_runtime::AdmittedRecoveryFilesystemMedia,
        artifact: worth_store::physical_runtime::recovery_wal::WalSegmentArtifactIdentity,
    ) -> StoreCleanupCommandExecution {
        match &self.state {
            RecoveryCleanupCommandState::Active(plan) => StoreCleanupCommandExecution::Attempt(
                coordination.execute_cleanup_candidate(media, &mut plan.borrow_mut(), artifact),
            ),
            RecoveryCleanupCommandState::AdmissionDenied { failure, .. } => failure
                .borrow_mut()
                .take()
                .map_or(StoreCleanupCommandExecution::Unavailable, |failure| {
                    StoreCleanupCommandExecution::AdmissionDenied(failure)
                }),
            RecoveryCleanupCommandState::Deferred(_) => StoreCleanupCommandExecution::Unavailable,
        }
    }

    pub(super) fn live_media_handle_delta(
        &self,
        media: &worth_store::physical_runtime::AdmittedRecoveryFilesystemMedia,
    ) -> u64 {
        match &self.state {
            RecoveryCleanupCommandState::Active(plan) => {
                plan.borrow().live_media_handle_delta(media)
            }
            RecoveryCleanupCommandState::AdmissionDenied { .. }
            | RecoveryCleanupCommandState::Deferred(_) => 0,
        }
    }

    pub(super) fn close(self, live_media_handle_delta: u64) -> ClosedPhysicalRecoveryCleanup {
        match self.state {
            RecoveryCleanupCommandState::Active(plan) => {
                plan.into_inner().close(live_media_handle_delta)
            }
            RecoveryCleanupCommandState::AdmissionDenied { closed, .. } => closed,
            RecoveryCleanupCommandState::Deferred(closed) => closed,
        }
    }
}
