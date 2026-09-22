use crate::authority::commit::structural_summary::CommitStructuralSummary;
use crate::branch::SelectedRelationalBranchState;
use crate::runtime::RelationalPreparationRuntime;
use crate::storage::overlay::summarize_entity_chunk_plan;
use crate::storage::overlay::EntityWorkingSetLayout;
use crate::storage::overlay::PartitionAccess;
use crate::storage::overlay::PartitionCloneMode;
use crate::storage::overlay::WorkingState;
use crate::transactions::data::CommitPhaseTiming;
use crate::transactions::data::{
    CommitTopology, CreateIntent, EntityMutationIntent, MergedCommitPlan, MutationIntent,
    TransactionCommitError,
};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

pub(crate) struct PreparedWorkingStateScope {
    pub(crate) selected_branch_state: SelectedRelationalBranchState,
    pub(crate) merged_plan: MergedCommitPlan,
    pub(crate) structural_summary: CommitStructuralSummary,
    pub(crate) working_state: WorkingState,
    pub(crate) phase_timing: CommitPhaseTiming,
    pub(crate) footprint: crate::mvcc::RelationalTransactionFootprint,
    pub(crate) schema_authority: std::sync::Arc<crate::branch::RelationalBranchRootSchemaAuthority>,
}

const AOSOA_ENTITY_CHUNK_WIDTH_SMALL: usize = 128;
const AOSOA_ENTITY_CHUNK_WIDTH_MEDIUM: usize = 256;
const AOSOA_ENTITY_CHUNK_WIDTH_LARGE: usize = 512;
const AOSOA_SPARSE_ENTITY_SLOT_LIMIT: usize = 1024;
const AOSOA_SPARSE_PARTITION_LIMIT: usize = 8;

fn select_entity_working_set_layout(
    clone_mode: PartitionCloneMode,
    cloned_entity_slots: usize,
) -> EntityWorkingSetLayout {
    if !matches!(
        clone_mode,
        PartitionCloneMode::EntityOnly | PartitionCloneMode::GraphSparseEntities
    ) {
        return EntityWorkingSetLayout::CanonicalSoA;
    }

    let chunk_width = if cloned_entity_slots <= 4_096 {
        AOSOA_ENTITY_CHUNK_WIDTH_SMALL
    } else if cloned_entity_slots <= 16_384 {
        AOSOA_ENTITY_CHUNK_WIDTH_MEDIUM
    } else {
        AOSOA_ENTITY_CHUNK_WIDTH_LARGE
    };

    EntityWorkingSetLayout::AoSoACandidate { chunk_width }
}

fn sparse_entity_slots_for_plan(
    clone_mode: PartitionCloneMode,
    merged_plan: &MergedCommitPlan,
    footprint: Option<&crate::mvcc::RelationalTransactionFootprint>,
) -> Option<BTreeMap<crate::identity::data::PartitionId, BTreeSet<usize>>> {
    if !matches!(
        clone_mode,
        PartitionCloneMode::EntityOnly | PartitionCloneMode::GraphSparseEntities
    ) {
        return None;
    }

    let mut slots_by_partition = BTreeMap::new();
    for intent in &merged_plan.merged_intents {
        match intent {
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(spec)) => {
                slots_by_partition
                    .entry(spec.entity_id.partition_id)
                    .or_insert_with(BTreeSet::new)
                    .insert(spec.entity_id.slot_index());
            }
            // A revalidation demand writes no field, but it marks the
            // record's slot touched, and a touched slot must be materialized
            // in the working state. Naming its slot here keeps the sparse
            // clone both correct and narrow: without this arm the plan falls
            // back to cloning every slot in the partition to cover one record
            // the demand never changed.
            MutationIntent::Entity(EntityMutationIntent::Revalidate(spec)) => {
                slots_by_partition
                    .entry(spec.entity_id.partition_id)
                    .or_insert_with(BTreeSet::new)
                    .insert(spec.entity_id.slot_index());
            }
            MutationIntent::Create(_)
                if matches!(clone_mode, PartitionCloneMode::GraphSparseEntities) => {}
            _ => return None,
        }
    }

    if let Some(footprint) = footprint {
        for read in footprint.reads() {
            if let crate::mvcc::RelationalTransactionReadLocus::Existing(
                crate::transactions::data::RecordRef::Entity(entity),
            ) = read
            {
                slots_by_partition
                    .entry(entity.partition_id)
                    .or_insert_with(BTreeSet::new)
                    .insert(entity.slot_index());
            }
        }
    }

    let total_slots: usize = slots_by_partition.values().map(BTreeSet::len).sum();
    (total_slots > 0
        && total_slots <= AOSOA_SPARSE_ENTITY_SLOT_LIMIT
        && slots_by_partition.len() <= AOSOA_SPARSE_PARTITION_LIMIT)
        .then_some(slots_by_partition)
}

fn sparse_relation_overlay_partitions_for_plan(
    clone_mode: PartitionCloneMode,
    merged_plan: &MergedCommitPlan,
    footprint: Option<&crate::mvcc::RelationalTransactionFootprint>,
) -> Option<BTreeSet<crate::identity::data::PartitionId>> {
    if !matches!(clone_mode, PartitionCloneMode::GraphSparseEntities) {
        return None;
    }

    let mut partitions = BTreeSet::new();
    let mut saw_relation_create = false;
    for intent in &merged_plan.merged_intents {
        match intent {
            MutationIntent::Create(CreateIntent::Relation(spec)) => {
                saw_relation_create = true;
                partitions.insert(spec.partition_id);
            }
            MutationIntent::Create(CreateIntent::BulkRelations(spec)) => {
                saw_relation_create = true;
                partitions.insert(spec.partition_id);
            }
            MutationIntent::Create(CreateIntent::Entity(_))
            | MutationIntent::Create(CreateIntent::BulkEntities(_))
            | MutationIntent::Entity(EntityMutationIntent::UpdateFields(_))
            // A demand touches one entity record and no adjacency, so it adds
            // no relation overlay partition and must not veto the sparse path.
            | MutationIntent::Entity(EntityMutationIntent::Revalidate(_)) => {}
            _ => return None,
        }
    }

    if let Some(footprint) = footprint {
        for read in footprint.reads() {
            if let crate::mvcc::RelationalTransactionReadLocus::Existing(
                crate::transactions::data::RecordRef::Relation(relation),
            ) = read
            {
                partitions.insert(relation.partition_id);
            }
        }
    }

    saw_relation_create.then_some(partitions)
}

pub(crate) fn prepare_working_state_scope(
    runtime: &RelationalPreparationRuntime,
    transaction: &mut crate::mvcc::BranchBoundRelationalTransaction,
) -> Result<PreparedWorkingStateScope, TransactionCommitError> {
    transaction
        .ensure_current_basis(runtime)
        .map_err(TransactionCommitError::conflict)?;
    let selected_branch_state =
        SelectedRelationalBranchState::from_admitted_basis(&transaction.basis);
    runtime
        .services
        .symbols
        .with_read(|symbols| {
            transaction.validate_staged_branch_locality(selected_branch_state.state(), symbols)
        })
        .map_err(TransactionCommitError::conflict)?;
    let mut phase_timing = CommitPhaseTiming::default();
    let normalization_started = Instant::now();
    let intents = transaction.normalized_intents_for_merge(runtime);
    phase_timing.draft_intent_normalization_micros =
        normalization_started.elapsed().as_micros() as u64;
    let merge_plan_started = Instant::now();
    let (merged_plan, merged_plan_timing) = transaction
        .build_merged_plan_for_state_with_timing(runtime, selected_branch_state.state(), intents)
        .map_err(TransactionCommitError::conflict)?;
    phase_timing.draft_merge_plan_micros = merge_plan_started.elapsed().as_micros() as u64;
    phase_timing.draft_intent_validation_micros = merged_plan_timing.validation_micros;
    phase_timing.draft_intent_sort_micros = merged_plan_timing.sort_micros;
    phase_timing.draft_conflict_detection_micros = merged_plan_timing.conflict_detection_micros;
    transaction
        .footprint
        .derive_validation_dependencies(&merged_plan, transaction.maximum_footprint_loci)
        .map_err(|denial| TransactionCommitError::conflict(denial.into_conflict()))?;
    let (structural_summary, working_state, prepare_phase_timing) =
        prepare_authoritative_working_state_scope_for_base(
            runtime,
            selected_branch_state.state(),
            &merged_plan,
            transaction.merge_parent_bases.len(),
            Some(&transaction.footprint),
        );
    phase_timing.draft_structural_summary_micros =
        prepare_phase_timing.draft_structural_summary_micros;
    phase_timing.draft_working_state_clone_micros =
        prepare_phase_timing.draft_working_state_clone_micros;

    Ok(PreparedWorkingStateScope {
        selected_branch_state,
        merged_plan,
        structural_summary,
        working_state,
        phase_timing,
        footprint: transaction.footprint.clone(),
        schema_authority: std::sync::Arc::clone(&transaction.schema_authority),
    })
}

pub(crate) fn prepare_lowered_working_state_scope(
    runtime: &RelationalPreparationRuntime,
    transaction: &crate::mvcc::BranchBoundRelationalTransaction,
    selected_branch_state: SelectedRelationalBranchState,
    merged_plan: MergedCommitPlan,
) -> PreparedWorkingStateScope {
    let (structural_summary, working_state, phase_timing) =
        prepare_authoritative_working_state_scope_for_base(
            runtime,
            selected_branch_state.state(),
            &merged_plan,
            transaction.merge_parent_bases.len(),
            Some(&transaction.footprint),
        );

    PreparedWorkingStateScope {
        selected_branch_state,
        merged_plan,
        structural_summary,
        working_state,
        phase_timing,
        footprint: transaction.footprint.clone(),
        schema_authority: std::sync::Arc::clone(&transaction.schema_authority),
    }
}

pub(crate) fn prepare_authoritative_working_state_scope_for_base(
    runtime: &RelationalPreparationRuntime,
    base_state: &impl PartitionAccess,
    merged_plan: &MergedCommitPlan,
    merge_parent_count: usize,
    footprint: Option<&crate::mvcc::RelationalTransactionFootprint>,
) -> (CommitStructuralSummary, WorkingState, CommitPhaseTiming) {
    let mut phase_timing = CommitPhaseTiming::default();
    let summary_started = Instant::now();
    let mut structural_summary =
        CommitStructuralSummary::derive(base_state, base_state, merged_plan, merge_parent_count);
    if let Some(footprint) = footprint {
        structural_summary
            .touched_partitions
            .extend(footprint.validation_partitions());
    }
    phase_timing.draft_structural_summary_micros = summary_started.elapsed().as_micros() as u64;
    let clone_mode = match structural_summary.commit_topology {
        CommitTopology::FlatEntityBatch => PartitionCloneMode::EntityOnly,
        CommitTopology::GraphMutation => PartitionCloneMode::GraphSparseEntities,
        CommitTopology::BranchMerge => PartitionCloneMode::Full,
    };
    let cloned_partition_count = structural_summary.touched_partitions.len();
    let cloned_entity_slots = structural_summary
        .touched_partitions
        .iter()
        .map(|partition_id| {
            base_state
                .get_partition(*partition_id)
                .map(|partition| partition.entity_arena.slot_count())
                .unwrap_or(0)
        })
        .sum();
    let cloned_relation_slots = if matches!(clone_mode, PartitionCloneMode::Full) {
        structural_summary
            .touched_partitions
            .iter()
            .map(|partition_id| {
                base_state
                    .get_partition(*partition_id)
                    .map(|partition| partition.relation_arena.slot_count())
                    .unwrap_or(0)
            })
            .sum()
    } else {
        0
    };
    let sparse_entity_slots = sparse_entity_slots_for_plan(clone_mode, merged_plan, footprint);
    let sparse_relation_overlay_partitions =
        sparse_relation_overlay_partitions_for_plan(clone_mode, merged_plan, footprint);
    let clone_mode = if matches!(clone_mode, PartitionCloneMode::GraphSparseEntities)
        && sparse_entity_slots.is_none()
        && sparse_relation_overlay_partitions.is_none()
    {
        PartitionCloneMode::Full
    } else {
        clone_mode
    };
    let sparse_entity_slot_count = sparse_entity_slots
        .as_ref()
        .map(|slots| slots.values().map(BTreeSet::len).sum::<usize>());
    let entity_working_set_layout = select_entity_working_set_layout(
        clone_mode,
        sparse_entity_slot_count.unwrap_or(cloned_entity_slots),
    );
    let candidate_chunk_plan = summarize_entity_chunk_plan(
        sparse_entity_slot_count.unwrap_or(cloned_entity_slots),
        entity_working_set_layout,
    );
    let clone_started = Instant::now();
    let working_state = WorkingState::from_touched_partitions_with_layout_and_sparse_slots(
        base_state,
        structural_summary.touched_partitions.iter().copied(),
        runtime.config.storage.adjacency_policy.clone(),
        clone_mode,
        entity_working_set_layout,
        sparse_entity_slots.as_ref(),
        sparse_relation_overlay_partitions.as_ref(),
    );
    phase_timing.draft_working_state_clone_micros = clone_started.elapsed().as_micros() as u64;
    if matches!(clone_mode, PartitionCloneMode::Full) {
        runtime.performance_access().count_working_state_clone(
            cloned_partition_count,
            cloned_entity_slots,
            cloned_relation_slots,
        );
    }
    match working_state.entity_working_set_layout() {
        EntityWorkingSetLayout::CanonicalSoA => {}
        EntityWorkingSetLayout::AoSoACandidate { .. } => {
            runtime.performance_access().count_aosoa_prepare_chunks(
                candidate_chunk_plan.chunk_count,
                candidate_chunk_plan.slot_count,
            )
        }
    }

    (structural_summary, working_state, phase_timing)
}

pub(crate) fn record_preparation_counters(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    working_state: &WorkingState,
    structural_summary: &CommitStructuralSummary,
) {
    let mut counters = runtime
        .services
        .instrumentation
        .complexity_counters
        .lock()
        .expect("complexity counter lock poisoned");
    counters.commit_topology_flags = structural_summary.commit_topology.mask();
    counters.partitions_touched_by_commit = working_state.touched_partition_count();
    counters.bulk_entity_slots_reserved = structural_summary.bulk_entity_slots_reserved;
    counters.bulk_relation_slots_reserved = structural_summary.bulk_relation_slots_reserved;
}

pub(crate) fn record_mutation_counters(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    working_state: &WorkingState,
) {
    let mut counters = runtime
        .services
        .instrumentation
        .complexity_counters
        .lock()
        .expect("complexity counter lock poisoned");
    counters.entity_slots_touched_by_commit = working_state
        .mutation_journal()
        .values()
        .map(|journal| journal.entity_slots.len())
        .sum();
    counters.relation_slots_touched_by_commit = working_state
        .mutation_journal()
        .values()
        .map(|journal| journal.relation_slots.len())
        .sum();
}

#[cfg(test)]
#[path = "prepare_tests.rs"]
mod tests;
