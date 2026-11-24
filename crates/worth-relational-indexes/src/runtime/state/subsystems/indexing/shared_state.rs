use std::fmt;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::IndexingState;

trait IndexingStateBacking: Send + Sync {
    fn lock(&self) -> &RwLock<IndexingState>;
}

impl IndexingStateBacking for RwLock<IndexingState> {
    fn lock(&self) -> &RwLock<IndexingState> {
        self
    }
}

/// Type-erased handle onto the indexing subsystem's authoritative mutable
/// state, preserving its single `Arc<RwLock<IndexingState>>` allocation while
/// bounding rustdoc auto-trait recursion across index-definition read bindings.
pub(crate) struct SharedIndexingState {
    backing: Arc<dyn IndexingStateBacking>,
}

impl fmt::Debug for SharedIndexingState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedIndexingState")
            .field("state", self.backing.lock())
            .finish()
    }
}

impl SharedIndexingState {
    pub(crate) fn new(state: IndexingState) -> Self {
        let backing: Arc<RwLock<IndexingState>> = Arc::new(RwLock::new(state));
        Self { backing }
    }

    pub(crate) fn share(&self) -> Self {
        Self {
            backing: Arc::clone(&self.backing),
        }
    }

    pub(crate) fn read(&self) -> RwLockReadGuard<'_, IndexingState> {
        self.backing.lock().read().expect(/* Proof: Authoritative subsystem guards are private owner operations: user callbacks, I/O and fallible preparation run outside them; guarded native updates operate on admitted phase values and do not unwind. */ "runtime subsystem lock poisoned")
    }

    pub(crate) fn write(&self) -> RwLockWriteGuard<'_, IndexingState> {
        self.backing.lock().write().expect(/* Proof: Authoritative subsystem guards are private owner operations: user callbacks, I/O and fallible preparation run outside them; guarded native updates operate on admitted phase values and do not unwind. */ "runtime subsystem lock poisoned")
    }
}

impl Default for SharedIndexingState {
    fn default() -> Self {
        Self::new(IndexingState::default())
    }
}
