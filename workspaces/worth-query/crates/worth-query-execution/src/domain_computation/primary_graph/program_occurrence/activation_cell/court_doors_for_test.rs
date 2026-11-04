//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProgramActivationCell {
    #[doc(hidden)]
    pub fn unpublished_for_test() -> Self {
        Self::unpublished()
    }
}

impl WorthQueryProgramActivationCell {
    #[doc(hidden)]
    pub fn published_for_test(&self) -> Option<EntityId> {
        Self::published(self)
    }
}
