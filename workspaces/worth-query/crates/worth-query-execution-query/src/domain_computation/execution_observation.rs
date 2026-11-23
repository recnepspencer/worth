//! Owned observation lifetime for Query execution, independent of request authority.

use std::ops::Deref;

use worth_query_execution_lineage::facade::domain_computation::execution_observation::WorthQueryExecutionObservation as LineageObservation;
pub use worth_query_execution_lineage::facade::domain_computation::execution_observation::{
    is_installed, owned_control_scope, register_cleanup,
};
pub(crate) use worth_query_execution_lineage::facade::domain_computation::execution_observation::{
    record_outgoing_rows_revisited, record_retained_reads_visited, record_wave_charged,
    record_wave_event,
};

/// Records this thread's Query work while an owned listener exists.
/// It grants no execution, publication or installation authority.
#[must_use = "the observation ends when its guard is dropped"]
pub struct WorthQueryExecutionObservation(LineageObservation);

impl Deref for WorthQueryExecutionObservation {
    type Target = LineageObservation;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl WorthQueryExecutionObservation {
    /// Install passive Query, Signal and World observation on this thread.
    /// Take retained report vectors before dropping the final listener.
    pub fn install_on_this_thread() -> Self {
        Self(LineageObservation::install_on_this_thread())
    }

    /// The existing observer has admitted an advancement on this thread.
    /// No new event or counter is recorded by this owned observation.
    pub fn advancement_is_active_for_test(&self) -> bool {
        self.0.advancement_is_active_for_test()
    }
}
