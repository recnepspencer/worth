//! Original-request backing for the historical roster retained across repair.
use crate::domain_computation::primary_graph::{
    application_checkpoint::{CheckpointDecodeDenial, CheckpointMetadataFunding},
    application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity as Identity,
};

pub(super) struct PreservedOutputs {
    rows: Vec<Identity>,
    // Last to drop: shallow identity clones retain their original immutable
    // heaps/payers; this ticket pays the independent outer roster allocation.
    _funding: CheckpointMetadataFunding,
}
impl PreservedOutputs {
    pub(super) fn retain(
        rows: &[Identity],
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, CheckpointDecodeDenial> {
        let mut funding = CheckpointMetadataFunding::new(execution)?;
        let mut retained = funding.vec::<Identity>(rows.len(), execution)?;
        for row in rows {
            funding.check_live(execution)?;
            retained.push(row.clone());
        }
        Ok(Self {
            rows: retained,
            _funding: funding,
        })
    }
    pub(super) fn rows(&self) -> &[Identity] {
        &self.rows
    }
}
