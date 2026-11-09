//! The conventional producer's C principal precedes its exact D source check.
use super::*;
use crate::domain_computation::primary_graph::{
    product_operation::document_history::HistoryProducerInvocation,
    WorthQueryApplicationEntityIdentity, WorthQueryApplicationQueryAdmissionDenialKind as Query,
};

pub(super) fn require_source<S: ApplicationSchema, E>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    history: HistoryProducerInvocation<'_>,
    scope: &WorthQueryApplicationEntityIdentity<S, E>,
    truth: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    subject: &str,
) -> Result<(), SelectedConventionalAdmissionStop> {
    use crate::domain_computation::authorization::{
        WorthQueryOperationAuthorizationDenial as Denial,
        WorthQueryOperationAuthorizationDenialKind as Kind,
        WorthQueryOperationAuthorizationRefusalKind as Refusal,
    };
    runtime.runtime.primary_provider.graph.with_runtime(execution, |native| {
        history.require_historical_source(execution, native, scope, truth, runtime, subject).map_err(|cause| {
            let kind = match cause.kind() {
                Query::ExecutionRequest(kind) => Kind::Stopped(Ok(kind)),
                Query::Cancelled => Kind::Stopped(Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled)),
                Query::DeadlineExceeded => Kind::Stopped(Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut)),
                _ => Kind::Refused(Refusal::StaleScope),
            };
            SelectedConventionalAdmissionStop::Authorization(Denial::new(kind, subject))
        })
    })
}
