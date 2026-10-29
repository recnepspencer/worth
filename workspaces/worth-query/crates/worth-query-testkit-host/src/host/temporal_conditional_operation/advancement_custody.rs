//! The wake pump is also the executing host door for conditional reentry.
pub use super::{primary_graph, world::CourtroomWorld};
pub use std::num::NonZeroUsize;
pub use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Denial,
        WorthQueryManagedComputationResourceDenial as Resource,
    },
    primary_graph::{
        advancement_requests_on_this_thread_for_test as reports,
        bound_advancement_requests_on_this_thread_for_test as bound,
        installed_source_reads_on_this_thread_for_test as reads,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryExecutionPlacementForTest as Placement,
    },
};
pub struct Restore(
    pub Placement,
    pub Option<worth_foundational::ExecutionBudget>,
);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}

#[path = "advancement_custody/conditional_delivery.rs"]
pub mod conditional_delivery;

#[path = "advancement_custody/width_differential.rs"]
pub mod width_differential;
