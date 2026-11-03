//! Sealed test doors over this module's private members.
use super::*;

impl RecordedOutput {
    #[doc(hidden)]
    pub fn settlement_identity_for_test(&self) -> &Arc<RecordedSettlementIdentity> {
        &self.settlement_identity
    }
}
