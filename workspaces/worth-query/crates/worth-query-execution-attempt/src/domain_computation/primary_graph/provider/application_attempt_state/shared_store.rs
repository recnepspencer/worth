//! Type-erased shared handle to the primary-graph application attempt store.
//!
//! Uses unsized coercion on the existing `Arc<Mutex<...>>` allocation so
//! rustdoc auto-trait synthesis does not traverse deep attempt-store internals
//! through provider handles.

use std::ops::Deref;
use std::sync::{Arc, Mutex};

use super::WorthQueryPrimaryGraphApplicationAttemptStore;

trait ApplicationAttemptStoreBacking: Send + Sync {
    fn as_mutex(&self) -> &Mutex<WorthQueryPrimaryGraphApplicationAttemptStore>;
}

impl ApplicationAttemptStoreBacking for Mutex<WorthQueryPrimaryGraphApplicationAttemptStore> {
    fn as_mutex(&self) -> &Mutex<WorthQueryPrimaryGraphApplicationAttemptStore> {
        self
    }
}

#[derive(Clone)]
pub struct SharedApplicationAttemptStore(Arc<dyn ApplicationAttemptStoreBacking>);

impl Default for SharedApplicationAttemptStore {
    fn default() -> Self {
        Self::new(WorthQueryPrimaryGraphApplicationAttemptStore::default())
    }
}

impl SharedApplicationAttemptStore {
    pub fn new(store: WorthQueryPrimaryGraphApplicationAttemptStore) -> Self {
        Self(Arc::new(Mutex::new(store)))
    }
}

impl Deref for SharedApplicationAttemptStore {
    type Target = Mutex<WorthQueryPrimaryGraphApplicationAttemptStore>;

    fn deref(&self) -> &Self::Target {
        ApplicationAttemptStoreBacking::as_mutex(&*self.0)
    }
}
