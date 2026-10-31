use super::RuntimeWorldOwnerRoot;
use crate::recovery::{
    RuntimeWorldRecoveryDenial as Denial, WorldRecoveredReleaseOutcome as Outcome,
    WorldRecoveredUnpublishedEffects,
};

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn release_recovered_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        effects: WorldRecoveredUnpublishedEffects,
    ) -> Outcome {
        if !self.state.recovery.admits_recovered(&effects) {
            return Outcome::AdmissionRefused {
                cause: Denial::ForeignHandle,
                custody: effects,
            };
        }
        let _operation = match self.reserve_recovery_operation_if_open_and_bootstrapped() {
            Ok(operation) => operation,
            Err(()) => {
                return Outcome::AdmissionRefused {
                    cause: super::super::super::RuntimeWorldOwnerUnavailable::new().into(),
                    custody: effects,
                }
            }
        };
        effects.release_original(execution)
    }
}
