//! Prepare only the full-generation records in the two owned closures.
use worth_relational_mutation::internal_port as mutation;
use mutation::mvcc::transaction::footprint::retained_selection::RelationalTransactionFootprintOwnerPort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::owned_graph_selection::CanonicalOwnedGraphSelectionOwnerPort as _;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use super::*;
use mutation::mvcc::owned_graph_selection::prepared::PreparedOwnedGraphSelection;
use worth_relational_contracts::internal_port::transactions::data::primitives::RecordRef;

pub struct SelectionInputs<'a>(pub super::working_state_inputs::SelectionTransaction<'a>);

pub fn prepare<'a, Transaction>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: impl Into<super::working_state_inputs::WorkingStatePreparation<'a>>,
    transaction: &mut Transaction,
    selected: SelectedRelationalBranchRead<'_>,
    retained: &PreparedOwnedGraphSelection,
) -> Result<PreparedWorkingStateScope, TransactionCommitError>
where
    for<'t> &'t mut Transaction: Into<SelectionInputs<'t>>,
{
    let inputs = inputs.into();
    let transaction = transaction.into().0;
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
            .owner_inner()
            .root
            .demand_head_partition(execution, partition)
            .map_err(|cause| {
                worth_relational_branch::internal_port::authority::commit::preparation::storage_denial::storage_read_denial(
                    retained.graph.entities().selected_basis(),
                    cause,
                )
            })?;
    }
    for partition in retained.meaning.partitions() {
        partitions.insert(partition);
        use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;
        selected.get_current_partition(partition);
    }
    let working_state = WorkingState::from_touched_partitions_with_layout_and_sparse_slots(
        &selected,
        partitions.iter().copied(),
        inputs.adjacency_policy.clone(),
        PartitionCloneMode::GraphSparseEntities,
        EntityWorkingSetLayout::CanonicalSoA,
        Some(&slots),
        None,
    );
    let plan = PreparedRelationalCommitPlan::OwnedGraphSelection {
        commands: mutation::mvcc::transaction::normalized_plan::OwnedNormalizedPlan::empty(
            transaction.transaction_id,
        ),
        retained: retained.clone(),
    };
    Ok(PreparedWorkingStateScope {
        selected_branch_state: selected.selected().clone(),
        schema_authority: transaction.schema,
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
