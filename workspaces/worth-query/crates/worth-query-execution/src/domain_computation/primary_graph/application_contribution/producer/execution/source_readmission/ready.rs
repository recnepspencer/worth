use super::super::super::demand::ProducerPublicationStale;
use super::*;
use crate::domain_computation::primary_graph::invariant_projection::VerificationCapacityStop;

pub(super) fn ready_work_denial(subject: &'static str) -> ProducerExecutionStop {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        subject,
    )
    .into()
}

pub(in crate::domain_computation::primary_graph::application_contribution::producer::execution) fn ready_resource_denial(
    subject: &'static str,
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> ProducerExecutionStop {
    use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
    WorthQueryOutputDemandDenial::new(
        match stop {
            Stop::Execution(cause) => {
                crate::domain_computation::primary_graph::output_reuse::read_denial(cause).kind()
            }
            Stop::WorkExhausted { .. }
            | Stop::WorkExhaustedWithoutRequirement { .. }
            | Stop::WorkCounterOverflow => WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
            Stop::Interrupted(event) => {
                WorthQueryOutputDemandDenialKind::of_interruption(event.interruption())
            }
            Stop::TopologyPending
            | Stop::SelectedSourceMismatch
            | Stop::SelectedPositionUnavailable { .. }
            | Stop::ForeignCell
            | Stop::RegistrationChanged
            | Stop::PreparationMemoryExhausted { .. }
            | Stop::PreparationMemoryCounterOverflow
            | Stop::RetainedCompanionCapacityExhausted { .. } => {
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
            }
        },
        subject,
    )
    .with_capacity_stop(VerificationCapacityStop::of_meter(stop))
    .into()
}

pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn ready_currentness_denial(
    subject: &'static str,
    stop: CurrentAcceptedStop,
) -> ProducerExecutionStop {
    use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop;
    let capacity = match &stop {
        CurrentAcceptedStop::Closure(ConsumedOutputVerificationStop::CapacityExhausted(cause)) => {
            Some(*cause)
        }
        CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(meter)) => {
            VerificationCapacityStop::of_meter(*meter)
        }
        CurrentAcceptedStop::Closure(_) | CurrentAcceptedStop::Registration(_) => None,
    };
    let stale = match &stop {
        CurrentAcceptedStop::Closure(ConsumedOutputVerificationStop::RetryCurrentness(cause)) => {
            Some(ProducerPublicationStale::ReadyCurrentnessClosure(*cause))
        }
        CurrentAcceptedStop::Registration(SettlementRegistrationStop::Edit(cause)) => Some(
            ProducerPublicationStale::ReadyCurrentnessRegistration(*cause),
        ),
        CurrentAcceptedStop::Closure(_) | CurrentAcceptedStop::Registration(_) => None,
    };
    let kind = match stop {
        CurrentAcceptedStop::Closure(stop @ ConsumedOutputVerificationStop::SourceQueryAuthorization(_)) => return crate::domain_computation::primary_graph::output_reuse::verification_denial(stop, subject.to_owned()).into(),
        CurrentAcceptedStop::Closure(ConsumedOutputVerificationStop::Execution(cause))
        | CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(worth_relational::facade::mvcc::CompanionPreflightStop::Execution(cause))) => return crate::domain_computation::primary_graph::output_reuse::read_denial(cause).into(),
        CurrentAcceptedStop::Closure(ConsumedOutputVerificationStop::WorkExhausted)
        | CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(
            worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted { .. } | worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhaustedWithoutRequirement { .. }
            | worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow,
        )) => WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        CurrentAcceptedStop::Closure(ConsumedOutputVerificationStop::Interrupted(event))
        | CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(
            worth_relational::facade::mvcc::CompanionPreflightStop::Interrupted(event),
        )) => WorthQueryOutputDemandDenialKind::of_interruption(event.interruption()),
        CurrentAcceptedStop::Closure(ConsumedOutputVerificationStop::PendingUpstream) => {
            WorthQueryOutputDemandDenialKind::SchedulingDeferred
        }
        CurrentAcceptedStop::Closure(ConsumedOutputVerificationStop::Unavailable)
        | CurrentAcceptedStop::Registration(
            SettlementRegistrationStop::Alignment(_)
            | SettlementRegistrationStop::SourceUnavailable,
        ) => WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
        CurrentAcceptedStop::Registration(SettlementRegistrationStop::Foreign) => {
            WorthQueryOutputDemandDenialKind::ForeignSettlement
        }
        CurrentAcceptedStop::Closure(ConsumedOutputVerificationStop::RetryCurrentness(_))
        | CurrentAcceptedStop::Registration(SettlementRegistrationStop::Edit(_)) => {
            WorthQueryOutputDemandDenialKind::PublicationStale
        }
        CurrentAcceptedStop::Closure(ConsumedOutputVerificationStop::CapacityExhausted(_))
        | CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(_)) => {
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
        }
    };
    WorthQueryOutputDemandDenial::new(kind, subject)
        .with_capacity_stop(capacity)
        .with_publication_stale(stale)
        .into()
}

#[cfg(test)]
mod tests {
    use super::VerificationCapacityStop::{Meter, ScratchAllocationRefused};
    use super::*;
    use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop as Verification;
    use worth_relational::facade::mvcc::CompanionPreflightStop;

    /// Ready readmission keeps the verifier's capacity cause, with the meter's
    /// own amounts, as evidence; its kind and posture are unchanged.
    #[test]
    fn ready_readmission_keeps_the_typed_capacity_cause() {
        let meter = CompanionPreflightStop::PreparationMemoryExhausted {
            required: 2,
            maximum: 1,
        };
        let refused = || {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                "producer",
            )
        };
        let allocation = ScratchAllocationRefused { bytes: 8 };
        for (stop, kept) in [
            (
                CurrentAcceptedStop::Closure(Verification::CapacityExhausted(allocation)),
                allocation,
            ),
            (
                CurrentAcceptedStop::Closure(Verification::CapacityExhausted(Meter(meter))),
                Meter(meter),
            ),
            (
                CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(meter)),
                Meter(meter),
            ),
        ] {
            assert_eq!(
                ready_currentness_denial("producer", stop).into_output_denial(),
                refused().with_capacity_stop(Some(kept))
            );
        }
        assert_eq!(
            ready_resource_denial("producer", meter).into_output_denial(),
            refused().with_capacity_stop(Some(Meter(meter)))
        );
        // A stop that shares the kind but is not a capacity stop keeps none.
        let pending = CompanionPreflightStop::TopologyPending;
        assert_eq!(
            ready_currentness_denial(
                "producer",
                CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(pending)),
            )
            .into_output_denial(),
            refused()
        );
    }
}
