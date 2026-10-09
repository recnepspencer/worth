use super::super::retained_decision_facts::StoreDenial;
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

pub(in crate::domain_computation::primary_graph) fn check_request_live(
    request: &WorthQueryRequestScope,
    subject: &str,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    if let Some(stop) = request.interruption() {
        return Err(StoreDenial::RequestInterruption(stop).into_attempt_denial(subject));
    }
    Ok(())
}
