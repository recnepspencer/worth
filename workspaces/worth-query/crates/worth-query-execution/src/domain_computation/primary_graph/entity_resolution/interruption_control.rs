//! Unit-only cancellation immediately before the real native freshness lookup.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
thread_local! { static PENDING: RefCell<Option<Pending>> = const { RefCell::new(None) }; }
enum Action {
    Cancel(WorthQueryCancellationSource),
    Deadline {
        time: worth_execution::MonotonicTimeController,
        deadline: std::time::Instant,
    },
}
struct Pending {
    action: Action,
    remaining: usize,
    triggered: Rc<Cell<bool>>,
}
pub(in crate::domain_computation::primary_graph) struct FreshnessCancellationGuardForTest {
    previous: Option<Pending>,
    triggered: Rc<Cell<bool>>,
}
impl FreshnessCancellationGuardForTest {
    pub(in crate::domain_computation::primary_graph) fn install_on_this_thread(
        source: WorthQueryCancellationSource,
        preceding_reads: usize,
    ) -> Self {
        Self::install(Action::Cancel(source), preceding_reads)
    }
    pub(in crate::domain_computation::primary_graph) fn deadline_on_this_thread(
        time: worth_execution::MonotonicTimeController,
        deadline: std::time::Instant,
    ) -> Self {
        Self::install(Action::Deadline { time, deadline }, 0)
    }
    fn install(action: Action, preceding_reads: usize) -> Self {
        assert!(crate::domain_computation::execution_observation::is_installed());
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
    pub(in crate::domain_computation::primary_graph) fn triggered(&self) -> bool {
        self.triggered.get()
    }
}
impl Drop for FreshnessCancellationGuardForTest {
    fn drop(&mut self) {
        PENDING.replace(self.previous.take());
    }
}
pub(super) fn cancel_before_lookup_for_test() {
    let pending = PENDING.with(|slot| {
        let mut slot = slot.borrow_mut();
        let pending = slot.as_mut()?;
        if pending.remaining > 0 {
            pending.remaining -= 1;
            return None;
        }
        slot.take()
    });
    if let Some(pending) = pending {
        pending.triggered.set(true);
        match pending.action {
            Action::Cancel(source) => source.cancel(),
            Action::Deadline { time, deadline } => {
                let elapsed = deadline
                    .checked_duration_since(time.clock().now())
                    .expect("the held request is fresh before lookup");
                time.advance(elapsed).unwrap();
            }
        }
    }
}
