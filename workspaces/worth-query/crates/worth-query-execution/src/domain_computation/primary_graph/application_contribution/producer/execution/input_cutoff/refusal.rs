//! Existing input-cutoff denial mappings.
use super::*;

/// Basis preparation may refuse its first claim, so no diagnostic allocation
/// can depend on a budget that has already been exhausted.
pub(super) fn cutoff_basis_denial(
    stop: InputCutoffVerificationStop,
) -> WorthQueryOutputDemandDenial {
    use worth_relational::facade::mvcc::CompanionPreflightStop as Admission;
    let kind = match stop {
        InputCutoffVerificationStop::Admission(
            Admission::WorkExhausted { .. }
            | Admission::WorkExhaustedWithoutRequirement { .. }
            | Admission::WorkCounterOverflow,
        )
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
            Stop::WorkExhausted { .. }
            | Stop::WorkExhaustedWithoutRequirement { .. }
            | Stop::WorkCounterOverflow => WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
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
