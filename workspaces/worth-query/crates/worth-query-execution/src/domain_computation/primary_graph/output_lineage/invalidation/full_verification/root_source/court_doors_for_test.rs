//! Sealed test doors over this module's private members.
use super::*;

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
