use worth_relational_contracts::internal_port::execution::monotonic_time::{elapsed_since, monotonic_now};
use contracts::mvcc::transaction::staging::RelationalTransactionStagingDenialOwnerPort as _;
use mutation::mvcc::transaction::footprint::validation_dependencies::RelationalTransactionFootprintOwnerPort as _;
use mutation::mvcc::transaction::footprint::RelationalTransactionFootprintOwnerPort as _;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port as mutation;
// Diagnostic counters keep their valid numeric prefix on unwind; they grant no authority.
use crate::authority::commit::semantic_plan::PreparedRelationalCommitPlan;
use crate::authority::commit::structural_summary::CommitStructuralSummary;
use crate::mvcc::transaction::staged_plan::StagedPlanValidation;
use crate::transactions::data::outcomes::commit_result::CommitPhaseTiming;
use contracts::transactions::data::intents::mutation_intent::CreateIntent;
use contracts::transactions::data::intents::mutation_intent::EntityMutationIntent;
use contracts::transactions::data::intents::mutation_intent::MutationIntent;
use contracts::transactions::data::mutation_planning::CommitTopology;
use contracts::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use std::collections::{BTreeMap, BTreeSet};
use working_state_inputs::{WorkingStateDraft, WorkingStateDraftRuntime, WorkingStateScope};
use worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState;
use worth_relational_branch::internal_port::branch::selected_read::SelectedRelationalBranchRead;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;
use worth_relational_storage::internal_port::storage::overlay::partition::{
    summarize_entity_chunk_plan, EntityWorkingSetLayout, PartitionCloneMode,
};
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;
#[path = "prepare/retained_entity_selection.rs"]
pub mod retained_entity_selection;
pub mod working_state_inputs;
pub mod working_state_metrics;

pub struct PreparedWorkingStateScope {
    pub selected_branch_state: SelectedRelationalBranchState,
    pub plan: PreparedRelationalCommitPlan,
    pub structural_summary: CommitStructuralSummary,
    pub working_state: WorkingState,
    pub(crate) phase_timing: CommitPhaseTiming,
    pub footprint: mutation::mvcc::transaction::footprint::RelationalTransactionFootprint,
    pub schema_authority:
        std::sync::Arc<worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority>,
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

// Some(empty) admits a supported plan needing no existing entity payload.
// None declines sparse materialization; it must never authorize a shape-only clone.
fn sparse_entity_slots_for_plan(
    clone_mode: PartitionCloneMode,
    merged_plan: &MergedCommitPlan,
    footprint: Option<&mutation::mvcc::transaction::footprint::RelationalTransactionFootprint>,
) -> Option<BTreeMap<contracts::identity::data::primitives::PartitionId, BTreeSet<usize>>> {
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
            if let mutation::mvcc::transaction::footprint::RelationalTransactionReadLocus::Existing(
                contracts::transactions::data::primitives::RecordRef::Entity(entity),
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
    (total_slots <= AOSOA_SPARSE_ENTITY_SLOT_LIMIT
        && slots_by_partition.len() <= AOSOA_SPARSE_PARTITION_LIMIT)
        .then_some(slots_by_partition)
}

fn sparse_relation_overlay_partitions_for_plan(
    clone_mode: PartitionCloneMode,
    merged_plan: &MergedCommitPlan,
    footprint: Option<&mutation::mvcc::transaction::footprint::RelationalTransactionFootprint>,
) -> Option<BTreeSet<contracts::identity::data::primitives::PartitionId>> {
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
            if let mutation::mvcc::transaction::footprint::RelationalTransactionReadLocus::Existing(
                contracts::transactions::data::primitives::RecordRef::Relation(relation),
            ) = read
            {
                partitions.insert(relation.partition_id);
            }
        }
    }

    saw_relation_create.then_some(partitions)
}

pub fn prepare_working_state_scope<'a, Runtime, Transaction>(
    runtime: &'a Runtime,
    transaction: &mut Transaction,
    selected: SelectedRelationalBranchRead<'_>,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<PreparedWorkingStateScope, TransactionCommitError>
where
    &'a Runtime: Into<WorkingStateDraftRuntime<'a>>,
    for<'t> &'t mut Transaction: Into<WorkingStateDraft<'t>>,
{
    let inputs = runtime.into();
    let draft = transaction.into();
    let normalization_started = monotonic_now();
    let intents = inputs.normalization.normalized(
        draft.transaction,
        draft.overlay,
        draft.footprint,
        request,
    )?;
    mutation::aggregate_identity::guard::current_relation_creations(
        request,
        inputs.commit_work,
        draft.basis_schema,
        &intents.plan().merged_intents,
    )?;
    let normalization_micros = elapsed_since(normalization_started).as_micros() as u64;
    let merge_plan_started = monotonic_now();
    let (merged_plan, merged_plan_timing) = StagedPlanValidation {
        schema: draft.mutation_schema.registry(),
        cross_context: inputs.cross_context,
        instrumentation: inputs.working_state.instrumentation,
        coupled_commands: draft.coupled_commands,
    }
    .validate(&selected, intents, request)?;
    let merge_plan_micros = elapsed_since(merge_plan_started).as_micros() as u64;
    draft
        .footprint
        .derive_validation_dependencies(merged_plan.plan(), request.allocation_policy())
        .map_err(|denial| TransactionCommitError::conflict(denial.into_conflict()))?;
    let mut prepared = prepare_lowered_with_scope(
        inputs.working_state,
        WorkingStateScope {
            footprint: draft.footprint,
            schema: draft.mutation_schema,
        },
        selected,
        merged_plan,
    );
    let timing = &mut prepared.phase_timing;
    timing.draft_intent_normalization_micros = normalization_micros;
    timing.draft_merge_plan_micros = merge_plan_micros;
    timing.draft_intent_validation_micros = merged_plan_timing.validation_micros;
    timing.draft_intent_sort_micros = merged_plan_timing.sort_micros;
    timing.draft_conflict_detection_micros = merged_plan_timing.conflict_detection_micros;
    Ok(prepared)
}

pub fn prepare_lowered_working_state_scope<'a, 't, Transaction>(
    inputs: impl Into<working_state_inputs::WorkingStatePreparation<'a>>,
    transaction: &'t Transaction,
    selected: SelectedRelationalBranchRead<'_>,
    merged_plan: mutation::mvcc::transaction::normalized_plan::OwnedNormalizedPlan,
) -> PreparedWorkingStateScope
where
    &'t Transaction: Into<WorkingStateScope<'t>>,
{
    prepare_lowered_with_scope(inputs.into(), transaction.into(), selected, merged_plan)
}

fn prepare_lowered_with_scope(
    inputs: working_state_inputs::WorkingStatePreparation<'_>,
    transaction: WorkingStateScope<'_>,
    selected: SelectedRelationalBranchRead<'_>,
    merged_plan: mutation::mvcc::transaction::normalized_plan::OwnedNormalizedPlan,
) -> PreparedWorkingStateScope {
    let (structural_summary, working_state, phase_timing) =
        prepare_authoritative_working_state_scope_for_base(
            inputs,
            &selected,
            merged_plan.plan(),
            Some(transaction.footprint),
        );

    PreparedWorkingStateScope {
        selected_branch_state: selected.selected().clone(),
        plan: PreparedRelationalCommitPlan::Mutation(merged_plan),
        structural_summary,
        working_state,
        phase_timing,
        footprint: transaction.footprint.clone(),
        schema_authority: std::sync::Arc::clone(transaction.schema),
    }
}

pub fn prepare_authoritative_working_state_scope_for_base<'a>(
    inputs: impl Into<working_state_inputs::WorkingStatePreparation<'a>>,
    base_state: &impl CurrentPartitionAccess,
    merged_plan: &MergedCommitPlan,
    footprint: Option<&mutation::mvcc::transaction::footprint::RelationalTransactionFootprint>,
) -> (CommitStructuralSummary, WorkingState, CommitPhaseTiming) {
    let inputs = inputs.into();
    let mut phase_timing = CommitPhaseTiming::default();
    let summary_started = monotonic_now();
    let mut structural_summary = CommitStructuralSummary::derive(base_state, merged_plan);
    if let Some(footprint) = footprint {
        structural_summary
            .touched_partitions
            .extend(footprint.validation_partitions());
    }
    phase_timing.draft_structural_summary_micros =
        elapsed_since(summary_started).as_micros() as u64;
    let clone_mode = match structural_summary.commit_topology {
        // The read demand cannot use an entity-only image: real relation and
        // endpoint preimages must reach judgment without claiming a graph write.
        CommitTopology::FlatEntityBatch if merged_plan.revalidates_relations() => {
            PartitionCloneMode::Full
        }
        CommitTopology::FlatEntityBatch => PartitionCloneMode::EntityOnly,
        CommitTopology::GraphMutation => PartitionCloneMode::GraphSparseEntities,
    };
    let cloned_partition_count = structural_summary.touched_partitions.len();
    let cloned_entity_slots = structural_summary
        .touched_partitions
        .iter()
        .map(|partition_id| {
            base_state
                .get_current_partition(*partition_id)
                .map(|partition| partition.entity_arena().slot_count())
                .unwrap_or(0)
        })
        .sum();
    let sparse_entity_slots = sparse_entity_slots_for_plan(clone_mode, merged_plan, footprint);
    let sparse_relation_overlay_partitions =
        sparse_relation_overlay_partitions_for_plan(clone_mode, merged_plan, footprint);
    // Sparse relations cannot supply an unavailable entity preimage. This is
    // the sole production admission into sparse working-state construction.
    let clone_mode = if matches!(clone_mode, PartitionCloneMode::GraphSparseEntities)
        && sparse_entity_slots.is_none()
    {
        PartitionCloneMode::Full
    } else {
        clone_mode
    };
    let cloned_relation_slots = if matches!(clone_mode, PartitionCloneMode::Full) {
        structural_summary
            .touched_partitions
            .iter()
            .map(|partition_id| {
                base_state
                    .get_current_partition(*partition_id)
                    .map(|partition| partition.relation_arena().slot_count())
                    .unwrap_or(0)
            })
            .sum()
    } else {
        0
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
    let clone_started = monotonic_now();
    let working_state = WorkingState::from_touched_partitions_with_layout_and_sparse_slots(
        base_state,
        structural_summary.touched_partitions.iter().copied(),
        inputs.adjacency_policy.clone(),
        clone_mode,
        entity_working_set_layout,
        sparse_entity_slots.as_ref(),
        sparse_relation_overlay_partitions.as_ref(),
    );
    phase_timing.draft_working_state_clone_micros = elapsed_since(clone_started).as_micros() as u64;
    if matches!(clone_mode, PartitionCloneMode::Full) {
        working_state_metrics::count_working_state_clone(
            inputs.instrumentation,
            cloned_partition_count,
            cloned_entity_slots,
            cloned_relation_slots,
        );
    }
    match working_state.entity_working_set_layout() {
        EntityWorkingSetLayout::CanonicalSoA => {}
        EntityWorkingSetLayout::AoSoACandidate { .. } => {
            working_state_metrics::count_aosoa_prepare_chunks(
                inputs.instrumentation,
                candidate_chunk_plan.chunk_count,
                candidate_chunk_plan.slot_count,
            )
        }
    }

    (structural_summary, working_state, phase_timing)
}

#[cfg(test)]
#[path = "prepare_tests.rs"]
pub mod tests;
