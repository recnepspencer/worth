use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

use crate::identity::data::VersionId;
use crate::snapshots::data::SnapshotId;

use super::super::SnapshotHandleBinding;
#[cfg(test)]
use super::SnapshotRegistryCostCounters;

/// The live active snapshot handles, owned behind their own lock so an observer
/// can open or close a snapshot without exclusive access to the runtime.
///
/// The guard is private to this file and never escapes it. A caller composing an
/// active fact with a published fact therefore cannot name this lock, let alone
/// hold it across the published registry's acquisition.
#[derive(Debug)]
pub(super) struct ActiveSnapshotHandleRegistry {
    entries: Mutex<ActiveEntries>,
    key_lookups: AtomicU64,
    mutations: AtomicU64,
}

#[derive(Debug)]
struct ActiveEntries {
    handles: HashMap<SnapshotId, SnapshotHandleBinding>,
    versions: super::version_counts::VersionCounts<VersionId>,
}

impl ActiveSnapshotHandleRegistry {
    pub(super) fn new() -> Self {
        Self {
            entries: Mutex::new(ActiveEntries {
                handles: HashMap::new(),
                versions: super::version_counts::VersionCounts::new(),
            }),
            key_lookups: AtomicU64::new(0),
            mutations: AtomicU64::new(0),
        }
    }

    pub(super) fn count(&self) -> usize {
        self.lock().handles.len()
    }

    pub(super) fn insert(&self, snapshot_id: SnapshotId, binding: SnapshotHandleBinding) {
        let mut entries = self.lock();
        assert!(
            /* Proof: The private snapshot id allocator is checked and shared by active and published handles; it never resets or wraps in this runtime. This insertion owns the issued id and holds the active registry Mutex. */
            !entries.handles.contains_key(&snapshot_id),
            "snapshot identity allocator collided with a live active handle"
        );
        self.key_lookups.fetch_add(1, Ordering::Relaxed);
        self.mutations.fetch_add(1, Ordering::Relaxed);
        entries.versions.insert(binding.version_id);
        entries.handles.insert(snapshot_id, binding);
    }

    pub(super) fn remove(&self, snapshot_id: SnapshotId) -> Option<SnapshotHandleBinding> {
        self.key_lookups.fetch_add(1, Ordering::Relaxed);
        let mut entries = self.lock();
        let removed = entries.handles.remove(&snapshot_id);
        if let Some(binding) = &removed {
            entries.versions.remove(&binding.version_id);
            self.mutations.fetch_add(1, Ordering::Relaxed);
        }
        removed
    }

    /// The binding for one live active handle, copied out of the registry lock.
    pub(super) fn binding(&self, snapshot_id: SnapshotId) -> Option<SnapshotHandleBinding> {
        self.key_lookups.fetch_add(1, Ordering::Relaxed);
        self.lock().handles.get(&snapshot_id).cloned()
    }

    pub(super) fn selected_branch_bytes(&self, snapshot_id: SnapshotId) -> Option<usize> {
        self.key_lookups.fetch_add(1, Ordering::Relaxed);
        self.lock()
            .handles
            .get(&snapshot_id)
            .map(|binding| binding.basis.branch_id().0.len())
    }

    pub(super) fn selected_root(
        &self,
        snapshot_id: SnapshotId,
    ) -> Option<std::sync::Arc<crate::branch::RelationalBranchRoot>> {
        self.key_lookups.fetch_add(1, Ordering::Relaxed);
        self.lock()
            .handles
            .get(&snapshot_id)
            .map(|binding| std::sync::Arc::clone(binding.basis.root()))
    }

    pub(super) fn retains_handle(&self, snapshot_id: SnapshotId) -> bool {
        self.lock().handles.contains_key(&snapshot_id)
    }

    pub(super) fn oldest_version(&self) -> Option<VersionId> {
        self.lock().versions.oldest().copied()
    }

    #[cfg(test)]
    pub(super) fn cost_counters(&self) -> SnapshotRegistryCostCounters {
        SnapshotRegistryCostCounters {
            entries: self.lock().handles.len() as u64,
            key_lookups: self.key_lookups.load(Ordering::Relaxed),
            mutations: self.mutations.load(Ordering::Relaxed),
        }
    }

    fn lock(&self) -> MutexGuard<'_, ActiveEntries> {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
