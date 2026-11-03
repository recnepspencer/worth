//! Unit-only cancellation immediately before the real native freshness lookup.

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::entity_resolution::interruption_control::Action;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::entity_resolution::interruption_control::PENDING;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::entity_resolution::interruption_control::Pending;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;

pub struct FreshnessCancellationGuardForTest {
    pub previous: Option<Pending>,
    pub triggered: Rc<Cell<bool>>,
}
impl FreshnessCancellationGuardForTest {
    pub fn install_on_this_thread(
        source: WorthQueryCancellationSource,
        preceding_reads: usize,
    ) -> Self {
        Self::install(Action::Cancel(source), preceding_reads)
    }
    pub fn deadline_on_this_thread(deadline: std::time::Instant) -> Self {
        Self::install(Action::Deadline(deadline), 0)
    }
    pub fn install(action: Action, preceding_reads: usize) -> Self {
        assert!(worth_query_execution::court_doors_for_test::domain_computation::execution_observation::is_installed());
        let triggered = Rc::new(Cell::new(false));
        let previous = PENDING.replace(Some(Pending {
            action,
            remaining: preceding_reads,
            triggered: Rc::clone(&triggered),
        }));
        Self {
            previous,
            triggered,
        }
    }
    pub fn triggered(&self) -> bool {
        self.triggered.get()
    }
}
impl Drop for FreshnessCancellationGuardForTest {
    fn drop(&mut self) {
        PENDING.replace(self.previous.take());
    }
}
