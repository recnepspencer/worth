//! Sealed test doors over this module's private members.
use super::*;

impl PreparedInputReuseKey {
    #[doc(hidden)]
    pub fn new_for_test( selection: WorthQueryObservedSourceSelection, input_identity: [u8; 32], installed_edition: InstalledProducerEdition) -> Self {
        Self::new(selection, input_identity, installed_edition)
    }
}
