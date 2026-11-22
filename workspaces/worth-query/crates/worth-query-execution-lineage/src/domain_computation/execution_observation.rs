//! Owned observation lifetime for Query execution, independent of request authority.

use std::cell::{Cell, RefCell};

use worth_query_execution_contracts::facade::execution_observation::ExecutionObservationGuard;
pub use worth_query_execution_contracts::facade::execution_observation::{
    is_installed, owned_control_scope, register_cleanup,
};

thread_local! {
    static WAVE_EVENTS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static WAVE_UNITS: Cell<u64> = const { Cell::new(0) };
    static WAVE_CLEANUP_REGISTERED: Cell<bool> = const { Cell::new(false) };
}

fn observing_wave() -> bool {
    if !is_installed() {
        return false;
    }
    if WAVE_CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        register_cleanup(clear_wave);
    }
    true
}

fn clear_wave() {
    WAVE_CLEANUP_REGISTERED.with(|registered| registered.set(false));
    WAVE_EVENTS.with(|events| events.borrow_mut().clear());
    WAVE_UNITS.with(|units| units.set(0));
}

/// Adds work the synchronous output wave charged, only while observed.
pub fn record_wave_charged(units: u64) {
    if observing_wave() {
        WAVE_UNITS.with(|total| total.set(total.get().saturating_add(units)));
    }
}

/// Records one synchronous output wave event, formatted only while observed.
pub fn record_wave_event(event: impl FnOnce() -> String) {
    if observing_wave() {
        WAVE_EVENTS.with(|events| events.borrow_mut().push(event()));
    }
}

/// Records this thread's Query work while an owned listener exists.
/// It grants no execution, publication or installation authority.
#[must_use = "the observation ends when its guard is dropped"]
pub struct WorthQueryExecutionObservation(ExecutionObservationGuard);

impl WorthQueryExecutionObservation {
    /// Install passive Query, Signal and World observation on this thread.
    /// Take retained report vectors before dropping the final listener.
    pub fn install_on_this_thread() -> Self {
        Self(ExecutionObservationGuard::install_on_this_thread())
    }

    /// The existing observer has admitted an advancement on this thread.
    /// No new event or counter is recorded by this owned observation.
    pub fn advancement_is_active_for_test(&self) -> bool {
        self.0.advancement_is_active()
    }

    /// The wave decisions recorded on this thread since the last call, in
    /// order: each ranked batch, each prepare with its occurrence identity and
    /// outcome, each break and each quiescent reclosure.
    pub fn wave_events_for_test(&self) -> Vec<String> {
        WAVE_EVENTS.with(|events| std::mem::take(&mut *events.borrow_mut()))
    }

    /// The work units the wave charged on this thread since the last call.
    pub fn wave_charged_units_for_test(&self) -> u64 {
        WAVE_UNITS.with(|units| units.replace(0))
    }
}
