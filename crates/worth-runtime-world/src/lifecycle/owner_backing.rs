//! Opaque strong and weak owner handles preserving the exact World owner
//! allocation and `Send + Sync` bounds without exposing internal state depth.
use std::ops::Deref;
use std::sync::{Arc, Weak};

trait WorldOwnerBacking<R>: Send + Sync {
    fn as_root(&self) -> &R;
    fn into_root(self: Arc<Self>) -> Arc<R>;
}

impl<R: Send + Sync> WorldOwnerBacking<R> for R {
    fn as_root(&self) -> &R {
        self
    }

    fn into_root(self: Arc<Self>) -> Arc<R> {
        self
    }
}

pub(super) struct StrongWorldOwner<R>(Arc<dyn WorldOwnerBacking<R>>);

impl<R: Send + Sync + 'static> StrongWorldOwner<R> {
    pub(super) fn from_arc(root: Arc<R>) -> Self {
        Self(root)
    }
}

impl<R> StrongWorldOwner<R> {
    pub(super) fn into_arc(self) -> Arc<R> {
        self.0.into_root()
    }

    pub(super) fn to_arc(&self) -> Arc<R> {
        Arc::clone(&self.0).into_root()
    }

    pub(super) fn downgrade(&self) -> WeakWorldOwner<R> {
        WeakWorldOwner(Arc::downgrade(&self.0))
    }
}

impl<R> Deref for StrongWorldOwner<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        WorldOwnerBacking::as_root(&*self.0)
    }
}

impl<R> AsRef<R> for StrongWorldOwner<R> {
    fn as_ref(&self) -> &R {
        WorldOwnerBacking::as_root(&*self.0)
    }
}

impl<R> Clone for StrongWorldOwner<R> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

pub(super) struct WeakWorldOwner<R>(Weak<dyn WorldOwnerBacking<R>>);

impl<R> WeakWorldOwner<R> {
    pub(super) fn upgrade(&self) -> Option<Arc<R>> {
        self.0.upgrade().map(WorldOwnerBacking::into_root)
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
