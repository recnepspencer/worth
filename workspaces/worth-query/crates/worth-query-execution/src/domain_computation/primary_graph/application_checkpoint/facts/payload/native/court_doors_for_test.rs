//! Sealed test doors over this module's private members.
use super::*;

impl NativeFactPayloadBuilder {
    #[doc(hidden)]
    pub fn allocate_completed_read_for_test(
        count: usize,
        control: StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        Self::allocate_completed_read(count, control)
    }
}
