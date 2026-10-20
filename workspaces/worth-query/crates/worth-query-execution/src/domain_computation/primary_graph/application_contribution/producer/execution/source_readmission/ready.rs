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
            | Stop::RetainedCompanionCounterOverflow { .. }
            | Stop::RetainedCompanionCapacityExhausted { .. } => {
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
            }
        },
        subject,
    )
    .with_capacity_stop(VerificationCapacityStop::of_meter(stop))
    .with_producer_admission_stop(Some(stop))
    .into()
}

pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn ready_currentness_denial(
    subject: &'static str,
    stop: CurrentAcceptedStop,
) -> ProducerExecutionStop {
    use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop;
    let native_stop = match &stop {
        CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(stop))
        | CurrentAcceptedStop::Closure(ConsumedOutputVerificationStop::CapacityExhausted(
            VerificationCapacityStop::Meter(stop),
        )) => Some(*stop),
        CurrentAcceptedStop::Closure(_) | CurrentAcceptedStop::Registration(_) => None,
    };
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
        | CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(worth_relational::facade::mvcc::CompanionPreflightStop::Execution(cause))) => return crate::domain_computation::primary_graph::output_reuse::read_denial(cause)
            .with_producer_admission_stop(native_stop).into(),
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
        .with_producer_admission_stop(native_stop)
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
        for (stop, kept, native) in [
            (
                CurrentAcceptedStop::Closure(Verification::CapacityExhausted(allocation)),
                allocation,
                None,
            ),
            (
                CurrentAcceptedStop::Closure(Verification::CapacityExhausted(Meter(meter))),
                Meter(meter),
                Some(meter),
            ),
            (
                CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(meter)),
                Meter(meter),
                Some(meter),
            ),
        ] {
            let actual = ready_currentness_denial("producer", stop).into_output_denial();
            assert_eq!(actual.producer_admission_stop(), native);
            assert_eq!(actual.output_verification_capacity_stop(), Some(kept));
            assert_eq!(actual.publication_companion_stop(), None);
            assert_eq!(
                actual,
                refused()
                    .with_capacity_stop(Some(kept))
                    .with_producer_admission_stop(native)
            );
        }
        assert_eq!(
            ready_resource_denial("producer", meter).into_output_denial(),
            refused()
                .with_capacity_stop(Some(Meter(meter)))
                .with_producer_admission_stop(Some(meter))
        );
        // A non-capacity stop retains its native identity without a capacity quote.
        let pending = CompanionPreflightStop::TopologyPending;
        assert_eq!(
            ready_currentness_denial(
                "producer",
                CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(pending)),
            )
            .into_output_denial(),
            refused().with_producer_admission_stop(Some(pending))
        );
    }

    #[test]
    fn ready_readmission_keeps_native_structural_stops_and_their_origin() {
        use worth_relational::facade::{history::CommitId, mvcc::RelationalInterruptionBoundary};
        let cancelled =
            worth_relational::facade::mvcc::RelationalInterruptionEvent::from_request_stop(
                worth_execution::MapKernelStop::Cancelled,
                RelationalInterruptionBoundary::PublicationPreflight,
            )
            .unwrap();
        for (native, kind) in [
            (
                CompanionPreflightStop::TopologyPending,
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
            ),
            (
                CompanionPreflightStop::SelectedSourceMismatch,
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
            ),
            (
                CompanionPreflightStop::SelectedPositionUnavailable {
                    commit_id: CommitId(7),
                },
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
            ),
            (
                CompanionPreflightStop::ForeignCell,
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
            ),
            (
                CompanionPreflightStop::RegistrationChanged,
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
            ),
            (
                CompanionPreflightStop::Interrupted(cancelled),
                WorthQueryOutputDemandDenialKind::Cancelled,
            ),
        ] {
            let registration =
                CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(native));
            for actual in [
                ready_resource_denial("producer", native).into_output_denial(),
                ready_currentness_denial("producer", registration).into_output_denial(),
            ] {
                assert_eq!(actual.kind(), kind);
                assert_eq!(actual.subject(), "producer");
                assert_eq!(
                    actual.recovery_posture(),
                    crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Terminal
                );
                assert_eq!(actual.producer_admission_stop(), Some(native));
                assert_eq!(actual.publication_companion_stop(), None);
                assert_eq!(actual.clone(), actual);
            }
        }
    }

    #[test]
    fn ready_execution_stop_keeps_its_original_mapping_and_native_origin() {
        use worth_relational::facade::transactions::RelationalExecutionDenialCause;
        let cause = RelationalExecutionDenialCause::WorkExhausted;
        let native = CompanionPreflightStop::Execution(cause);
        let registration =
            CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(native));
        let actual = ready_currentness_denial("producer", registration).into_output_denial();
        let original = crate::domain_computation::primary_graph::output_reuse::read_denial(cause);
        let resource = ready_resource_denial("producer", native).into_output_denial();
        assert_eq!(resource.kind(), original.kind());
        assert_eq!(resource.subject(), "producer");
        assert_eq!(resource.producer_admission_stop(), Some(native));
        assert_eq!(resource.publication_companion_stop(), None);
        assert_eq!(
            actual,
            original.clone().with_producer_admission_stop(Some(native))
        );
        assert_eq!(actual.publication_companion_stop(), None);
        assert_eq!(actual.producer_admission_stop(), Some(native));
        assert_eq!(
            ready_currentness_denial(
                "producer",
                CurrentAcceptedStop::Closure(Verification::Execution(cause))
            )
            .into_output_denial(),
            original,
        );
    }
}
