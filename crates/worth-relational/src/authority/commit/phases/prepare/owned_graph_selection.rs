//! Prepare only the full-generation records in the two owned closures.
use super::*;
use crate::mvcc::{
    owned_graph_selection::PreparedOwnedGraphSelection, BranchBoundRelationalTransaction,
};
use crate::transactions::data::RecordRef;
use std::sync::Arc;

pub(crate) fn prepare(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    transaction: &mut BranchBoundRelationalTransaction,
    selected: SelectedRelationalBranchRead<'_>,
    retained: &PreparedOwnedGraphSelection,
) -> Result<PreparedWorkingStateScope, TransactionCommitError> {
    transaction
        .footprint
        .record_owned_graph_selection(
            retained.meaning.targets(),
            retained.meaning.partitions(),
            execution.allocation_policy(),
        )
        .map_err(|denial| TransactionCommitError::conflict(denial.into_conflict()))?;
    let mut partitions = BTreeSet::new();
    let mut slots: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
    for record in retained.meaning.targets() {
        execution
            .consult()
            .map_err(|cause| TransactionCommitError::execution(cause.into()))?;
        let partition = match record {
            RecordRef::Entity(id) => {
                slots
                    .entry(id.partition_id)
                    .or_default()
                    .insert(id.slot_index());
                id.partition_id
            }
            RecordRef::Relation(id) => id.partition_id,
        };
        partitions.insert(partition);
        retained
            .graph
            .entities()
            .selected_basis()
            .inner
            .root
            .demand_head_partition(execution, partition)
            .map_err(|cause| {
                crate::authority::commit::preparation::storage_read_denial(
                    retained.graph.entities().selected_basis(),
                    cause,
                )
            })?;
    }
    for partition in retained.meaning.partitions() {
        partitions.insert(partition);
        use crate::storage::overlay::CurrentPartitionAccess;
        selected.get_current_partition(partition);
    }
    let working_state = WorkingState::from_touched_partitions_with_layout_and_sparse_slots(
        &selected,
        partitions.iter().copied(),
        runtime.config.storage.adjacency_policy.clone(),
        PartitionCloneMode::GraphSparseEntities,
        EntityWorkingSetLayout::CanonicalSoA,
        Some(&slots),
        None,
    );
    let plan = PreparedRelationalCommitPlan::OwnedGraphSelection {
        commands: crate::mvcc::OwnedNormalizedPlan::empty(transaction.transaction_id),
        retained: retained.clone(),
    };
    Ok(PreparedWorkingStateScope {
        selected_branch_state: selected.selected().clone(),
        schema_authority: Arc::clone(transaction.mutation_schema_authority()),
        structural_summary: CommitStructuralSummary {
            invariant_contract: plan.view().invariant_contract(),
            commit_topology: CommitTopology::GraphMutation,
            touched_partitions: partitions,
            bulk_entity_slots_reserved: 0,
            bulk_relation_slots_reserved: 0,
        },
        plan,
        working_state,
        phase_timing: CommitPhaseTiming::default(),
        footprint: transaction.footprint.clone(),
    })
}
