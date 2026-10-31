use super::published_release::PublishedSnapshotReleaseObservation;
mod original_cleanup;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

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
    by_id: HashMap<SnapshotId, PublishedSnapshotRegistration>,
    by_version: HashMap<VersionId, SnapshotId>,
    key_lookups: u64,
    mutations: u64,
}

#[derive(Debug)]
struct PublishedSnapshotRegistration {
    binding: SnapshotHandleBinding,
    release: Arc<PublishedSnapshotReleaseObservation>,
}

impl PublishedSnapshotHandleRegistry {
    pub(super) fn try_empty(&self) -> Result<bool, ()> {
        match self.entries.try_lock() {
            Ok(value) => Ok(value.by_id.is_empty()),
            Err(std::sync::TryLockError::Poisoned(error)) => {
                Ok(error.into_inner().by_id.is_empty())
            }
            Err(std::sync::TryLockError::WouldBlock) => Err(()),
        }
    }
    pub(super) fn new() -> Self {
        Self {
            entries: Mutex::new(PublishedSnapshotHandleEntries::default()),
        }
    }

    pub(super) fn count(&self) -> usize {
        self.lock().by_id.len()
    }

    pub(super) fn insert(
        self: &Arc<Self>,
        snapshot_id: SnapshotId,
        binding: SnapshotHandleBinding,
    ) {
        let mut entries = self.lock();
        let version_id = binding.version_id;
        assert!(
            /* Proof: Published insertion receives a reserved snapshot slot after one authorized commit settlement. Snapshot ids come from the shared checked non-wrapping allocator; each commit version has one settlement executor and live version allocation is unique. Recovery does not insert published snapshot handles. */
            !entries.by_id.contains_key(&snapshot_id),
            "snapshot identity allocator collided with a live published handle"
        );
        assert!(
            /* Proof: Published insertion receives a reserved snapshot slot after one authorized commit settlement. Snapshot ids come from the shared checked non-wrapping allocator; each commit version has one settlement executor and live version allocation is unique. Recovery does not insert published snapshot handles. */
            !entries.by_version.contains_key(&version_id),
            "one published snapshot already owns this exact version"
        );
        entries.key_lookups = entries.key_lookups.saturating_add(2);
        entries.mutations = entries.mutations.saturating_add(2);
        let release = PublishedSnapshotReleaseObservation::new(self, snapshot_id, &binding);
        let previous_binding = entries.by_id.insert(
            snapshot_id,
            PublishedSnapshotRegistration { binding, release },
        );
        let previous_snapshot = entries.by_version.insert(version_id, snapshot_id);
        debug_assert!(
            /* Proof: Published insertion receives a reserved snapshot slot after one authorized commit settlement. Snapshot ids come from the shared checked non-wrapping allocator; each commit version has one settlement executor and live version allocation is unique. Recovery does not insert published snapshot handles. */
            previous_binding.is_none()
        );
        debug_assert!(
            /* Proof: Published insertion receives a reserved snapshot slot after one authorized commit settlement. Snapshot ids come from the shared checked non-wrapping allocator; each commit version has one settlement executor and live version allocation is unique. Recovery does not insert published snapshot handles. */
            previous_snapshot.is_none()
        );
    }

    pub(super) fn remove(
        &self,
        snapshot_id: SnapshotId,
        capacity: Option<&super::PublishedSnapshotCapacityOwner>,
    ) -> Option<()> {
        let mut entries = self.lock();
        let registration = entries.remove(snapshot_id)?;
        registration.release.install_removed(registration.binding);
        if let Some(capacity) = capacity {
            capacity.release();
        }
        drop(entries);
        registration.release.finish_retirement_blocking();
        Some(())
    }

    pub(super) fn release_observation(
        &self,
        snapshot_id: SnapshotId,
    ) -> Option<Arc<PublishedSnapshotReleaseObservation>> {
        self.lock()
            .by_id
            .get(&snapshot_id)
            .map(|entry| entry.release.clone())
    }

    pub(super) fn retains_handle(&self, snapshot_id: SnapshotId) -> bool {
        self.lock().by_id.contains_key(&snapshot_id)
    }

    pub(super) fn binding(&self, snapshot_id: SnapshotId) -> Option<SnapshotHandleBinding> {
        self.lock()
            .by_id
            .get(&snapshot_id)
            .map(|entry| entry.binding.clone())
    }

    pub(super) fn selected_branch_bytes(&self, snapshot_id: SnapshotId) -> Option<usize> {
        self.lock()
            .by_id
            .get(&snapshot_id)
            .map(|binding| binding.binding.basis.branch_id().0.len())
    }

    pub(super) fn selected_root(
        &self,
        snapshot_id: SnapshotId,
    ) -> Option<std::sync::Arc<crate::branch::RelationalBranchRoot>> {
        self.lock()
            .by_id
            .get(&snapshot_id)
            .map(|binding| std::sync::Arc::clone(binding.binding.basis.root()))
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
            .map(|entry| (snapshot_id, entry.binding.clone()))
    }

    /// Every version a published handle currently retains, collected so no
    /// caller scans the registry while holding its lock.
    pub(super) fn versions(&self) -> Vec<VersionId> {
        self.lock()
            .by_id
            .values()
            .map(|entry| entry.binding.version_id)
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
    fn remove(&mut self, snapshot_id: SnapshotId) -> Option<PublishedSnapshotRegistration> {
        self.key_lookups = self.key_lookups.saturating_add(1);
        let registration = self.by_id.remove(&snapshot_id)?;
        let binding = &registration.binding;
        self.mutations = self.mutations.saturating_add(1);
        self.key_lookups = self.key_lookups.saturating_add(1);
        if self.by_version.get(&binding.version_id) == Some(&snapshot_id) {
            self.key_lookups = self.key_lookups.saturating_add(1);
            self.mutations = self.mutations.saturating_add(1);
            self.by_version.remove(&binding.version_id);
        }
        Some(registration)
    }
}
