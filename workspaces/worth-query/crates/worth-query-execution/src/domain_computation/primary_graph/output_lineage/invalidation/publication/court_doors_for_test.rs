//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::mark_state::FullVerificationReason;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn require_settlement_verification_for_test(
        &mut self,
        identity: &RecordedSettlementIdentity,
        reason: FullVerificationReason,
    ) {
        Self::require_settlement_verification(self, identity, reason)
    }
}
