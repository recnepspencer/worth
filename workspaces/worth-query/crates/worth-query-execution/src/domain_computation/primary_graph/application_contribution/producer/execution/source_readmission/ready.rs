use super::*;

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
            Stop::WorkExhausted { .. } | Stop::WorkCounterOverflow => {
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
            }
            _ => WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        },
        subject,
    )
    .into()
}

pub(super) fn ready_currentness_denial(
    subject: &'static str,
    stop: CurrentAcceptedStop,
) -> ProducerExecutionStop {
    use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop;
    let kind = match stop {
        CurrentAcceptedStop::Closure(ConsumedOutputVerificationStop::WorkExhausted)
        | CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(
            worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted { .. }
            | worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow,
        )) => WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
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
        CurrentAcceptedStop::Registration(SettlementRegistrationStop::Admission(_)) => {
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
        }
    };
    WorthQueryOutputDemandDenial::new(kind, subject).into()
}
