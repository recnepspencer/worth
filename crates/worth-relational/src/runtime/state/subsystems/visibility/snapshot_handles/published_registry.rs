use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use crate::identity::data::VersionId;
use crate::snapshots::data::SnapshotId;

use super::super::SnapshotHandleBinding;
#[cfg(test)]
use super::SnapshotRegistryCostCounters;

/// The published snapshot handles, owned behind their own lock so publication
/// and closeout never demand exclusive access to the runtime.
///
/// The guard is private to this file and never escapes it, and this registry is
/// the single authority for published handle removal: both the runtime path and
/// a dropped closeout obligation release through [`Self::remove`].
#[derive(Debug)]
pub(super) struct PublishedSnapshotHandleRegistry {
    entries: Mutex<PublishedSnapshotHandleEntries>,
}

#[derive(Debug, Default)]
struct PublishedSnapshotHandleEntries {
    by_id: HashMap<SnapshotId, SnapshotHandleBinding>,
    by_version: HashMap<VersionId, SnapshotId>,
    key_lookups: u64,
    mutations: u64,
}

impl PublishedSnapshotHandleRegistry {
    pub(super) fn new() -> Self {
        Self {
            entries: Mutex::new(PublishedSnapshotHandleEntries::default()),
        }
    }

    pub(super) fn count(&self) -> usize {
        self.lock().by_id.len()
    }

    pub(super) fn insert(&self, snapshot_id: SnapshotId, binding: SnapshotHandleBinding) {
        let mut entries = self.lock();
        let version_id = binding.version_id;
        assert!(
            !entries.by_id.contains_key(&snapshot_id),
            "snapshot identity allocator collided with a live published handle"
        );
        assert!(
            !entries.by_version.contains_key(&version_id),
            "one published snapshot already owns this exact version"
        );
        entries.key_lookups = entries.key_lookups.saturating_add(2);
        entries.mutations = entries.mutations.saturating_add(2);
        let previous_binding = entries.by_id.insert(snapshot_id, binding);
        let previous_snapshot = entries.by_version.insert(version_id, snapshot_id);
        debug_assert!(previous_binding.is_none());
        debug_assert!(previous_snapshot.is_none());
    }

    pub(super) fn remove(&self, snapshot_id: SnapshotId) -> Option<SnapshotHandleBinding> {
        self.lock().remove(snapshot_id)
    }

    pub(super) fn retains_handle(&self, snapshot_id: SnapshotId) -> bool {
        self.lock().by_id.contains_key(&snapshot_id)
    }

    pub(super) fn binding(&self, snapshot_id: SnapshotId) -> Option<SnapshotHandleBinding> {
        self.lock().by_id.get(&snapshot_id).cloned()
    }

    pub(super) fn selected_branch_bytes(&self, snapshot_id: SnapshotId) -> Option<usize> {
        self.lock()
            .by_id
            .get(&snapshot_id)
            .map(|binding| binding.basis.branch_id().0.len())
    }

    pub(super) fn selected_root(
        &self,
        snapshot_id: SnapshotId,
    ) -> Option<std::sync::Arc<crate::branch::RelationalBranchRoot>> {
        self.lock()
            .by_id
            .get(&snapshot_id)
            .map(|binding| std::sync::Arc::clone(binding.basis.root()))
    }

    pub(super) fn binding_for_version(
        &self,
        version_id: VersionId,
    ) -> Option<(SnapshotId, SnapshotHandleBinding)> {
        let entries = self.lock();
        let snapshot_id = *entries.by_version.get(&version_id)?;
        entries
            .by_id
            .get(&snapshot_id)
            .cloned()
            .map(|binding| (snapshot_id, binding))
    }

    /// Every version a published handle currently retains, collected so no
    /// caller scans the registry while holding its lock.
    pub(super) fn versions(&self) -> Vec<VersionId> {
        self.lock()
            .by_id
            .values()
            .map(|binding| binding.version_id)
            .collect()
    }

    #[cfg(test)]
    pub(super) fn cost_counters(&self) -> SnapshotRegistryCostCounters {
        let entries = self.lock();
        SnapshotRegistryCostCounters {
            entries: entries.by_id.len() as u64,
            key_lookups: entries.key_lookups,
            mutations: entries.mutations,
        }
    }

    fn lock(&self) -> MutexGuard<'_, PublishedSnapshotHandleEntries> {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl PublishedSnapshotHandleEntries {
    fn remove(&mut self, snapshot_id: SnapshotId) -> Option<SnapshotHandleBinding> {
        self.key_lookups = self.key_lookups.saturating_add(1);
        let binding = self.by_id.remove(&snapshot_id)?;
        self.mutations = self.mutations.saturating_add(1);
        self.key_lookups = self.key_lookups.saturating_add(1);
        if self.by_version.get(&binding.version_id) == Some(&snapshot_id) {
            self.key_lookups = self.key_lookups.saturating_add(1);
            self.mutations = self.mutations.saturating_add(1);
            self.by_version.remove(&binding.version_id);
        }
        Some(binding)
    }
}
