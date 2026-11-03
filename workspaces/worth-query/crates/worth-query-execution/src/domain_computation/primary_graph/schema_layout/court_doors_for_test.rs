//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn provider_dispatch_outbox_for_test(&self) -> &WorthQueryDispatchOutboxLayout {
        Self::provider_dispatch_outbox(self)
    }
}
