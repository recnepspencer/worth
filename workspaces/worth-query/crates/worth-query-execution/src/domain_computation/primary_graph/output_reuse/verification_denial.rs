//! Original native output-verification stops at demand admission boundaries.
use crate::domain_computation::primary_graph::vocabulary::invariant_projection::verification::ConsumedOutputVerificationStop as Stop;
use {
    crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial,
    crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind as Kind,
};
pub(in crate::domain_computation::primary_graph) fn verification_denial(
    stop: Stop,
    subject: impl Into<std::borrow::Cow<'static, str>>,
) -> WorthQueryOutputDemandDenial {
    let subject = subject.into();
    match stop {
        Stop::ManagedIndex(cause) => WorthQueryOutputDemandDenial::managed_index(cause, subject),
        Stop::Execution(cause) => super::read_denial(cause),
        Stop::Interrupted(event) => super::interruption_denial(event),
        Stop::SourceQueryAuthorization(kind) => WorthQueryOutputDemandDenial::new(
            Kind::SourceQueryAdmission(
                crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenialKind::Authorization(kind),
            ),
            subject,
        ),
        Stop::CapacityExhausted(cause) => {
            WorthQueryOutputDemandDenial::new(Kind::RetentionBudgetExceeded, subject)
                .with_capacity_stop(Some(cause))
        }
        Stop::WorkExhausted => WorthQueryOutputDemandDenial::new(Kind::WorkBudgetExceeded, subject),
        Stop::RetryCurrentness(_) => {
            WorthQueryOutputDemandDenial::new(Kind::PublicationStale, subject)
        }
        Stop::Unavailable => {
            WorthQueryOutputDemandDenial::new(Kind::RetainedBasisUnavailable, subject)
        }
        Stop::PendingUpstream => {
            WorthQueryOutputDemandDenial::new(Kind::SchedulingDeferred, subject)
        }
    }
}
