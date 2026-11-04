//! Sealed test doors over this module's private members.
use super::*;

impl SelectedRequiredPublicationHead {
    #[doc(hidden)]
    pub fn cold_predecessor_for_test(&self) -> Result<RequiredPublicationPredecessor, Stop> {
        Self::cold_predecessor(self)
    }
}

impl SelectedRequiredPublicationHead {
    #[doc(hidden)]
    pub fn retain_for_test(selected: &Arc<OnceLock<RecordedOutput>>) -> Result<Self, Stop> {
        Self::retain(selected)
    }
}

impl SelectedRequiredPublicationHead {
    #[doc(hidden)]
    pub fn warm_publication_for_test(&self) -> Option<RelationalRequiredOutputPublication> {
        Self::warm_publication(self)
    }
}
