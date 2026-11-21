//! Passive observation of wave scheduling decisions on one thread (#254).
//! Recording never changes control flow; it runs only under an installed
//! `WorthQueryExecutionObservation` and clears with it.
use std::cell::{Cell, RefCell};

thread_local! {
    static EVENTS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static UNITS: Cell<u64> = const { Cell::new(0) };
    static VISITED: Cell<u64> = const { Cell::new(0) };
    static CLEANUP_REGISTERED: Cell<bool> = const { Cell::new(false) };
}
fn observing() -> bool {
    if !crate::domain_computation::execution_observation::is_installed() {
        return false;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear);
    }
    true
}
fn clear() {
    CLEANUP_REGISTERED.with(|registered| registered.set(false));
    EVENTS.with(|events| events.borrow_mut().clear());
    UNITS.with(|units| units.set(0));
    VISITED.with(|visited| visited.set(0));
}
/// Adds work the wave charged, only while observed. Qualified use charges
/// its marking, comparisons and lookups here too: it is scheduling work.
pub(in crate::domain_computation::primary_graph) fn charged(units: u64) {
    if observing() {
        UNITS.with(|total| total.set(total.get().saturating_add(units)));
    }
}
/// Counts retained candidate reads qualified use visited in one lookup,
/// only while observed: an index visits the entries it needs, a scan all.
pub(in crate::domain_computation::primary_graph) fn visited_retained_reads(entries: u64) {
    if observing() {
        VISITED.with(|total| total.set(total.get().saturating_add(entries)));
    }
}
/// Records one event, formatted only while observed.
pub(super) fn record(event: impl FnOnce() -> String) {
    if observing() {
        EVENTS.with(|events| events.borrow_mut().push(event()));
    }
}
/// Records one task prepare outcome, formatted only while observed.
pub(super) fn record_prepare<S: super::ApplicationSchema>(
    i: usize,
    task: &super::Scheduled<S>,
    prepared: &Result<super::PlannedOutput, super::SynchronousOutputFailure>,
) {
    record(|| {
        let outcome = match prepared {
            Ok(super::PlannedOutput::Prepared(parent)) => {
                if parent.is_some() {
                    "prepared"
                } else {
                    "prepared-none"
                }
            }
            Ok(super::PlannedOutput::Recapture) => "recapture",
            Err(_) => "refused",
        };
        format!(
            "prepare task={i} subject={} family={} source={:?} {outcome}",
            task.subject,
            task.tree.family(),
            task.key.as_ref().map(|key| key.scope),
        )
    });
}
impl crate::domain_computation::execution_observation::WorthQueryExecutionObservation {
    /// The wave decisions recorded on this thread since the last call, in
    /// order: each ranked batch, each prepare with its occurrence identity and
    /// outcome, each break and each quiescent reclosure.
    pub fn wave_events_for_test(&self) -> Vec<String> {
        EVENTS.with(|events| std::mem::take(&mut *events.borrow_mut()))
    }
    /// The work units the wave charged on this thread since the last call.
    pub fn wave_charged_units_for_test(&self) -> u64 {
        UNITS.with(|units| units.replace(0))
    }
    /// Retained candidate reads qualified use visited since the last call.
    pub fn retained_reads_visited_for_test(&self) -> u64 {
        VISITED.with(|visited| visited.replace(0))
    }
}
