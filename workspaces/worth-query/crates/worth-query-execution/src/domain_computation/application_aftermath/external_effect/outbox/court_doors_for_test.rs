//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::vocabulary::application_aftermath::correlation::ExternalEffectCorrelationIdentity;
use crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryDispatchOutboxRecord;
use crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryPendingDispatchOutbox;



impl WorthQueryPendingDispatchOutbox {
    #[doc(hidden)]
    pub fn record_for_test(&self) -> &WorthQueryDispatchOutboxRecord {
        Self::record(self)
    }
}
