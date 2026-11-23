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
        let root = matches!(entry.commit().parent(), CompositeCommitParent::Root);
        let parent = match entry.commit().parent() {
            CompositeCommitParent::Root => None,
            CompositeCommitParent::Ordinary(parent) => Some(parent.commit().clone()),
        };
        self.install_with_parent(state, entry, parent.as_ref(), root);
    }

    pub(super) fn install_with_parent(
        &self,
        state: &mut CompositeHistoryCatalogState,
        entry: CompositeHistoryCatalogEntry,
        installed_parent: Option<&CompositeCommitIdentity>,
        marks_root: bool,
    ) {
        let identity = entry.identity().clone();
        let mut reachability = lock_index(&state.reachability);
        reachability.install_reserved_slot(&self.reachability);
        if let Some(parent) = installed_parent {
            reachability.install_forming_child(parent, &identity);
        }
        drop(reachability);
        self.entry
            .set(entry)
            .expect("one reservation populates one immutable occurrence");
        super::counters::lock_counters(&state.counters).record_reserved_entry_write();
        if marks_root {
            if state.root.is_none() {
                state.root = Some(identity);
            }
            state.root_ever_installed = true;
        }
    }
}
