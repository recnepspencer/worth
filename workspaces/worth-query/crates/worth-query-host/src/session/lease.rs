//! Metered lease opening for [`WorthQueryHostSession`] execution policies.
use worth_execution::{
    ExecutionAllocationPolicy, ExecutionResourceLease, LeaseDenial, LeaseRequest,
    MemoryLimitDenial, MemoryLimitLevel,
};
use worth_foundational::facade::ExecutionRequestPolicy;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryAdvancementPhase,
        WorthQueryManagedComputationResourceDenial, WorthQueryMemoryLimitLevel,
    },
    primary_graph::{WorthQueryExecutionLeaseDenial, WorthQueryPrimaryGraphApplicationRuntime},
};

pub(super) fn open_policy_lease<'a, Schema: ApplicationSchema>(
    graph: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    policy: ExecutionRequestPolicy,
    scope: &WorthQueryRequestScope,
) -> Result<ExecutionResourceLease<'a>, WorthQueryAdvancementDenial> {
    let request = LeaseRequest {
        policy,
        cancellation: scope.cancellation().execution_token(),
        deadline: scope.deadline(),
    };
    match graph.request_execution_lease(request.clone()) {
        Ok(lease) => Ok(lease),
        Err(WorthQueryExecutionLeaseDenial::Unavailable) => {
            ExecutionResourceLease::serial(request).map_err(map_lease_denial)
        }
        Err(WorthQueryExecutionLeaseDenial::Lease(denial)) => Err(map_lease_denial(denial)),
    }
}

pub(super) fn with_policy_advancement<Schema: ApplicationSchema, R>(
    graph: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    policy: ExecutionRequestPolicy,
    scope: &WorthQueryRequestScope,
    body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
) -> Result<R, WorthQueryAdvancementDenial> {
    let lease = open_policy_lease(graph, policy, scope)?;
    graph.with_funded_application_advancement(
        scope,
        ExecutionAllocationPolicy::Execution(&lease),
        body,
    )
}

fn map_lease_denial(denial: LeaseDenial) -> WorthQueryAdvancementDenial {
    use WorthQueryManagedComputationResourceDenial as Resource;
    let resource = match denial {
        LeaseDenial::WorkerLimitExceedsParent => Resource::WorkerLimit,
        LeaseDenial::MemoryLimitExceedsParent => Resource::PolicyMemoryLimit,
        LeaseDenial::WorkLimitExceedsParent => Resource::WorkLimit,
        LeaseDenial::MemoryExhausted(MemoryLimitDenial::CounterOverflow { .. }) => {
            Resource::ChargedBytesOverflow
        }
        LeaseDenial::MemoryExhausted(MemoryLimitDenial::Capacity {
            requested,
            admitted,
            level,
        }) => Resource::MemoryLimit {
            requested,
            admitted,
            level: match level {
                MemoryLimitLevel::Policy { .. } => WorthQueryMemoryLimitLevel::Policy,
                MemoryLimitLevel::Process => WorthQueryMemoryLimitLevel::Process,
                MemoryLimitLevel::Declared => WorthQueryMemoryLimitLevel::Declared,
            },
        },
        LeaseDenial::ChargedBytesOverflow => Resource::ChargedBytesOverflow,
        LeaseDenial::UnrelatedNestedLease => Resource::NestedLeaseMisuse,
        LeaseDenial::NoActiveExecutionScope => Resource::NoActiveExecutionScope,
        LeaseDenial::EquivalenceContractUnavailable => Resource::EquivalenceContractUnavailable,
    };
    WorthQueryAdvancementDenial::Resource(resource)
}
