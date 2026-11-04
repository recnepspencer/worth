//! Sealed test doors over this module's private members.
use super::*;

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn resources_for_test(&self) -> &WorthQueryInvalidationResources {
        &self.resources
    }
}
