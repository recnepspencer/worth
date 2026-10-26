use crate::authority::commit::phases::mutation::branch_local_delete_allowance_for_plan;
use crate::authority::mutation::apply_plan_to_working_state;
use crate::branch::SelectedRelationalBranchState;
use crate::runtime::{RelationalPreparationRuntime, WorkingState};
use crate::transactions::data::{AuthoritativeApplyPlan, MergedCommitPlan, TransactionCommitError};

/// Detached authoritative application and the correspondence minted by that apply.
#[derive(Debug)]
pub(crate) struct AppliedInvariantState {
    working_state: WorkingState,
    created_entities: crate::transactions::data::CommitCreatedEntityBindings,
    record_allocations: Option<crate::runtime::PendingRecordAllocations>,
}
impl AppliedInvariantState {
    pub(crate) fn take_record_allocations(&mut self) -> crate::runtime::PendingRecordAllocations {
        self.record_allocations
            .take()
            .expect("move proposed allocation custody once")
    }

    pub(crate) fn working_state(&self) -> &WorkingState {
        &self.working_state
    }
    pub(crate) fn created_entities(
        &self,
    ) -> &crate::transactions::data::CommitCreatedEntityBindings {
        &self.created_entities
    }
}

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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    materialization_mode: Option<crate::mvcc::RelationalMaterializationTransactionMode>,
) -> Result<AppliedInvariantState, TransactionCommitError> {
    let lease = crate::authority::commit::pipeline::request_lease(execution);
    let apply_plan = AuthoritativeApplyPlan {
        transaction_id: merged_plan.transaction_id,
        version_id,
        merged_intents: merged_plan.merged_intents.clone(),
    };
    let mutation_config = crate::config::data::MutationConfig {
        cascade_delete_policy: runtime.config.storage.cascade_delete_policy,
        adjacency_policy: runtime.config.storage.adjacency_policy.clone(),
        cross_context_policy: runtime.config.storage.cross_context_policy,
    };
    let mut proposed = working_state.clone();
    crate::aggregate_identity::guard::ordinary_plan(
        execution,
        runtime.commit_work_budget.as_ref(),
        schema_authority.registry(),
        &crate::validation::engine::state_view::InvariantStateView::new(
            selected_state,
            selected_state.version_id(),
        ),
        &merged_plan.merged_intents,
        materialization_mode,
    )?;
    let allowance = branch_local_delete_allowance_for_plan(selected_state, &proposed, merged_plan);
    let mut symbols = runtime.services.symbols.interner_snapshot();
    let mut record_allocations = runtime.record_identity.begin_allocations();
    let applied = apply_plan_to_working_state(
        &mut proposed,
        &apply_plan,
        &mutation_config,
        schema_authority.registry(),
        schema_authority.aspect_plans(),
        &mut symbols,
        allowance,
        &mut record_allocations,
        lease,
        runtime.commit_work_budget.clone(),
    )?;
    record_allocations
        .prepare_reapplication()
        .map_err(|denial| {
            TransactionCommitError::conflict(crate::transactions::data::CommitConflict::new(
                crate::transactions::data::ConflictClass::RecordAllocationDenied { denial },
            ))
        })?;
    crate::authority::mutation::apply_adjacency_deltas(
        &mut proposed,
        &applied.effect.adjacency.deltas,
        version_id,
    );
    Ok(AppliedInvariantState {
        working_state: proposed,
        created_entities: applied.created_entities,
        record_allocations: Some(record_allocations),
    })
}
