use super::GraphSummary;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for GraphSummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            profile: _,
            active_node_count: _,
            arena_capacity: _,
            tombstone_count: _,
            clean_node_count: _,
            maybe_stale_node_count: _,
            dirty_node_count: _,
            dependency_edge_count: _,
            subscriber_edge_count: _,
            nodes_with_partition_scopes: _,
            nodes_with_trace_summary: _,
            nodes_with_execution_record: _,
            nodes_with_causality: _,
            partition_interner_size: _,
            sample_dirty_nodes,
            sample_nodes_with_execution_record,
            metrics,
        } = self;
        sample_dirty_nodes
            .retained_heap_charge(work)?
            .checked_add(sample_nodes_with_execution_record.retained_heap_charge(work)?)?
            .checked_add(metrics.retained_heap_charge(work)?)
    }
}
