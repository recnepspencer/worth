//! Opaque strong and weak owner handles preserving the exact `SignalOwner`
//! allocation and generic auto-trait variance without exposing internal graph
//! storage depth across downstream boundaries.
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{Arc, Weak};

use super::SignalOwner;

trait SignalOwnerBacking<R>: Send + Sync {
    fn as_owner(&self) -> &R;
    fn into_owner(self: Arc<Self>) -> Arc<R>;
}

impl<R: Send + Sync> SignalOwnerBacking<R> for R {
    fn as_owner(&self) -> &R {
        self
    }

    fn into_owner(self: Arc<Self>) -> Arc<R> {
        self
    }
}

pub(crate) struct StrongSignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    owner: Arc<dyn SignalOwnerBacking<SignalOwner<D, I, T>>>,
    _marker: PhantomData<(D, I, T)>,
}

impl<D, I, T> StrongSignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(crate) fn from_arc(owner: Arc<SignalOwner<D, I, T>>) -> Self {
        Self {
            owner,
            _marker: PhantomData,
        }
    }
}

impl<D, I, T> StrongSignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn to_arc(&self) -> Arc<SignalOwner<D, I, T>> {
        Arc::clone(&self.owner).into_owner()
    }

    pub(crate) fn downgrade(&self) -> WeakSignalOwner<D, I, T> {
        WeakSignalOwner {
            owner: Arc::downgrade(&self.owner),
            _marker: PhantomData,
        }
    }
}

impl<D, I, T> Deref for StrongSignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    type Target = SignalOwner<D, I, T>;

    fn deref(&self) -> &Self::Target {
        SignalOwnerBacking::as_owner(&*self.owner)
    }
}

pub(crate) struct WeakSignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    owner: Weak<dyn SignalOwnerBacking<SignalOwner<D, I, T>>>,
    _marker: PhantomData<(D, I, T)>,
}

impl<D, I, T> WeakSignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn upgrade(&self) -> Option<Arc<SignalOwner<D, I, T>>> {
        self.owner.upgrade().map(SignalOwnerBacking::into_owner)
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "Addresses serve live allocation lookup and custody; canonical order uses declared identity"
    )]
    pub(super) fn matches_arc(&self, owner: &Arc<SignalOwner<D, I, T>>) -> bool {
        std::ptr::eq(
            self.owner.as_ptr() as *const (),
            Arc::as_ptr(owner) as *const (),
        )
    }
}

impl<D, I, T> Clone for WeakSignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
            _marker: PhantomData,
        }
    }
}
