//! Sealed test doors over this module's private members.
use super::*;

impl<'a> WorthQueryManagedIndexControl<'a> {
    #[doc(hidden)]
    pub fn reserve_content_memory_for_test(
        &self,
        bytes: u64,
    ) -> Result<Option<worth_execution::ExecutionMemoryReservation>, Stop> {
        Self::reserve_content_memory(self, bytes)
    }
}
