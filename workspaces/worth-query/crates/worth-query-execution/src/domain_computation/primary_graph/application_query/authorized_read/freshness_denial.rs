//! A freshness interruption keeps its read stop; structural loss keeps its stale posture.
use super::super::read_execution::{
    read_execution_denial, WorthQueryApplicationReadExecutionDenialKind as Read,
};
use super::WorthQueryAuthorizedApplicationReadDenial as Denial;
use crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption as Stop;
pub(super) fn freshness_denial(denial: WorthQueryEntityResolutionDenial, stale: Denial) -> Denial {
    let kind = match denial.interruption() {
        Some(Stop::Cancelled) => Read::Cancelled,
        Some(Stop::DeadlineExceeded) => Read::DeadlineExceeded,
        None => return stale,
    };
    Denial::Read(read_execution_denial(kind, denial.subject()))
}
