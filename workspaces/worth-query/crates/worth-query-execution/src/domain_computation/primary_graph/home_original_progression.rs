//! Keep the same original and invoking request alive across physical Pending steps.
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::{
    RuntimeWorldRecoveryPort, WorldRecoveredContinuationOutcome as Continuation,
    WorldRecoveredReleaseOutcome as Release, WorldRecoveredUnpublishedEffects,
};

pub(super) fn continue_original(
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

pub(super) fn release_original(
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
