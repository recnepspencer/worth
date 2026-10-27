//! Preserve a measured scope's domain error or its original execution category.
use super::publication_execution_kind;
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as Kind,
};
use worth_execution::{MapKernelFailure, ScopeStop, WorkCeilingDenial};

pub(in crate::domain_computation) fn scope_stop_kind<E>(
    stop: ScopeStop<E>,
) -> Result<E, Result<Kind, Control>> {
    Err(match stop {
        ScopeStop::Failure(MapKernelFailure::Domain(cause)) => return Ok(cause),
        ScopeStop::Admission(cause) => publication_execution_kind(WorkCeilingDenial::Admission(cause)),
        ScopeStop::Failure(MapKernelFailure::Stop(cause)) => publication_execution_kind(WorkCeilingDenial::Stopped(cause)),
        ScopeStop::Failure(MapKernelFailure::Panic) => publication_execution_kind(WorkCeilingDenial::Panicked),
        ScopeStop::Failure(MapKernelFailure::ResultCapacityExceeded) => Ok(Kind::ExecutionResource {
            denial: crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::ResultCapacityExceeded,
            partition_identity: None,
            policy_ancestor: None,
        }),
    })
}
