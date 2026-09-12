use crate::authority::commit::phases::mutation::branch_local_delete_allowance_for_plan;
use crate::authority::mutation::apply_plan_to_working_state;
use crate::branch::SelectedRelationalBranchState;
use crate::runtime::{RelationalPreparationRuntime, WorkingState};
use crate::transactions::data::{AuthoritativeApplyPlan, MergedCommitPlan, TransactionCommitError};

/// Build the detached post-plan state used by commit-boundary invariants.
///
/// The mutation engine remains the sole interpreter of mutation intents.  The
/// caller's ordinary working set is cloned before application, and the symbol
/// table is also cloned, so proposal construction cannot publish effects while
/// it prepares invariant evidence.
pub(crate) fn prepare_proposed_invariant_state(
    runtime: &RelationalPreparationRuntime,
    selected_state: &SelectedRelationalBranchState,
    working_state: &WorkingState,
    merged_plan: &MergedCommitPlan,
    schema_authority: &crate::branch::RelationalBranchRootSchemaAuthority,
    version_id: crate::identity::data::VersionId,
) -> Result<WorkingState, TransactionCommitError> {
    let apply_plan = AuthoritativeApplyPlan {
        transaction_id: merged_plan.transaction_id,
        version_id,
        merged_intents: merged_plan.merged_intents.clone(),
    };
    let mutation_config = crate::config::data::MutationConfig {
        cascade_delete_policy: runtime.config.storage.cascade_delete_policy,
        adjacency_policy: runtime.config.storage.adjacency_policy.clone(),
        cross_context_policy: runtime.config.storage.cross_context_policy,
        execution_model: runtime.config.execution.execution_model,
    };
    let mut proposed = working_state.clone();
    let allowance = branch_local_delete_allowance_for_plan(selected_state, &proposed, merged_plan);
    let mut symbols = runtime.services.symbols.interner_snapshot();
    let mut record_allocations = crate::runtime::PendingRecordAllocations::new(
        crate::runtime::RuntimeSubsystem::fork(&runtime.record_identity),
        None,
    );
    let applied = apply_plan_to_working_state(
        &mut proposed,
        &apply_plan,
        &mutation_config,
        schema_authority.registry(),
        schema_authority.aspect_plans(),
        &mut symbols,
        allowance,
        &mut record_allocations,
    )
    .map_err(TransactionCommitError::conflict)?;
    crate::authority::mutation::apply_adjacency_deltas(
        &mut proposed,
        &applied.effect.adjacency.deltas,
    );
    Ok(proposed)
}
