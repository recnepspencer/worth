//! Counters for the selected preparation working state.
use crate::authority::commit::structural_summary::CommitStructuralSummary;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::storage::overlay::WorkingState;

pub(crate) fn record_preparation_counters(
    instrumentation: &RuntimeInstrumentation,
    working_state: &WorkingState,
    structural_summary: &CommitStructuralSummary,
) {
    let mut counters = instrumentation
        .complexity_counters
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    counters.commit_topology_flags = structural_summary.commit_topology.mask();
    counters.partitions_touched_by_commit = working_state.touched_partition_count();
    counters.bulk_entity_slots_reserved = structural_summary.bulk_entity_slots_reserved;
    counters.bulk_relation_slots_reserved = structural_summary.bulk_relation_slots_reserved;
}

pub(crate) fn record_mutation_counters(
    instrumentation: &RuntimeInstrumentation,
    working_state: &WorkingState,
) {
    let mut counters = instrumentation
        .complexity_counters
        .lock()
        .unwrap_or_else(|error| error.into_inner());
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

pub(crate) fn count_working_state_clone(
    instrumentation: &RuntimeInstrumentation,
    partitions: usize,
    entity_slots: usize,
    relation_slots: usize,
) {
    instrumentation.count(|counters| {
        counters.partitions_cloned += partitions;
        counters.entity_slots_cloned += entity_slots;
        counters.relation_slots_cloned += relation_slots;
    });
}

pub(crate) fn count_aosoa_prepare_chunks(
    instrumentation: &RuntimeInstrumentation,
    chunk_count: usize,
    slot_count: usize,
) {
    instrumentation.count(|counters| {
        counters.aosoa_entity_chunks_staged += chunk_count;
        counters.aosoa_entity_chunk_slots_materialized += slot_count;
    });
}
