//! Freshness forwards the captured read stop; ordinary refusals remain stale.
use crate::domain_computation::primary_graph::vocabulary::application_query::authorized_read::WorthQueryAuthorizedApplicationReadDenial as Denial;
use {crate::domain_computation::primary_graph::vocabulary::entity_resolution_denial::WorthQueryEntityResolutionDenial, crate::domain_computation::vocabulary::primary_graph::entity_resolution_denial::WorthQueryEntityResolutionDenialKind as Kind};
pub(super) fn freshness_denial(denial: WorthQueryEntityResolutionDenial, stale: Denial) -> Denial {
    match denial.kind() {
        Kind::Stopped(cause) => Denial::Read(super::super::read_execution::relational_read_denial(
            cause,
            denial.subject(),
        )),
        Kind::Refused(_) => stale,
    }
}
