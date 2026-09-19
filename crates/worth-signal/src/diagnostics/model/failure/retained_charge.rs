use super::{FailureSummary, RollbackDiagnostic};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for FailureSummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            profile: _,
            phase: _,
            stage_index: _,
            node: _,
            executor: _,
            execution_record_id: _,
            has_plan_summary: _,
            rolled_back: _,
            staged_node_patch_count: _,
            max_touched_nodes_in_txn: _,
            event_epochs,
            message,
        } = self;
        event_epochs
            .retained_heap_charge(work)?
            .checked_add(message.retained_heap_charge(work)?)
    }
}
impl RetainedStorageMeasurement for RollbackDiagnostic {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            rolled_back: _,
            staged_node_patch_count: _,
            max_touched_nodes_in_txn: _,
            reason,
            event_epochs,
        } = self;
        reason
            .retained_heap_charge(work)?
            .checked_add(event_epochs.retained_heap_charge(work)?)
    }
}
