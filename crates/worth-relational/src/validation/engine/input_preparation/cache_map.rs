use std::sync::{Arc, Mutex, OnceLock};

use crate::storage::substrate::SharedMap;

use super::allocation::{arc_bytes, CandidateInputAllocation};

/// The map itself is never cloned. AVL rotations move unique nodes; only cells
/// escape its lock, so a new key allocates exactly one map node and value.
pub(super) struct CandidateInputMap<K: Ord + Copy, V: Clone> {
    entries: Mutex<SharedMap<K, Arc<OnceLock<Option<V>>>>>,
}

impl<K: Ord + Copy, V: Clone> Default for CandidateInputMap<K, V> {
    fn default() -> Self {
        Self {
            entries: Mutex::new(SharedMap::new()),
        }
    }
}

impl<K: Ord + Copy, V: Clone> CandidateInputMap<K, V> {
    pub(super) fn cell(
        &self,
        key: K,
        allocation: &CandidateInputAllocation,
    ) -> Option<Arc<OnceLock<Option<V>>>> {
        if !allocation.claim(0) {
            return None;
        }
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(cell) = entries.get(&key) {
            return Some(cell.clone());
        }
        let bytes = SharedMap::<K, Arc<OnceLock<Option<V>>>>::entry_allocation_bytes()
            + arc_bytes::<OnceLock<Option<V>>>();
        if !allocation.claim(bytes) {
            return None;
        }
        let cell = Arc::new(OnceLock::new());
        let copied_bytes = entries.insert(key, cell.clone());
        debug_assert_eq!(copied_bytes, 0, "request cache map is never shared");
        Some(cell)
    }
}
