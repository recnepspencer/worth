//! Sealed test doors over this module's private members.
use super::*;

impl<T> RunDirectory<T> {
    #[doc(hidden)]
    pub fn ensure_for_test(
        &mut self,
        level: usize,
        policy: super::super::StorageControl<'_, '_>,
    ) -> Result<(), StoreDenial> {
        Self::ensure(self, level, policy)
    }
}
