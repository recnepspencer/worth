//! Prepare the real selection scope without inventing an authored command.
use super::{PreparedWorkingStateScope, RelationalPreparationRuntime};
use crate::authority::commit::semantic_plan::{
    PreparedRelationalCommitPlan, PreparedRetainedEntityPayload,
};
use crate::authority::commit::structural_summary::CommitStructuralSummary;
use crate::branch::SelectedRelationalBranchRead;
use crate::mvcc::BranchBoundRelationalTransaction;
use crate::storage::overlay::{EntityWorkingSetLayout, PartitionCloneMode, WorkingState};
use crate::transactions::data::{
    CommitPhaseTiming, CommitTopology, MergedCommitPlan, TransactionCommitError,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub(crate) fn prepare(
    runtime: &RelationalPreparationRuntime,
    transaction: &mut BranchBoundRelationalTransaction,
    selected: SelectedRelationalBranchRead<'_>,
    retained: &PreparedRetainedEntityPayload,
    policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<PreparedWorkingStateScope, TransactionCommitError> {
    let entity = retained.meaning.entity();
    transaction
        .footprint
        .record_retained_entity_selection(entity, retained.meaning.kind(), policy)
        .map_err(|denial| TransactionCommitError::conflict(denial.into_conflict()))?;
    let plan = PreparedRelationalCommitPlan::RetainedEntitySelection {
        commands: MergedCommitPlan {
            transaction_id: transaction.transaction_id,
            merged_intents: Vec::new(),
        },
        retained: retained.clone(),
    };
    let partitions = BTreeSet::from([entity.partition_id]);
    let slots = BTreeMap::from([(entity.partition_id, BTreeSet::from([entity.slot_index()]))]);
    let working_state = WorkingState::from_touched_partitions_with_layout_and_sparse_slots(
        &selected,
        partitions.iter().copied(),
        runtime.config.storage.adjacency_policy.clone(),
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
        schema_authority: Arc::clone(&transaction.schema_authority),
    })
}
