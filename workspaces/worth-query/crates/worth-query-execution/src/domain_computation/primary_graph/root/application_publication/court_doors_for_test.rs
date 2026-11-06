//! Sealed test doors over this module's private members.
use super::*;

impl ApplicationPublication {
    #[doc(hidden)]
    pub fn owner_for_test(&self) -> &InstalledApplicationPublicationOwner {
        &self.owner
    }
}
