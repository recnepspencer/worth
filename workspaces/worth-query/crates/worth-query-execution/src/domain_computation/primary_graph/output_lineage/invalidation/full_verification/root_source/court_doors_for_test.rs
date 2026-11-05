//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity;

impl FullVerificationImage<'_> {
    #[doc(hidden)]
    pub fn require_complete_root_upstream_for_test(
        &self,
        source: EvidenceView<'_>,
        terminal: &Arc<RecordedSettlementIdentity>,
    ) -> Result<(), FullVerificationStop> {
        Self::require_complete_root_upstream(self, source, terminal)
    }
}
