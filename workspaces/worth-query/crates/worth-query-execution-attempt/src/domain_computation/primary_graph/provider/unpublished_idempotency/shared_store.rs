//! Type-erased shared handle to the unpublished-idempotency store.
//!
//! Uses unsized coercion on the existing `Arc<Mutex<...>>` allocation so
//! rustdoc auto-trait synthesis does not traverse retained unpublished-attempt
//! and lineage internals through provider handles.

use std::ops::Deref;
use std::sync::{Arc, Mutex};

use super::WorthQueryUnpublishedIdempotencyStore;

trait UnpublishedIdempotencyStoreBacking:
    Send + Sync + std::panic::UnwindSafe + std::panic::RefUnwindSafe
{
    fn as_mutex(&self) -> &Mutex<WorthQueryUnpublishedIdempotencyStore>;
}

impl UnpublishedIdempotencyStoreBacking for Mutex<WorthQueryUnpublishedIdempotencyStore> {
    fn as_mutex(&self) -> &Mutex<WorthQueryUnpublishedIdempotencyStore> {
        self
    }
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph::provider) struct SharedUnpublishedIdempotencyStore(
    Arc<dyn UnpublishedIdempotencyStoreBacking>,
);

impl SharedUnpublishedIdempotencyStore {
    pub(in crate::domain_computation::primary_graph::provider) fn new(
        store: WorthQueryUnpublishedIdempotencyStore,
    ) -> Self {
        Self(Arc::new(Mutex::new(store)))
    }
}

impl Deref for SharedUnpublishedIdempotencyStore {
    type Target = Mutex<WorthQueryUnpublishedIdempotencyStore>;

    fn deref(&self) -> &Self::Target {
        UnpublishedIdempotencyStoreBacking::as_mutex(&*self.0)
    }
}

const _: fn() = || {
    fn assert_auto_traits<T: Send + Sync + std::panic::UnwindSafe + std::panic::RefUnwindSafe>() {}
    assert_auto_traits::<SharedUnpublishedIdempotencyStore>();
};
