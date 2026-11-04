//! Sealed test doors over this module's private members.
use super::*;

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn prepare_current_stable_settlement_for_test<'selected>(
        &self,
        registration: SettlementRegistration,
        selected: &'selected PositionedRelationalSnapshot,
        equality: StableEqualityConsequence<'selected>,
    ) -> Result<PreparedCurrentSettlementRegistration<'selected>, SettlementRegistrationStop> {
        Self::prepare_current_stable_settlement(self, registration, selected, equality)
    }
}
