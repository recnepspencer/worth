use super::super::RecordPublicationDirector;
use super::indeterminate;
use crate::physical_runtime::{
    PhysicalMutationAttempt, PhysicalMutationIndeterminateStage,
    PhysicalMutationPreSealAdmissionDetail, PhysicalMutationProvenNoEffectCause,
    PhysicalMutationTerminalFact, PhysicalPreSealCancellationOutcome, PreparedPhysicalMutation,
};

impl RecordPublicationDirector {
    pub(super) fn pre_seal_denial(
        &self,
        attempt: &PhysicalMutationAttempt,
    ) -> Option<PhysicalMutationProvenNoEffectCause> {
        if let Some(stop) = attempt.original_request_stop() {
            return Some(match stop {
                worth_execution::MapKernelStop::DeadlineElapsed => {
                    PhysicalMutationProvenNoEffectCause::DeadlineElapsedBeforeGroupSeal
                }
                _ => PhysicalMutationProvenNoEffectCause::CancelledBeforeGroupSeal,
            });
        }
        if attempt.cancellation_requested() {
            return Some(PhysicalMutationProvenNoEffectCause::CancelledBeforeGroupSeal);
        }
        match self.deadline_elapsed(attempt) {
            Ok(true) => Some(PhysicalMutationProvenNoEffectCause::DeadlineElapsedBeforeGroupSeal),
            Err(()) => Some(PhysicalMutationProvenNoEffectCause::WorkerUnavailableBeforeGroupSeal),
            Ok(false) => None,
        }
    }

    pub(super) fn deadline_elapsed(&self, attempt: &PhysicalMutationAttempt) -> Result<bool, ()> {
        self.runtime
            .upgrade()
            .ok_or(())?
            .signal
            .clock_observation()
            .map(|clock| attempt.deadline().signal_deadline().get() <= clock.current_tick())
            .map_err(|_| ())
    }

    pub(super) fn pre_effect_terminal(
        &self,
        prepared: PreparedPhysicalMutation,
        attempt: &PhysicalMutationAttempt,
        cause: PhysicalMutationProvenNoEffectCause,
    ) -> PhysicalMutationTerminalFact {
        self.pre_effect_terminal_with_admission_detail(prepared, attempt, cause, None)
    }

    pub(super) fn pre_effect_terminal_with_admission_detail(
        &self,
        prepared: PreparedPhysicalMutation,
        attempt: &PhysicalMutationAttempt,
        cause: PhysicalMutationProvenNoEffectCause,
        detail: Option<PhysicalMutationPreSealAdmissionDetail>,
    ) -> PhysicalMutationTerminalFact {
        match self.settle_prepared_before_group_seal(prepared, cause) {
            PhysicalPreSealCancellationOutcome::ProvenNoEffect(terminal) => {
                attempt.install_terminal(PhysicalMutationTerminalFact::ProvenNoEffect(
                    terminal.clone(),
                ));
                let terminal = match detail {
                    Some(detail) => terminal.with_admission_detail(detail),
                    None => terminal,
                };
                PhysicalMutationTerminalFact::ProvenNoEffect(terminal)
            }
            PhysicalPreSealCancellationOutcome::NotCancelled { .. } => {
                indeterminate(attempt, PhysicalMutationIndeterminateStage::WalAppend, 0)
            }
        }
    }
}
