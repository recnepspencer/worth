//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn retain_consumer_handoff_for_test(
        &self,
        identity: &Arc<RecordedSettlementIdentity>,
    ) -> Result<ConsumerCustody, WorthQueryOutputDemandDenial> {
        Self::retain_consumer_handoff(self, identity)
    }
}
