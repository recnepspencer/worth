//! Moved whole from `domain_computation::primary_graph::application_query::managed_index::control`: the items below and every impl they own.
use crate::domain_computation::vocabulary::primary_graph::control::WorthQueryManagedIndexControl;
use crate::domain_computation::vocabulary::primary_graph::stop::WorthQueryManagedIndexStop as Stop;

impl<'a> WorthQueryManagedIndexControl<'a> {
    #[doc(hidden)]
    pub fn reserve_content_memory_for_test(
        &self,
        bytes: u64,
    ) -> Result<Option<worth_execution::ExecutionMemoryReservation>, Stop> {
        Self::reserve_content_memory(self, bytes)
    }
}
