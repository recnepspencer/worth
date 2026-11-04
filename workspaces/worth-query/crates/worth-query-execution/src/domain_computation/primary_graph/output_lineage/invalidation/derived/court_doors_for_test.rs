//! Sealed test doors over this module's private members.
use super::*;

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn currentness_for_test(
        &self,
        selected: &PositionedRelationalSnapshot,
        identity: &RecordedSettlementIdentity,
    ) -> Result<SourceSettlementCurrentness, CompanionPreflightStop> {
        Self::currentness(self, selected, identity)
    }
}

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn register_settlement_for_test(
        &self,
        registration: SettlementRegistration,
    ) -> Result<(), SettlementRegistrationStop> {
        Self::register_settlement(self, registration)
    }
}
