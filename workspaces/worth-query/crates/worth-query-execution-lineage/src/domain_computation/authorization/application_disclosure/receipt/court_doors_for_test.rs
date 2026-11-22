//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationDisclosureReceipt {
    #[doc(hidden)]
    pub fn governed_for_test(
        classification: impl Into<String>,
        disclosed: Vec<(ApplicationQueryResultSlotKey, AspectValue)>,
        omitted: Vec<(ApplicationQueryResultSlotKey, AspectValue)>,
        capability_authority_identity: impl Into<String>,
        decision_identity: [u8; 32],
        authorization_decision_fact_count: usize,
    ) -> Self {
        Self::governed(
            classification,
            disclosed,
            omitted,
            capability_authority_identity,
            decision_identity,
            authorization_decision_fact_count,
        )
    }
}
