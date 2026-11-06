//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryDispatchOutboxRecord;
use crate::domain_computation::vocabulary::primary_graph::outbox_commit::WorthQueryCommittedDispatchOutboxBinding;

impl WorthQueryCommittedDispatchOutboxBinding {
    #[doc(hidden)]
    pub fn record_for_test(&self) -> &WorthQueryDispatchOutboxRecord {
        Self::record(self)
    }
}

impl WorthQueryCommittedDispatchOutboxBinding {
    #[doc(hidden)]
    pub fn record_ref_for_test(&self) -> &RecordRef {
        Self::record_ref(self)
    }
}
