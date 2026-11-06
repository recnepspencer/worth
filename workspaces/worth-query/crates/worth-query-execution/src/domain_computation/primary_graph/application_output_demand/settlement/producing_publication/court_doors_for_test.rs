//! Sealed test doors over this module's private members.
use super::*;

impl OriginalProducingPublication {
    #[doc(hidden)]
    pub fn changes_for_test(&self) -> &WorthQueryApplicationCommittedChanges {
        &self.changes
    }
}

impl OriginalProducingPublication {
    #[doc(hidden)]
    pub fn correspondence_for_test(&self) -> &Arc<WorthQueryApplicationOutputCorrespondence> {
        &self.correspondence
    }
}
