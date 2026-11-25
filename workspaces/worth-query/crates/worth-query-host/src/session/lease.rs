//! Metered lease opening for [`WorthQueryHostSession`] execution policies,
//! enforcing installed serial World ceilings whenever no physical pool is present.
use std::num::NonZeroUsize;

use worth_execution::{
    ExecutionAllocationPolicy, ExecutionResourceLease, LeaseDenial, LeaseRequest,
    MemoryLimitDenial, MemoryLimitLevel,
};
use worth_foundational::facade::{ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryAdvancementPhase,
        WorthQueryManagedComputationResourceDenial, WorthQueryMemoryLimitLevel,
    },
    primary_graph::{WorthQueryExecutionLeaseDenial, WorthQueryPrimaryGraphApplicationRuntime},
};

use crate::application_home::DEFAULT_HOST_EXECUTION_POLICY;

/// Computes the default bounded policy for a new [`WorthQueryHostSession`],
/// staying within both [`DEFAULT_HOST_EXECUTION_POLICY`] and the Home's
/// installed execution policy.
pub(super) fn default_session_policy(
    installed: Option<ExecutionRequestPolicy>,
) -> ExecutionRequestPolicy {
    let Some(installed) = installed else {
        return DEFAULT_HOST_EXECUTION_POLICY;
    };
    let default_budget = DEFAULT_HOST_EXECUTION_POLICY
        .accounting()
        .expect("default host execution policy carries a finite budget");
    let installed_budget = installed.accounting();
    let max_workers = default_budget.max_workers().min(installed.max_workers());
    let memory = match (
        default_budget.charged_memory_bytes(),
        installed_budget.and_then(|b| b.charged_memory_bytes()),
    ) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };
    let work = match (
        default_budget.work_ceiling(),
        installed_budget.and_then(|b| b.work_ceiling()),
    ) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };
    let posture = if installed.posture() == ExecutionPosture::Serial {
        ExecutionPosture::Serial
    } else {
        DEFAULT_HOST_EXECUTION_POLICY.posture()
    };
    ExecutionRequestPolicy::new(
        posture,
        installed.determinism(),
        ExecutionBudget::new(max_workers, memory, work),
    )
}

fn constrain_against_installed_serial(
    requested: ExecutionRequestPolicy,
    installed: ExecutionRequestPolicy,
) -> Result<ExecutionRequestPolicy, LeaseDenial> {
    if requested.determinism() != installed.determinism() {
        return Err(LeaseDenial::EquivalenceContractUnavailable);
    }
    if requested.max_workers().get() > installed.max_workers().get() {
        return Err(LeaseDenial::WorkerLimitExceedsParent);
    }
    let installed_budget = installed.accounting();
    let requested_budget = requested.accounting();
    let installed_memory = installed_budget.and_then(|b| b.charged_memory_bytes());
    let requested_memory = requested_budget.and_then(|b| b.charged_memory_bytes());
    if let Some(parent) = installed_memory
        && requested_memory.is_none_or(|child| child > parent)
    {
        return Err(LeaseDenial::MemoryLimitExceedsParent);
    }
    let effective_memory = requested_memory.or(installed_memory);

    let installed_work = installed_budget.and_then(|b| b.work_ceiling());
    let requested_work = requested_budget.and_then(|b| b.work_ceiling());
    if let Some(parent) = installed_work
        && requested_work.is_none_or(|child| child > parent)
    {
        return Err(LeaseDenial::WorkLimitExceedsParent);
    }
    let effective_work = requested_work.or(installed_work);

    let effective = if effective_memory.is_some()
        || effective_work.is_some()
        || requested_budget.is_some()
        || installed_budget.is_some()
    {
        ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            installed.determinism(),
            ExecutionBudget::new(NonZeroUsize::MIN, effective_memory, effective_work),
        )
    } else {
        ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Serial,
            installed.determinism(),
            NonZeroUsize::MIN,
        )
    };
    Ok(effective)
}

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
            let installed =
                graph
                    .installed_execution_policy()
                    .ok_or(WorthQueryAdvancementDenial::Resource(
                        WorthQueryManagedComputationResourceDenial::NoActiveExecutionScope,
                    ))?;
            let bounded =
                constrain_against_installed_serial(policy, installed).map_err(map_lease_denial)?;
            ExecutionResourceLease::serial(LeaseRequest {
                policy: bounded,
                cancellation: request.cancellation,
                deadline: request.deadline,
            })
            .map_err(map_lease_denial)
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
