//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryDispatchOutboxRecord {
    #[doc(hidden)]
    pub fn from_installed_contract_for_test(
        correlation: ExternalEffectCorrelationIdentity,
        contract: &InstalledExternalEffectContract,
        payload: Vec<u8>,
        outcome_identity: u64,
    ) -> Option<Self> {
        Self::from_installed_contract(correlation, contract, payload, outcome_identity)
    }
}

impl WorthQueryPendingDispatchOutbox {
    #[doc(hidden)]
    pub fn created_entity_for_test(&self) -> &CreatedEntityRef {
        Self::created_entity(self)
    }
}

impl WorthQueryPendingDispatchOutbox {
    #[doc(hidden)]
    pub fn record_for_test(&self) -> &WorthQueryDispatchOutboxRecord {
        Self::record(self)
    }
}
