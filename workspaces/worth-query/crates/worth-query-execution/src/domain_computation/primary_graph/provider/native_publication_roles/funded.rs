//! Fund the new immutable role snapshot before releasing the attempt loan.
use super::*;
use crate::domain_computation::WorthQueryInvariantExecutionFailure as Failure;
use worth_execution::{
    ExecutionAllocationPolicy, ExecutionArray, ExecutionArrayBuilder, ExecutionMemoryReservation,
};

pub(in crate::domain_computation::primary_graph::provider) struct FundedPublicationRoles {
    pub(in crate::domain_computation::primary_graph::provider) roles:
        ExecutionArray<RequiredOutputRoleTarget>,
    // The array and its deep strings drop before their deep-heap reservation.
    _names: Option<ExecutionMemoryReservation>,
}

pub(in crate::domain_computation::primary_graph::provider) fn retain<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    layout: &WorthQueryPrimaryGraphLayout,
    targets: impl ExactSizeIterator<Item = (&'a str, Posture, &'a str, &'a EntityReference)> + Clone,
    producer_bytes: usize,
    policy: ExecutionAllocationPolicy<'_, '_>,
) -> Result<FundedPublicationRoles, Failure> {
    let mut bytes = u64::try_from(producer_bytes).map_err(|_| overflow())?;
    for (role, _, name, target) in targets.clone() {
        policy.check_live().map_err(allocation)?;
        work(execution, 1)?;
        let client = match target {
            EntityReference::Existing(_) => 0,
            EntityReference::Created(created) => {
                created.client_key.as_raw_str().map_or(0, str::len)
            }
        };
        for count in [role.len(), name.len(), client] {
            bytes = bytes
                .checked_add(u64::try_from(count).map_err(|_| overflow())?)
                .ok_or_else(overflow)?;
        }
    }
    work(execution, bytes)?;
    let names = policy.reserve_memory(bytes).map_err(allocation)?;
    let mut roles = ExecutionArrayBuilder::allocate(targets.len(), policy).map_err(allocation)?;
    for (role, posture, name, reference) in targets {
        policy.check_live().map_err(allocation)?;
        let role = target(layout, role, posture, name, reference).ok_or_else(|| {
            super::super::invariant_execution::closure_failure(
                "producer role names an uninstalled entity",
            )
        })?;
        roles.push(role).map_err(allocation)?;
    }
    Ok(FundedPublicationRoles {
        roles: roles.seal().map_err(allocation)?,
        _names: names,
    })
}
fn allocation(cause: worth_execution::ExecutionAllocationDenial) -> Failure {
    Failure::physical_allocation(
        cause,
        "producer publication role snapshot allocation refused",
    )
}
fn overflow() -> Failure {
    super::super::invariant_execution_failure::companion_failure(
        worth_relational::facade::mvcc::CompanionPreflightStop::PreparationMemoryCounterOverflow,
    )
}

fn work(execution: &worth_execution::ExecutionRequest<'_, '_>, units: u64) -> Result<(), Failure> {
    crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation::work(
        execution, units,
    )
    .map_err(|cause| {
        super::super::invariant_execution_failure::companion_failure(
            worth_relational::facade::mvcc::CompanionPreflightStop::Execution(cause),
        )
    })
}
