//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::derived::SettlementRegistrationStop;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::derived::SourceSettlementCurrentness;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity;

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
