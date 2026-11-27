//! Opaque shared handle to `WorthQueryApplicationOutputLineage` preserving the
//! exact `Arc<Mutex<...>>` allocation and `Send + Sync` bounds without exposing
//! nested lineage and recorded-output type depth across downstream facades.
use std::ops::Deref;
use std::sync::{Arc, Mutex, Weak};

use super::WorthQueryApplicationOutputLineage;

trait OutputLineageBacking: Send + Sync + std::panic::UnwindSafe + std::panic::RefUnwindSafe {
    fn as_mutex(&self) -> &Mutex<WorthQueryApplicationOutputLineage>;
}

impl OutputLineageBacking for Mutex<WorthQueryApplicationOutputLineage> {
    fn as_mutex(&self) -> &Mutex<WorthQueryApplicationOutputLineage> {
        self
    }
}

#[derive(Clone)]
pub struct SharedOutputLineage(Arc<dyn OutputLineageBacking>);

impl SharedOutputLineage {
    pub fn new(lineage: WorthQueryApplicationOutputLineage) -> Self {
        Self(Arc::new(Mutex::new(lineage)))
    }

    pub fn from_arc(owner: Arc<Mutex<WorthQueryApplicationOutputLineage>>) -> Self {
        Self(owner)
    }

    pub fn downgrade(&self) -> WeakOutputLineage {
        WeakOutputLineage(Arc::downgrade(&self.0))
    }
}

/// A handle that does not keep the lineage alive (public #429).
#[derive(Clone)]
pub struct WeakOutputLineage(Weak<dyn OutputLineageBacking>);

impl WeakOutputLineage {
    pub fn upgrade(&self) -> Option<SharedOutputLineage> {
        self.0.upgrade().map(SharedOutputLineage)
    }
}

impl Default for SharedOutputLineage {
    fn default() -> Self {
        Self::new(WorthQueryApplicationOutputLineage::default())
    }
}

impl Deref for SharedOutputLineage {
    type Target = Mutex<WorthQueryApplicationOutputLineage>;

    fn deref(&self) -> &Self::Target {
        OutputLineageBacking::as_mutex(&*self.0)
    }
}

const _: fn() = || {
    fn assert_auto_traits<T: Send + Sync + std::panic::UnwindSafe + std::panic::RefUnwindSafe>() {}
    assert_auto_traits::<SharedOutputLineage>();
};
