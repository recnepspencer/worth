//! Prepare the real selection scope without inventing an authored command.
use super::PreparedWorkingStateScope;
use crate::authority::commit::semantic_plan::{
    PreparedRelationalCommitPlan, PreparedRetainedEntityPayload,
};
use crate::authority::commit::structural_summary::CommitStructuralSummary;
use crate::branch::selected_read::SelectedRelationalBranchRead;
use crate::storage::overlay::{EntityWorkingSetLayout, PartitionCloneMode, WorkingState};
use crate::transactions::data::mutation_planning::CommitTopology;
use crate::transactions::data::outcomes::commit_result::CommitPhaseTiming;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use std::collections::{BTreeMap, BTreeSet};
use worth_relational_contracts::internal_port::mvcc::transaction::staging::RelationalTransactionStagingDenialOwnerPort as _;
use worth_relational_mutation::internal_port::mvcc::transaction::footprint::retained_selection::RelationalTransactionFootprintOwnerPort as _;

pub(crate) struct SelectionInputs<'a>(
    pub(crate) super::working_state_inputs::SelectionTransaction<'a>,
);

pub(crate) fn prepare<'a, Transaction>(
    inputs: impl Into<super::working_state_inputs::WorkingStatePreparation<'a>>,
    transaction: &mut Transaction,
    selected: SelectedRelationalBranchRead<'_>,
    retained: &PreparedRetainedEntityPayload,
    policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<PreparedWorkingStateScope, TransactionCommitError>
where
    for<'t> &'t mut Transaction: Into<SelectionInputs<'t>>,
{
    let inputs = inputs.into();
    let transaction = transaction.into().0;
    let entity = retained.meaning.entity();
    transaction
        .footprint
        .record_retained_entity_selection(entity, retained.meaning.kind(), policy)
        .map_err(|denial| TransactionCommitError::conflict(denial.into_conflict()))?;
    let plan = PreparedRelationalCommitPlan::RetainedEntitySelection {
        commands: crate::mvcc::transaction::normalized_plan::OwnedNormalizedPlan::empty(
            transaction.transaction_id,
        ),
        retained: retained.clone(),
    };
    let partitions = BTreeSet::from([entity.partition_id]);
    let slots = BTreeMap::from([(entity.partition_id, BTreeSet::from([entity.slot_index()]))]);
    let working_state = WorkingState::from_touched_partitions_with_layout_and_sparse_slots(
        &selected,
        partitions.iter().copied(),
        inputs.adjacency_policy.clone(),
        PartitionCloneMode::EntityOnly,
        EntityWorkingSetLayout::CanonicalSoA,
        Some(&slots),
        None,
    );
    let structural_summary = CommitStructuralSummary {
        invariant_contract: plan.view().invariant_contract(),
        commit_topology: CommitTopology::FlatEntityBatch,
        touched_partitions: partitions,
        bulk_entity_slots_reserved: 0,
        bulk_relation_slots_reserved: 0,
    };
    Ok(PreparedWorkingStateScope {
        selected_branch_state: selected.selected().clone(),
        plan,
        structural_summary,
        working_state,
        phase_timing: CommitPhaseTiming::default(),
        footprint: transaction.footprint.clone(),
        schema_authority: transaction.schema,
    })
}
