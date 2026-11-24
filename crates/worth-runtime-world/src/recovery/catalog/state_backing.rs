//! Type-erased shared and weak handles onto `Mutex<RecoveryCatalogState>`.

use std::ops::Deref;
use std::sync::{Arc, Mutex, Weak};

use super::RecoveryCatalogState;

trait RecoveryCatalogStateBacking:
    Send + Sync + std::panic::UnwindSafe + std::panic::RefUnwindSafe + std::fmt::Debug
{
    fn as_lock(&self) -> &Mutex<RecoveryCatalogState>;
}

impl RecoveryCatalogStateBacking for Mutex<RecoveryCatalogState> {
    fn as_lock(&self) -> &Mutex<RecoveryCatalogState> {
        self
    }
}

#[derive(Debug, Clone)]
pub(super) struct SharedRecoveryCatalogState(Arc<dyn RecoveryCatalogStateBacking>);

impl SharedRecoveryCatalogState {
    pub(super) fn new(state: RecoveryCatalogState) -> Self {
        Self(Arc::new(Mutex::new(state)))
    }

    pub(super) fn downgrade(&self) -> WeakRecoveryCatalogState {
        WeakRecoveryCatalogState(Arc::downgrade(&self.0))
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "Addresses serve live allocation lookup and custody; canonical order uses declared identity"
    )]
    pub(super) fn affinity(&self) -> usize {
        Arc::as_ptr(&self.0) as *const () as usize
    }
}

impl Deref for SharedRecoveryCatalogState {
    type Target = Mutex<RecoveryCatalogState>;

    fn deref(&self) -> &Self::Target {
        self.0.as_lock()
    }
}

#[derive(Debug, Clone)]
pub(super) struct WeakRecoveryCatalogState(Weak<dyn RecoveryCatalogStateBacking>);

impl WeakRecoveryCatalogState {
    pub(super) fn upgrade(&self) -> Option<SharedRecoveryCatalogState> {
        self.0.upgrade().map(SharedRecoveryCatalogState)
    }
}

const _: fn() = || {
    fn assert_auto_traits<T: Send + Sync + std::panic::UnwindSafe + std::panic::RefUnwindSafe>() {}
    assert_auto_traits::<SharedRecoveryCatalogState>();
    assert_auto_traits::<WeakRecoveryCatalogState>();
};
