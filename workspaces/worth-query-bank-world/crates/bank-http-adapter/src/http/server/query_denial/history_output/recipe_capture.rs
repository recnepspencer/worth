//! Original normalized recipe capture retains its parameter and request refusal.
use super::*;
use worth_query_host::facade::primary_graph::WorthQueryApplicationAttemptDenial;

pub(super) fn denial(cause: WorthQueryApplicationAttemptDenial) -> BankHttpDenial {
    if let Some(parameter) = cause.source_disclosure_parameter_denial() {
        return super::super::parameter(BankApplicationQueryParameterDenialKind::from_query(
            parameter.kind(),
        ));
    }
    if let Some(allocation) = cause.allocation_denial() {
        return managed_index::allocation(allocation);
    }
    // capture_parameters projects request stops with their original session kind.
    // Its only authored domain refusal here is an unavailable installed binding;
    // the caller's parameter projection is Ok(intent.parameters()), not a body callback.
    crate::http::server::estate_denial::attempt_kind(cause.kind().into())
}
