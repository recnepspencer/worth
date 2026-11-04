//! Sealed test doors over this module's private members.
use super::*;

impl PublishedStableLineage {
    #[doc(hidden)]
    pub fn exact_settlement_for_test(
        &self,
    ) -> &Arc<super::super::super::super::RecordedSettlementIdentity> {
        Self::exact_settlement(self)
    }
}
