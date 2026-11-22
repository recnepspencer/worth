//! Keep the same original and invoking request alive across physical Pending
//! steps, and issue the completed cleanup receipt only when World itself
//! publishes that continuation.

use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, RuntimeWorldRecoveryPort, WorldRecoveredContinuationDenial,
    WorldRecoveredContinuationOutcome as Continuation, WorldRecoveredReleaseOutcome as Release,
    WorldRecoveredUnpublishedEffects,
};

use super::WorthQueryProductBranchOwnerCleanupReceipt;

/// Terminal or request-stopped outcome of continuing a recovered World original,
/// pairing the published commit with the completed owner-cleanup receipt only
/// when World itself published that continuation.
pub enum ContinuedOriginalOutcome {
    Published {
        commit: CompositeCommitIdentity,
        cleanup: WorthQueryProductBranchOwnerCleanupReceipt,
    },
    Pending(WorldRecoveredUnpublishedEffects),
    Refused {
        cause: WorldRecoveredContinuationDenial,
        custody: WorldRecoveredUnpublishedEffects,
    },
    EvidenceRequired(WorldRecoveredUnpublishedEffects),
    Indeterminate(WorldRecoveredUnpublishedEffects),
}

pub fn continue_original(
    recovery: &RuntimeWorldRecoveryPort,
    execution: &ExecutionRequest<'_, '_>,
    mut original: WorldRecoveredUnpublishedEffects,
) -> Continuation {
    loop {
        match recovery.continue_recovered_effects(execution, original) {
            Continuation::Pending(next) => {
                if execution.consult().is_err() {
                    return Continuation::Pending(next);
                }
                original = next;
                std::thread::yield_now();
            }
            terminal @ (Continuation::Published { .. }
            | Continuation::Refused { .. }
            | Continuation::EvidenceRequired(_)
            | Continuation::Indeterminate(_)) => return terminal,
        }
    }
}

pub fn continue_original_with_cleanup(
    recovery: &RuntimeWorldRecoveryPort,
    execution: &ExecutionRequest<'_, '_>,
    original: WorldRecoveredUnpublishedEffects,
) -> ContinuedOriginalOutcome {
    match continue_original(recovery, execution, original) {
        Continuation::Published { commit } => ContinuedOriginalOutcome::Published {
            commit,
            cleanup: WorthQueryProductBranchOwnerCleanupReceipt::continued_original(),
        },
        Continuation::Pending(next) => ContinuedOriginalOutcome::Pending(next),
        Continuation::Refused { cause, custody } => {
            ContinuedOriginalOutcome::Refused { cause, custody }
        }
        Continuation::EvidenceRequired(next) => ContinuedOriginalOutcome::EvidenceRequired(next),
        Continuation::Indeterminate(next) => ContinuedOriginalOutcome::Indeterminate(next),
    }
}

pub fn release_original(
    recovery: &RuntimeWorldRecoveryPort,
    execution: &ExecutionRequest<'_, '_>,
    mut original: WorldRecoveredUnpublishedEffects,
) -> Release {
    loop {
        match recovery.release_recovered_effects(execution, original) {
            Release::Pending(next) => {
                if execution.consult().is_err() {
                    return Release::Pending(next);
                }
                original = next;
                std::thread::yield_now();
            }
            terminal @ (Release::Released(_)
            | Release::AdmissionRefused { .. }
            | Release::Refused { .. }
            | Release::EvidenceRequired(_)
            | Release::Indeterminate(_)) => return terminal,
        }
    }
}
