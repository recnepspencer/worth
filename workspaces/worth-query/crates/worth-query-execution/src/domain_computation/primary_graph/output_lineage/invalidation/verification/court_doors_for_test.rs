//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity;

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn clear_verified_dirty_for_test(
        &self,
        verified: VerifiedDirtySettlement,
    ) -> Result<(), SettlementVerificationStop> {
        Self::clear_verified_dirty(self, verified)
    }
}

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn reverify_dirty_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        identity: &Arc<RecordedSettlementIdentity>,
    ) -> Result<DirtyReverification, SettlementVerificationStop> {
        Self::reverify_dirty(self, execution, runtime, snapshot, selected, identity)
    }
}
