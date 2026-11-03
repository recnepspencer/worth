//! Sealed test doors over this module's private members.
use super::*;

impl FactPayload {
    #[doc(hidden)]
    pub fn copy_funded_for_test( facts: &(impl FactSequence + ?Sized), control: StorageControl<'_, '_>) -> Result<Self, StoreDenial> {
        Self::copy_funded(facts, control)
    }
}
