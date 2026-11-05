//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryConditionalOperationCell<Schema> {
    #[doc(hidden)]
    pub fn lock_operation_for_test(
        &self,
    ) -> MutexGuard<'_, Box<dyn WorthQueryInstalledConditionalOperation<Schema>>> {
        Self::lock_operation(self)
    }
}
