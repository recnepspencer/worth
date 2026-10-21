//! The exact accepted consumer whose registry custody a refresh retains.
use super::AcceptedCurrentCandidate;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;

impl AcceptedCurrentCandidate {
    pub(in crate::domain_computation::primary_graph) fn consumes_outputs(&self) -> bool {
        !self.selected.consumed_outputs().is_empty()
    }

    pub(in crate::domain_computation::primary_graph) fn retain_recorded_identity(
        &self,
    ) -> Result<
        std::sync::Arc<RecordedSettlementIdentity>,
        worth_relational::facade::mvcc::CompanionPreflightStop,
    > {
        Ok(std::sync::Arc::clone(self.selected.settlement_identity()))
    }
}
