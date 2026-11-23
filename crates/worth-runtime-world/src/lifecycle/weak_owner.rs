//! Weak service ports preserve the exact World owner allocation and lifetime.
use std::sync::{Arc, Weak};

pub(super) struct WeakWorldOwner<R>(Weak<dyn OwnerBacking<R>>);

trait OwnerBacking<R>: Send + Sync {
    fn into_root(self: Arc<Self>) -> Arc<R>;
}

impl<R: Send + Sync> OwnerBacking<R> for R {
    fn into_root(self: Arc<Self>) -> Arc<R> {
        self
    }
}

impl<R: Send + Sync + 'static> WeakWorldOwner<R> {
    pub(super) fn from_weak(owner: Weak<R>) -> Self {
        Self(owner)
    }
}

impl<R> WeakWorldOwner<R> {
    pub(super) fn upgrade(&self) -> Option<Arc<R>> {
        self.0.upgrade().map(OwnerBacking::into_root)
    }
}

impl<R> Clone for WeakWorldOwner<R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<R> std::fmt::Debug for WeakWorldOwner<R> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("WeakWorldOwner")
            .field(&self.0)
            .finish()
    }
}
