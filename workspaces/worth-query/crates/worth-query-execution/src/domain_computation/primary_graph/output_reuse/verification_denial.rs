//! Original native output-verification stops at demand admission boundaries.
use super::super::invariant_projection::ConsumedOutputVerificationStop as Stop;
use super::super::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind as Kind};
pub(in crate::domain_computation::primary_graph) fn verification_denial(
    stop: Stop,
    subject: impl Into<std::borrow::Cow<'static, str>>,
) -> WorthQueryOutputDemandDenial {
    let subject = subject.into();
    match stop {
        Stop::Execution(cause) => super::read_denial(cause),
        Stop::Interrupted(event) => super::interruption_denial(event),
        Stop::SourceQueryAuthorization(kind) => WorthQueryOutputDemandDenial::new(
            Kind::SourceQueryAdmission(
                super::super::WorthQueryApplicationQueryAdmissionDenialKind::Authorization(kind),
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
