use worth_store_physical_backend::{
    WalDurabilityObservationDenial, WalDurabilityObservationDenialKind,
};
use worth_store_recovery_physics::{
    ImmutablePhysicalRedoPlan, PhysicalCheckpointBase, PhysicalRedoDecisionKind,
    PhysicalRedoPlanningDenial,
};
use worth_store_recovery_runtime::{RecoveryCompletion, ReopenedPhysicalRecovery};

use super::DurabilityRecoveryAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurabilityOwnerMappingDenial {
    WalFenceFailureRequired,
    ReplayGenerationMismatchRequired,
}

pub const fn map_checkpoint_selection(
    _selection: &PhysicalCheckpointBase,
) -> DurabilityRecoveryAction {
    DurabilityRecoveryAction::CheckpointSelected
}

pub fn map_failed_wal_fence(
    denial: &WalDurabilityObservationDenial,
) -> Result<[DurabilityRecoveryAction; 3], DurabilityOwnerMappingDenial> {
    if denial.kind() != WalDurabilityObservationDenialKind::BarrierFailed {
        return Err(DurabilityOwnerMappingDenial::WalFenceFailureRequired);
    }
    Ok([
        DurabilityRecoveryAction::WalAppendProposed,
        DurabilityRecoveryAction::WalAppendCompletedInMemory,
        DurabilityRecoveryAction::WalFenceRequested,
    ])
}

pub fn map_redo_execution(plan: &ImmutablePhysicalRedoPlan) -> Vec<DurabilityRecoveryAction> {
    let mut actions = vec![DurabilityRecoveryAction::RecoveryReplayRequired];
    let (applied, skipped) = plan.resolved_decisions().fold(
        (false, false),
        |(applied, skipped), decision| match decision.kind() {
            PhysicalRedoDecisionKind::Apply => (true, skipped),
            PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn
            | PhysicalRedoDecisionKind::SkipOperationAlreadyMaterialized => (applied, true),
        },
    );
    if applied {
        actions.push(DurabilityRecoveryAction::RecoveryReplayApplied);
    }
    if skipped {
        actions.push(DurabilityRecoveryAction::RecoveryReplaySkippedIdempotent);
    }
    actions
}

pub fn map_redo_generation_denial(
    denial: &PhysicalRedoPlanningDenial,
) -> Result<DurabilityRecoveryAction, DurabilityOwnerMappingDenial> {
    if matches!(denial, PhysicalRedoPlanningDenial::GenerationMismatch) {
        Ok(DurabilityRecoveryAction::RecoveryReplayRejectedGenerationMismatch)
    } else {
        Err(DurabilityOwnerMappingDenial::ReplayGenerationMismatchRequired)
    }
}

pub const fn map_recovery_completion(
    _completion: &RecoveryCompletion,
) -> [DurabilityRecoveryAction; 2] {
    [
        DurabilityRecoveryAction::RecoveredRootPublicationPending,
        DurabilityRecoveryAction::RecoveredRootPublicationCompleted,
    ]
}

pub const fn map_reopened_physical_recovery(
    _reopened: &ReopenedPhysicalRecovery,
) -> DurabilityRecoveryAction {
    DurabilityRecoveryAction::Reopen
}
