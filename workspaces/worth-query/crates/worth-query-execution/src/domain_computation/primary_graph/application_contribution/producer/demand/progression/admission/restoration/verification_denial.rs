//! Typed request failures from restored checkpoint comparison.
use super::*;
pub(super) fn from_stop(
    stop: crate::domain_computation::primary_graph::output_lineage::invalidation::SettlementVerificationStop,
) -> WorthQueryOutputDemandDenial {
    use crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure as Read;
    use crate::domain_computation::primary_graph::output_lineage::invalidation::SettlementVerificationStop as Stop;
    match stop {
        Stop::SourceRead(Read::OutputObservation(stop)) => {
            crate::domain_computation::primary_graph::output_reuse::verification_denial(
                stop,
                "checkpoint comparison",
            )
        }
        Stop::SourceRead(Read::Execution(cause)) => {
            crate::domain_computation::primary_graph::output_reuse::read_denial(cause)
        }
        Stop::Admission(stop) => restoration_resource_denial(stop),
        Stop::SourceRead(Read::WorkBudgetExceeded) => denial(
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
            "checkpoint comparison",
        ),
        Stop::SourceRead(Read::Unavailable)
        | Stop::Alignment
        | Stop::Edit(_)
        | Stop::PendingUpstream => denial(
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
            "checkpoint comparison",
        ),
    }
}
