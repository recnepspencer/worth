//! Existing input-cutoff denial mappings.
use super::*;

/// Basis preparation may refuse its first claim, so no diagnostic allocation
/// can depend on a budget that has already been exhausted.
pub(super) fn cutoff_basis_denial(
    stop: InputCutoffVerificationStop,
) -> WorthQueryOutputDemandDenial {
    use worth_relational::facade::mvcc::CompanionPreflightStop as Admission;
    let kind = match stop {
        InputCutoffVerificationStop::OutputObservation(stop) => {
            return crate::domain_computation::primary_graph::output_reuse::verification_denial(
                stop,
                "producer input cutoff",
            )
        }
        InputCutoffVerificationStop::Admission(Admission::WorkCounterOverflow)
        | InputCutoffVerificationStop::WorkExhausted => {
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        }
        InputCutoffVerificationStop::Admission(Admission::Interrupted(event)) => {
            WorthQueryOutputDemandDenialKind::of_interruption(event.interruption())
        }
        InputCutoffVerificationStop::CapacityExhausted
        | InputCutoffVerificationStop::Admission(_) => {
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
        }
        InputCutoffVerificationStop::Execution(cause) => {
            return crate::domain_computation::primary_graph::output_reuse::read_denial(cause)
        }
        InputCutoffVerificationStop::SelectedSourceUnavailable => {
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable
        }
        _ => WorthQueryOutputDemandDenialKind::PublicationStale,
    };
    denial(kind, String::new())
}

pub(super) fn cutoff_admission_denial(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> WorthQueryOutputDemandDenial {
    use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
    denial(
        match stop {
            Stop::WorkCounterOverflow => WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
            Stop::Interrupted(event) => {
                WorthQueryOutputDemandDenialKind::of_interruption(event.interruption())
            }
            _ => WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        },
        String::new(),
    )
}

pub(super) fn cutoff_denial(stop: InputCutoffVerificationStop) -> WorthQueryOutputDemandDenial {
    match stop {
        InputCutoffVerificationStop::OutputObservation(stop) => crate::domain_computation::primary_graph::output_reuse::verification_denial(stop, "producer input cutoff"),
        InputCutoffVerificationStop::Execution(cause) => crate::domain_computation::primary_graph::output_reuse::read_denial(cause),
        InputCutoffVerificationStop::Admission(stop) => cutoff_admission_denial(stop),
        InputCutoffVerificationStop::CapacityExhausted => denial(WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded, String::new()),
        InputCutoffVerificationStop::WorkExhausted => denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, String::new()),
        InputCutoffVerificationStop::PendingOutput(identity) => denial(
            WorthQueryOutputDemandDenialKind::SchedulingDeferred,
            format!("producer input cutoff awaits upstream output {identity:?}"),
        ).with_recovery_posture(crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable),
        InputCutoffVerificationStop::PendingUpstream => denial(WorthQueryOutputDemandDenialKind::SchedulingDeferred, "producer input cutoff awaits upstream outputs")
            .with_recovery_posture(crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable),
        _ => denial(WorthQueryOutputDemandDenialKind::PublicationStale, "producer input cutoff source moved"),
    }
}

/// A stopped caller relinquishes pre-effect custody rather than failing shared work.
pub(super) fn cutoff_verification_stop(stop: InputCutoffVerificationStop) -> ProducerExecutionStop {
    let denial = cutoff_denial(stop);
    if matches!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::Cancelled | WorthQueryOutputDemandDenialKind::TimedOut
    ) {
        super::super::denial::request_admission_rejected(denial)
    } else {
        ProducerExecutionStop::ExecutionStopped(denial)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
    #[test]
    fn paid_cutoff_control_stops_relinquish_only_the_caller() {
        for (cause, kind) in [
            (
                Cause::Cancelled,
                WorthQueryOutputDemandDenialKind::Cancelled,
            ),
            (
                Cause::DeadlineElapsed,
                WorthQueryOutputDemandDenialKind::TimedOut,
            ),
        ] {
            let ProducerExecutionStop::RequestAdmissionDenied(stop) =
                cutoff_verification_stop(InputCutoffVerificationStop::Execution(cause))
            else {
                panic!("pre-effect interruption cannot fail shared output work");
            };
            let denial = stop.into_denial();
            assert_eq!(denial.kind(), kind);
            assert_eq!(denial.recovery_posture(), crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable);
        }
        assert!(matches!(
            cutoff_verification_stop(InputCutoffVerificationStop::Execution(
                Cause::WorkCounterOverflow
            )),
            ProducerExecutionStop::ExecutionStopped(_)
        ));
    }
}
