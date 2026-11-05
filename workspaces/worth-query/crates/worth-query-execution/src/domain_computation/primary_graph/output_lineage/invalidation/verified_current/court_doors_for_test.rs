//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::derived::SettlementRegistrationStop;

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn prepare_verified_current_for_test(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        identity: &Arc<RecordedSettlementIdentity>,
        expected_facts: &super::super::super::ComparableSourceFacts,
    ) -> Result<Option<PreparedVerifiedCurrent>, SettlementRegistrationStop> {
        Self::prepare_verified_current(self, runtime, snapshot, selected, identity, expected_facts)
    }
}

impl PreparedVerifiedCurrent {
    #[doc(hidden)]
    pub fn install_for_test(self) -> Result<VerifiedCurrentCleanup, StoppedVerifiedCurrent> {
        Self::install(self)
    }
}

impl StoppedVerifiedCurrent {
    #[doc(hidden)]
    pub fn reason_for_test(&self) -> CompanionCellEditStop {
        Self::reason(self)
    }
}
