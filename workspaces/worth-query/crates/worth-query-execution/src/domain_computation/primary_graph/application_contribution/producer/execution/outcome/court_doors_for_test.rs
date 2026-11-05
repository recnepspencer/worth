//! Sealed test doors over this module's private members.
use super::*;

impl ProducerCommitReceipt {
    #[doc(hidden)]
    pub fn record_publication_for_test(self, record: impl FnOnce()) -> Self {
        Self::record_publication(self, record)
    }
}
