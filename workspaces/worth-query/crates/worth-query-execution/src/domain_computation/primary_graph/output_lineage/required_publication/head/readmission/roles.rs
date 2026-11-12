//! Borrow the cold descriptor and fund every temporary Native role target.
use super::*;
use crate::domain_computation::primary_graph::{
    provider::native_publication_roles, source_facts::SourceFactsAllocation,
    WorthQueryApplicationOutputCorrespondence,
};
use worth_execution::{ExecutionArray, ExecutionArrayBuilder, ExecutionRequest};
pub(super) struct ReadmissionRoles {
    pub(super) targets: ExecutionArray<RequiredOutputRoleTarget>,
    _names: Arc<SourceFactsAllocation>,
}
pub(super) fn retain(
    execution: &ExecutionRequest<'_, '_>,
    layout: &WorthQueryPrimaryGraphLayout,
    correspondence: &WorthQueryApplicationOutputCorrespondence,
    family: &str,
) -> Result<ReadmissionRoles, RequiredPublicationReadmissionStop> {
    let roles = correspondence.native_witness_roles();
    let names = SourceFactsAllocation::open(execution).map_err(control)?;
    let mut bytes = u64::try_from(family.len()).map_err(|_| overflow())?;
    for (role, _, name, _) in roles.clone() {
        SourceFactsAllocation::work(execution, 1).map_err(control)?;
        for len in [role.len(), name.len()] {
            bytes = bytes
                .checked_add(u64::try_from(len).map_err(|_| overflow())?)
                .ok_or_else(overflow)?;
        }
    }
    SourceFactsAllocation::work(execution, bytes).map_err(control)?;
    names.claim(execution, bytes).map_err(control)?;
    let policy = execution.allocation_policy();
    let mut targets = ExecutionArrayBuilder::allocate(roles.len(), policy).map_err(allocation)?;
    for (role, posture, name, id) in roles {
        policy.check_live().map_err(allocation)?;
        let reference = EntityReference::Existing(id);
        let target = native_publication_roles::target(layout, role, posture, name, &reference)
            .ok_or(RequiredPublicationReadmissionStop::Selection(
                Stop::SelectedSourceMismatch,
            ))?;
        targets.push(target).map_err(allocation)?;
    }
    Ok(ReadmissionRoles {
        targets: targets.seal().map_err(allocation)?,
        _names: names,
    })
}
fn control(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> RequiredPublicationReadmissionStop {
    RequiredPublicationReadmissionStop::Selection(Stop::Execution(cause))
}
fn overflow() -> RequiredPublicationReadmissionStop {
    RequiredPublicationReadmissionStop::Selection(Stop::PreparationMemoryCounterOverflow)
}
fn allocation(
    denial: worth_execution::ExecutionAllocationDenial,
) -> RequiredPublicationReadmissionStop {
    RequiredPublicationReadmissionStop::Allocation(denial)
}
