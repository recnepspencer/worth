//! Owned thread-local count of scope setup in the one scope engine.
use std::{cell::Cell, marker::PhantomData, rc::Rc};

/// Setup events recorded on this thread while an observation is installed.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExecutionScopeSetupCountsForTest {
    /// Kernel contexts constructed for an admitted scope.
    pub stage_setups: usize,
    /// Run limits derived from a lease, parent and request.
    pub limit_derivations: usize,
    /// Scope admissions, successful or refused.
    pub admissions: usize,
}

// Unarmed recording pays one thread-local read; nothing is allocated.
thread_local! {
    static COUNTS: Cell<Option<ExecutionScopeSetupCountsForTest>> = const { Cell::new(None) };
}

/// Passive recorder; it observes setup and cannot change any scope.
#[doc(hidden)]
pub struct ExecutionScopeSetupObservationForTest {
    previous: Option<ExecutionScopeSetupCountsForTest>,
    _thread: PhantomData<Rc<()>>,
}

impl ExecutionScopeSetupObservationForTest {
    #[doc(hidden)]
    pub fn install_on_this_thread_for_test() -> Self {
        Self {
            previous: COUNTS.replace(Some(ExecutionScopeSetupCountsForTest::default())),
            _thread: PhantomData,
        }
    }
    #[doc(hidden)]
    pub fn counts_for_test(&self) -> ExecutionScopeSetupCountsForTest {
        COUNTS.get().unwrap_or_default()
    }
}

impl Drop for ExecutionScopeSetupObservationForTest {
    fn drop(&mut self) {
        COUNTS.set(self.previous.take());
    }
}

fn record(update: impl FnOnce(&mut ExecutionScopeSetupCountsForTest)) {
    if let Some(mut counts) = COUNTS.get() {
        update(&mut counts);
        COUNTS.set(Some(counts));
    }
}

pub(crate) fn stage_setup() {
    record(|counts| counts.stage_setups += 1);
}
pub(crate) fn limit_derivation() {
    record(|counts| counts.limit_derivations += 1);
}
pub(crate) fn admission() {
    record(|counts| counts.admissions += 1);
}
