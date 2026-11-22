//! Sealed test doors over this module's private members.
use super::*;

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
