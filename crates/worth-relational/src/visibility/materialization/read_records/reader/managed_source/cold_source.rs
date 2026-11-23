//! Required evidence before an explicit rebuild can create a missing Source.
use super::{Denial, EntityId, ManagedIndexSourceProgram};
use crate::{
    branch::RelationalBranchRoot,
    derived_relation::DerivedRelationReadDenial,
    execution::{request_stage, RelationalExecutionDenialCause},
    runtime::{
        RelationalBorrowedRecordReadDenial, SchemaContractRuntimeSubsystem,
        VisibilityProjectionView,
    },
};
use worth_execution::ExecutionRequest;

pub(super) fn admit(
    contracts: &SchemaContractRuntimeSubsystem,
    execution: &ExecutionRequest<'_, '_>,
    root: &RelationalBranchRoot,
    projection: &VisibilityProjectionView,
    program: &ManagedIndexSourceProgram,
    scope: EntityId,
) -> Result<(), Denial> {
    if root.has_materialization_unavailable() {
        return Err(Denial::RootUnavailable);
    }
    let sources = &contracts.custom_invariant_registries.source_history;
    let quote = projection
        .exact_entity_state_read_work_bound()
        .and_then(|bound| bound.checked_add(sources.len() as u64))
        .ok_or(Denial::RootUnavailable)?;
    request_stage::optional_stage(
        execution,
        |context, _| {
            request_stage::checkpoint(context, quote).map_err(|stop| request(stop.into()))?;
            sources.slot(program).map_err(|_| Denial::ForeignSource)?;
            Ok::<(), Denial>(())
        },
        |_| 0,
    )
    .map_err(request)??;
    let scope_state = projection
        .read_exact_entity_state(execution, scope, |metadata, _| metadata)
        .map_err(|cause| match cause {
            RelationalBorrowedRecordReadDenial::Execution(cause) => request(cause),
            RelationalBorrowedRecordReadDenial::ExactBasisRequired => Denial::ForeignSource,
        })?
        .ok_or(Denial::RootUnavailable)?;
    if scope_state.entity_id != scope || scope_state.kind_id != program.scope_kind() {
        return Err(Denial::ForeignSource);
    }
    Ok(())
}

fn request(cause: RelationalExecutionDenialCause) -> Denial {
    Denial::Source(DerivedRelationReadDenial::Request(cause))
}
