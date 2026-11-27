//! Custody-pinned cold naming store handle.
use std::any::Any;
use std::fmt;
use std::sync::Arc;

use crate::lineage::naming_pages::store::ColdNamingStore;

/// A [`ColdNamingStore`] pinned together with an opaque custody guard (such as
/// the selected `Arc<RelationalCheckpointDirectory>`) so concurrent checkpoint
/// switches cannot retire or drop the underlying members during an in-flight
/// fault or capture.
#[derive(Clone)]
pub struct PinnedNamingStore {
    store: Arc<ColdNamingStore>,
    custody: Option<Arc<dyn Any + Send + Sync>>,
}

impl fmt::Debug for PinnedNamingStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PinnedNamingStore")
            .field("store", &self.store)
            .finish_non_exhaustive()
    }
}

impl PinnedNamingStore {
    pub fn new(store: Arc<ColdNamingStore>, custody: Arc<dyn Any + Send + Sync>) -> Self {
        Self {
            store,
            custody: Some(custody),
        }
    }

    pub fn unpinned(store: Arc<ColdNamingStore>) -> Self {
        Self {
            store,
            custody: None,
        }
    }

    pub fn store(&self) -> &Arc<ColdNamingStore> {
        &self.store
    }

    pub fn custody(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.custody.clone()
    }
}

impl std::ops::Deref for PinnedNamingStore {
    type Target = ColdNamingStore;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}
