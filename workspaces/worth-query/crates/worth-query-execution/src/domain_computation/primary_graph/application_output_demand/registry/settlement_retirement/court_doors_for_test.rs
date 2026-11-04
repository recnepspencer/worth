//! Sealed test doors over this module's private members.
use super::*;

impl SupersededSettlements {
    #[doc(hidden)]
    pub fn new_for_test(
        registry: WorthQueryOutputDemandRegistry,
        key: Arc<WorthQueryOutputDemandKey>,
    ) -> Self {
        Self::new(registry, key)
    }
}

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn posts_settlement_for_test(
        &self,
        identity: &RecordedSettlementIdentity,
    ) -> Result<bool, crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial> {
        Self::posts_settlement(self, identity)
    }
}
