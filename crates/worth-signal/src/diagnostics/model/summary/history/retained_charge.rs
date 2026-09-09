use super::{ExecutionHistoryNodeSummary, ExecutionHistorySummary};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for ExecutionHistoryNodeSummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            node: _,
            execution_record_id: _,
            semantic_segment_id: _,
            output_change: _,
            memoized_origin: _,
            reuse_basis,
            reuse_origin: _,
            persistent_correspondence_kind: _,
            composition_region_count: _,
            reuse_certification_proof_count: _,
            changed_partition_count: _,
            causality_kind,
        } = self;
        reuse_basis
            .retained_heap_charge(work)?
            .checked_add(causality_kind.retained_heap_charge(work)?)
    }
}

impl RetainedStorageMeasurement for ExecutionHistorySummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            profile: _,
            traced_node_count: _,
            execution_record_count: _,
            latest_execution_record_id: _,
            reuse_origin_counts,
            nodes,
        } = self;
        reuse_origin_counts
            .retained_heap_charge(work)?
            .checked_add(nodes.retained_heap_charge(work)?)
    }
}

#[cfg(test)]
#[path = "retained_charge_tests.rs"]
mod tests;
