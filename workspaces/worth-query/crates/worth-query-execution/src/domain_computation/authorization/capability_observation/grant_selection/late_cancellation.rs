//! Owned, unit-only cancellation at the actual grant-selection join door.
use std::cell::RefCell;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
thread_local! { static SOURCE: RefCell<Option<(WorthQueryCancellationSource, bool)>> = const { RefCell::new(None) }; }
pub(in crate::domain_computation) struct CancelGrantSelectionGuardForTest(
    Option<(WorthQueryCancellationSource, bool)>,
);
impl CancelGrantSelectionGuardForTest {
    pub(in crate::domain_computation) fn install_on_this_thread(
        source: WorthQueryCancellationSource,
    ) -> Self {
        assert!(crate::domain_computation::execution_observation::is_installed());
        Self(SOURCE.replace(Some((source, false))))
    }
    pub(in crate::domain_computation) fn at_observation_on_this_thread(
        source: WorthQueryCancellationSource,
    ) -> Self {
        assert!(crate::domain_computation::execution_observation::is_installed());
        Self(SOURCE.replace(Some((source, true))))
    }
}
impl Drop for CancelGrantSelectionGuardForTest {
    fn drop(&mut self) {
        SOURCE.replace(self.0.take());
    }
}
pub(super) fn cancel_at_join() {
    cancel(false);
}
pub(super) fn cancel_at_observation() {
    cancel(true);
}
fn cancel(observation: bool) {
    SOURCE.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot
            .as_ref()
            .is_some_and(|(_, target)| *target == observation)
        {
            let (source, _) = slot.take().unwrap();
            source.cancel();
        }
    });
}
