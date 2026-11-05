//! Moved whole from `domain_computation::primary_graph::application_contribution::execution_denial::scope`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::application_contribution::publication::publication_execution_kind;
use crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind as Control;
use crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind as Kind;
use worth_execution::MapKernelFailure;
use worth_execution::ScopeStop;
use worth_execution::WorkCeilingDenial;

pub(in crate::domain_computation) fn scope_stop_kind<E>(
    stop: ScopeStop<E>,
) -> Result<E, Result<Kind, Control>> {
    Err(match stop {
        ScopeStop::Failure(MapKernelFailure::Domain(cause)) => return Ok(cause),
        ScopeStop::Admission(cause) => publication_execution_kind(WorkCeilingDenial::Admission(cause)),
        ScopeStop::Failure(MapKernelFailure::Stop(cause)) => publication_execution_kind(WorkCeilingDenial::Stopped(cause)),
        ScopeStop::Failure(MapKernelFailure::Panic) => publication_execution_kind(WorkCeilingDenial::Panicked),
        ScopeStop::Failure(MapKernelFailure::ResultCapacityExceeded) => Ok(Kind::ExecutionResource {
            denial: crate::domain_computation::vocabulary::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial::ResultCapacityExceeded,
            partition_identity: None,
            policy_ancestor: None,
        }),
    })
}
