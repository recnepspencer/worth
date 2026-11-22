//! Sealed test doors over this module's private members.
use super::*;

impl FactPayload {
    #[doc(hidden)]
    pub fn copy_performed_field_reads_for_test(
        facts: &Self,
        control: StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        Self::copy_performed_field_reads(facts, control)
    }
}
