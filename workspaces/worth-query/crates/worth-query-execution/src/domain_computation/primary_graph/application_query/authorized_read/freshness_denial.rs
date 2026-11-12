//! Freshness forwards the captured read stop; ordinary refusals remain stale.
use super::WorthQueryAuthorizedApplicationReadDenial as Denial;
use crate::domain_computation::primary_graph::{
    WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind as Kind,
};
pub(super) fn freshness_denial(
    denial: WorthQueryEntityResolutionDenial,
    check: super::super::WorthQueryScopeFreshnessCheck,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
) -> Denial {
    match denial.kind() {
        Kind::Stopped(cause) => Denial::Read(super::super::read_execution::relational_read_denial(
            cause,
            denial.subject(),
        )),
        Kind::Refused(_) => Denial::ScopeFreshness(
            super::super::WorthQueryScopeFreshnessDiagnostic::new(check, denial, snapshot),
        ),
    }
}
