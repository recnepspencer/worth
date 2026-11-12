//! Scoped readonly evidence of container layout and held budget during assembly.
//! Byte totals sum observations; they are not allocation totals.
use std::cell::Cell;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SynchronousCaptureRetentionCountsForTest {
    pub assemblies: u64,
    pub accounted: u64,
    pub unaccounted: u64,
    pub missing_tickets: u64,
    pub mismatched_tickets: u64,
    pub container_bytes: u64,
    pub retained_bytes: u64,
}
thread_local! {
    static ACTIVE: Cell<Option<SynchronousCaptureRetentionCountsForTest>> = const {Cell::new(None)};
}
/// Readonly original-thread recorder; dropping it clears the scope during unwinding.
/// It provides no admission, allocation or publication control.
pub struct SynchronousCaptureRetentionForTest {
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}
impl SynchronousCaptureRetentionForTest {
    pub fn install_on_this_thread() -> Self {
        assert!(
            ACTIVE.get().is_none(),
            "capture retention observation cannot nest"
        );
        ACTIVE.set(Some(Default::default()));
        Self {
            _thread: std::marker::PhantomData,
        }
    }
    pub fn counts(&self) -> SynchronousCaptureRetentionCountsForTest {
        ACTIVE.get().expect("the recorder owns its live scope")
    }
}
impl Drop for SynchronousCaptureRetentionForTest {
    fn drop(&mut self) {
        ACTIVE.set(None);
    }
}
pub(super) fn record(accounted_request: bool, layout: usize, ticket: Option<u64>) {
    let Some(mut counts) = ACTIVE.get() else {
        return;
    };
    counts.assemblies = counts.assemblies.saturating_add(1);
    if accounted_request || ticket.is_some() {
        counts.accounted = counts.accounted.saturating_add(1);
        counts.missing_tickets = counts
            .missing_tickets
            .saturating_add(u64::from(ticket.is_none()));
        counts.mismatched_tickets = counts
            .mismatched_tickets
            .saturating_add(u64::from(ticket != Some(layout as u64)));
        counts.container_bytes = counts.container_bytes.saturating_add(layout as u64);
        counts.retained_bytes = counts.retained_bytes.saturating_add(ticket.unwrap_or(0));
    } else {
        counts.unaccounted = counts.unaccounted.saturating_add(1);
    }
    ACTIVE.set(Some(counts));
}
