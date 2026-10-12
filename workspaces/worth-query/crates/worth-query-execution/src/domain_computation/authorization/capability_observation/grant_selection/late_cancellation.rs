//! Owned, unit-only cancellation at the actual grant-selection join door.
use std::cell::RefCell;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
thread_local! { static SOURCE: RefCell<Option<WorthQueryCancellationSource>> = const { RefCell::new(None) }; }
pub(in crate::domain_computation) struct CancelGrantSelectionGuardForTest(
    Option<WorthQueryCancellationSource>,
);
impl CancelGrantSelectionGuardForTest {
    pub(in crate::domain_computation) fn install_on_this_thread(
        source: WorthQueryCancellationSource,
    ) -> Self {
        assert!(crate::domain_computation::execution_observation::is_installed());
        Self(SOURCE.replace(Some(source)))
    }
}
impl Drop for CancelGrantSelectionGuardForTest {
    fn drop(&mut self) {
        SOURCE.replace(self.0.take());
    }
}
pub(super) fn cancel_at_join() {
    if let Some(source) = SOURCE.take() {
        source.cancel();
    }
}
