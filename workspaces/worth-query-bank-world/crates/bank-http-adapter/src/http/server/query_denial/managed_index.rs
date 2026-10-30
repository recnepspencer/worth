//! HTTP projection of the managed receiver's original owned denial.
use super::*;
pub(super) fn managed_index(
    cause: &worth_query_host::facade::primary_graph::WorthQueryManagedIndexDenial,
) -> BankHttpDenial {
    use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestInterruption as Stop;
    use worth_query_host::facade::primary_graph::WorthQueryManagedIndexDenialCause as Cause;
    match cause.cause() {
        Cause::Request(stop) => crate::http::server::advancement_denial::request(stop),
        Cause::Interrupted(Stop::Cancelled) => cancelled(),
        Cause::Interrupted(Stop::DeadlineExceeded) => deadline(),
        Cause::Execution(cause) => crate::http::server::relational_execution_denial::denial(*cause),
        Cause::Installation(cause) => query_denial(BankApplicationQueryDenial::from_request_query(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::BindingInstallation(cause.clone()))),
        Cause::Product(cause) => query_denial(BankApplicationQueryDenial::from_request_query(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::ProductSelection(*cause))),
        Cause::Principal(cause) => query_denial(BankApplicationQueryDenial::from_request_query(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::PrincipalResolution(cause.clone()))),
        Cause::Scope(cause) => query_denial(BankApplicationQueryDenial::from_request_query(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::ScopeResolution(cause.clone()))),
        Cause::QueryAdmission(cause) => query_denial(BankApplicationQueryDenial::from_request_query(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::Admission(cause.clone()))),
        Cause::QueryRead(cause) => query_denial(BankApplicationQueryDenial::from_request_query(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::Execution(cause.clone()))),
        Cause::Refused(_) | Cause::GraphWork(_) | Cause::Source(_) | Cause::Work(_) => unavailable(),
    }
}
