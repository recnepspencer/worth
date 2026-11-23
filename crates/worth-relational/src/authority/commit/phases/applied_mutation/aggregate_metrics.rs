//! Completed Native maintenance acquisition reports, without Query relabeling.
use crate::aggregate_identity::AggregateReadObservations;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;

pub(crate) fn count_aggregate_maintenance_reads(
    instrumentation: &RuntimeInstrumentation,
    reads: AggregateReadObservations,
) {
    instrumentation.count(|counters| {
        counters.aggregate_maintenance_adjacency_lists = counters
            .aggregate_maintenance_adjacency_lists
            .saturating_add(reads.adjacency_lists);
        counters.aggregate_maintenance_entity_records = counters
            .aggregate_maintenance_entity_records
            .saturating_add(reads.entity_records);
        counters.aggregate_maintenance_relation_records = counters
            .aggregate_maintenance_relation_records
            .saturating_add(reads.relation_records);
        counters.aggregate_maintenance_aspect_states = counters
            .aggregate_maintenance_aspect_states
            .saturating_add(reads.aspect_states);
    });
}
