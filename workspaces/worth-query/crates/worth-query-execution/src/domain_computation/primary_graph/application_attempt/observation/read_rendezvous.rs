//! One test-scoped rendezvous after the actual native field has been read.
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
std::thread_local! {
    static NEXT_READ: std::cell::RefCell<Option<WorthQueryCancellationSource>> = const { std::cell::RefCell::new(None) };
}
pub(in crate::domain_computation::primary_graph) struct ReadRendezvous;
impl ReadRendezvous {
    pub(in crate::domain_computation::primary_graph) fn cancel_next(
        source: WorthQueryCancellationSource,
    ) -> Self {
        NEXT_READ.with(|next| {
            assert!(next.borrow_mut().replace(source).is_none());
        });
        Self
    }
}
impl Drop for ReadRendezvous {
    fn drop(&mut self) {
        NEXT_READ.with(|next| next.borrow_mut().take());
    }
}
pub(super) fn reached() {
    if let Some(source) = NEXT_READ.with(|next| next.borrow_mut().take()) {
        source.cancel();
    }
}
