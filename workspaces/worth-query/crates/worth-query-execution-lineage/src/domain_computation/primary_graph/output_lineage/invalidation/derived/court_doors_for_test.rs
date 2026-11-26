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

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn cell_for_read_for_test(
        &self,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<Option<CompanionBranchCell<BranchMarkRoot>>, CompanionPreflightStop> {
        Self::cell_for_read(self, selected)
    }
}

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn consumed_output_currentness_for_test(
        &self,
        selected: &PositionedRelationalSnapshot,
        identity: &RecordedSettlementIdentity,
    ) -> Result<ConsumedOutputCurrentness, CompanionPreflightStop> {
        Self::consumed_output_currentness(self, selected, identity)
    }
}

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn prepare_settlement_for_test(
        &self,
        registration: SettlementRegistration,
    ) -> Result<PreparedSettlementRegistration, SettlementRegistrationStop> {
        Self::prepare_settlement(self, registration)
    }
}

impl SourceInvalidationOwner {
    /// Public #358 courts: the installed image at `selected` takes a declared
    /// unavailable change at its commit, exactly as a delivery without exact
    /// touches applies it (`delivery::affected::apply` with no keys).
    #[doc(hidden)]
    pub fn apply_discontinuity_for_test(
        &self,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<(), SettlementRegistrationStop> {
        let cell = self
            .cell_for_read(selected)?
            .ok_or(SettlementRegistrationStop::Alignment(
                FullVerificationReason::MissingSettlement,
            ))?;
        let image = cell.read_image();
        let mut state = (*image.payload().current).clone();
        let commit = selected
            .commit_id()
            .ok_or(SettlementRegistrationStop::SourceUnavailable)?;
        super::super::delivery::affected::apply(
            &mut state,
            None,
            commit,
            &mut Default::default(),
        )?;
        let mut root = (**image.payload()).clone();
        root.current = Arc::new(state);
        let prepared = self.prepare_root_replacement(cell, image, Arc::new(root))?;
        drop(
            prepared
                .install()
                .map_err(|stopped| SettlementRegistrationStop::Edit(stopped.reason()))?,
        );
        Ok(())
    }
}
