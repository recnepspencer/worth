use super::reachability::HistoryReachabilitySlot;
use super::{
    lock_index, CompositeCommitParent, CompositeHistoryCatalogEntry, CompositeHistoryCatalogState,
};
use crate::identity::CompositeCommitIdentity;
use std::sync::{Arc, OnceLock};

/// Allocated before effects and carried through publication. Hash indexes
/// select occurrences at admission; final installation writes these exact slots.
#[derive(Debug)]
pub(super) struct ReservedHistorySlots {
    entry: Arc<OnceLock<CompositeHistoryCatalogEntry>>,
    pub(super) reachability: HistoryReachabilitySlot,
}

impl ReservedHistorySlots {
    pub(super) fn reserve(
        state: &mut CompositeHistoryCatalogState,
        identity: &CompositeCommitIdentity,
    ) -> Self {
        let entry = Arc::new(OnceLock::new());
        assert!(state
            .entries
            .insert(identity.clone(), Arc::clone(&entry))
            .is_none());
        let reachability = lock_index(&state.reachability).reserve(identity.clone());
        Self {
            entry,
            reachability,
        }
    }

    pub(super) fn install(
        &self,
        state: &mut CompositeHistoryCatalogState,
        entry: CompositeHistoryCatalogEntry,
    ) {
        let identity = entry.identity().clone();
        let root = matches!(entry.commit().parent(), CompositeCommitParent::Root);
        lock_index(&state.reachability).install_reserved_slot(&self.reachability);
        self.entry
            .set(entry)
            .expect("one reservation populates one immutable occurrence");
        super::counters::lock_counters(&state.counters).record_reserved_entry_write();
        if root {
            state.root = Some(identity);
            state.root_ever_installed = true;
        }
    }
}
