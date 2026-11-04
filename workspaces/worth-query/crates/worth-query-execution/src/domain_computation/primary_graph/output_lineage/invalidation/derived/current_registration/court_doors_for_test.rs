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

impl<'selected> PreparedCurrentSettlementRegistration<'selected> {
    #[doc(hidden)]
    pub fn install_for_test(
        self,
    ) -> Result<CurrentSettlementRegistrationCleanup, StoppedCurrentSettlementRegistration<'selected>>
    {
        Self::install(self)
    }
}

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn prepare_current_settlement_for_test<'selected>(
        &self,
        registration: SettlementRegistration,
        selected: &'selected PositionedRelationalSnapshot,
    ) -> Result<PreparedCurrentSettlementRegistration<'selected>, SettlementRegistrationStop> {
        Self::prepare_current_settlement(self, registration, selected)
    }
}

impl<'selected> PreparedCurrentSettlementRegistration<'selected> {
    #[doc(hidden)]
    pub fn identity_for_test(&self) -> &Arc<RecordedSettlementIdentity> {
        Self::identity(self)
    }
}

impl<'selected> PreparedCurrentSettlementRegistration<'selected> {
    #[doc(hidden)]
    pub fn selected_source_for_test(&self) -> &'selected PositionedRelationalSnapshot {
        Self::selected_source(self)
    }
}
