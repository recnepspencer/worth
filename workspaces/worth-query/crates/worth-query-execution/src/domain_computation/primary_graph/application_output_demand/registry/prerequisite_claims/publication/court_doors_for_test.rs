//! Sealed test doors over this module's private members.
use super::*;

impl PreparedPrerequisiteClaims {
    #[doc(hidden)]
    pub fn publish_for_test(
        self, identity: Arc<RecordedSettlementIdentity>) -> super::super::super::SupersededSettlements {
        Self::publish(self, identity)
    }
}

impl PreparedPrerequisiteClaims {
    #[doc(hidden)]
    pub fn reserve_identity_for_test(
        &mut self, identity: &Arc<RecordedSettlementIdentity>) -> Result<(), WorthQueryOutputDemandDenial> {
        Self::reserve_identity(self, identity)
    }
}
