//! A bounded standalone provider fixture gates its real checkpoint restore at both placements.
//! The fixture has no product World, so it does not borrow an installed runtime's phase.

use crate::domain_computation::managed_run::tests::*;
use worth_query_execution::facade::domain_computation::bound_advancement_requests_on_this_thread_for_test as bound;
use worth_query_execution::facade::domain_computation::place_managed_computations_on_this_thread_for_test as place;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::advancement_host::AdvancementHost;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial as Denial;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionPlacementForTest as Placement;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
thread_local! { pub static RESTORES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
pub fn record_restore() {
    RESTORES.with(|n| n.set(n.get() + 1));
}
pub fn restores() -> usize {
    RESTORES.with(std::cell::Cell::get)
}
pub struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}
