//! Lossless native source-output stop in a decision dependency comparison.
use super::*;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop as Stop;
impl WorthQueryApplicationAttemptDenial {
    pub(in crate::domain_computation::primary_graph) fn output_observation(stop: Stop) -> Self {
        let kind = match stop {
            Stop::Execution(cause) => return Self::relational_read_stopped(cause, "source output observation"),
            Stop::SourceQueryAuthorization(kind) => return Self::request_authority_lost(
                crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::new(kind, "source query permission")),
            Stop::Interrupted(event) => match event.interruption() {
                worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled => WorthQueryApplicationAttemptDenialKind::Cancelled,
                worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut => WorthQueryApplicationAttemptDenialKind::DeadlineExceeded,
            },
            Stop::WorkExhausted | Stop::CapacityExhausted(_) => WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
            Stop::RetryCurrentness(_) => WorthQueryApplicationAttemptDenialKind::SourceChanged,
            Stop::Unavailable | Stop::PendingUpstream => WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
        };
        Self {
            kind,
            subject: "source output observation".to_owned(),
            cause: AttemptDenialCause::OutputObservation(Box::new(stop)),
        }
    }
}
