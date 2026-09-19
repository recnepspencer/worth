mod active_registry;
mod published_capacity;
mod published_closeout;
mod published_registry;

#[cfg(test)]
mod cross_registry_composition_tests;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::identity::data::VersionId;
use crate::snapshots::data::SnapshotId;

use super::SnapshotHandleBinding;
use active_registry::ActiveSnapshotHandleRegistry;
use published_registry::PublishedSnapshotHandleRegistry;

pub(crate) use published_capacity::{
    PublishedSnapshotCapacityOwner, PublishedSnapshotSlotReservation,
};
pub(crate) use published_closeout::PublishedSnapshotCloseout;

/// The runtime's two snapshot handle registries and the identity allocator they
/// share.
///
/// # Lock discipline
///
/// The discipline is enforced structurally, not by convention, and there is no
/// runtime check standing in for it. The active and published registries are
/// independent authorities behind independent locks, and neither guard type is
/// nameable from this module: each registry keeps its guard private to its own
/// file and answers only in owned facts. A paired cross-registry hold is
/// therefore not writable here, so no operation can hold one registry's guard
/// while acquiring the other's.
///
/// Every cross-registry answer is instead composed from per-registry
/// observations taken one at a time, in the canonical sequence active before
/// published. [`Self::is_known_snapshot`] and the test-only cost counter
/// composition both follow it. Should a genuinely atomic cross-registry fact ever be
/// required, it must keep that same order and be taken by one named function in
/// this module rather than assembled at a call site.
///
/// Composing rather than pairing loses nothing. A snapshot identity never
/// migrates between registries: [`Self::insert_active`] rejects collision with a
/// live active handle, [`Self::insert_published`] rejects collision with a live
/// published handle or version, and no path moves a binding from one registry to
/// the other. An identity is therefore resident in at most one registry, and the
/// sequential union equals the atomic union for every caller that uses the
/// answer after the lock is released, which is all of them.
#[derive(Debug)]
pub(crate) struct SnapshotHandles {
    active: ActiveSnapshotHandleRegistry,
    /// Shared ownership so a release obligation can hold a weak reference to the
    /// registry it must close against without keeping it alive.
    published: Arc<PublishedSnapshotHandleRegistry>,
    next_snapshot_id: Arc<AtomicU64>,
}

/// One registry's cost accounting, observed under that registry's lock alone.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct SnapshotRegistryCostCounters {
    entries: u64,
    key_lookups: u64,
    mutations: u64,
}

/// Both registries' cost accounting, composed from two independent per-registry
/// observations rather than one cross-registry instant. Every consumer reads
/// before/after deltas around a single-threaded operation, so the composition
/// boundary is not observable to them.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct SnapshotHandleRegistryCostCounters {
    pub(crate) active_entries: u64,
    pub(crate) active_key_lookups: u64,
    pub(crate) active_mutations: u64,
    pub(crate) published_entries: u64,
    pub(crate) published_key_lookups: u64,
    pub(crate) published_mutations: u64,
}

impl SnapshotHandles {
    pub(crate) fn snapshot_identity_binding(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.next_snapshot_id)
    }

    pub(crate) fn new() -> Self {
        Self {
            active: ActiveSnapshotHandleRegistry::new(),
            published: Arc::new(PublishedSnapshotHandleRegistry::new()),
            next_snapshot_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// A forked runtime starts from empty registries and its own identity
    /// allocator; it shares no snapshot authority with the runtime it came from.
    pub(crate) fn fork(&self) -> Self {
        Self::new()
    }

    pub(crate) fn active_count(&self) -> usize {
        self.active.count()
    }

    pub(crate) fn published_count(&self) -> usize {
        self.published.count()
    }

    pub(crate) fn next_snapshot_id(&self) -> Option<SnapshotId> {
        self.next_snapshot_id
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                (current != 0).then(|| current.checked_add(1).unwrap_or(0))
            })
            .ok()
            .map(SnapshotId)
    }

    pub(crate) fn insert_active(&self, snapshot_id: SnapshotId, binding: SnapshotHandleBinding) {
        self.active.insert(snapshot_id, binding);
    }

    pub(crate) fn remove_active(&self, snapshot_id: SnapshotId) -> Option<SnapshotHandleBinding> {
        self.active.remove(snapshot_id)
    }

    /// The binding for one live active handle, copied out of the registry lock.
    pub(crate) fn active_binding(&self, snapshot_id: SnapshotId) -> Option<SnapshotHandleBinding> {
        self.active.binding(snapshot_id)
    }

    /// Whether either registry still knows this identity, answered by one
    /// registry observation at a time. The published registry is consulted only
    /// when the active registry has already released its lock and answered no.
    pub(crate) fn is_known_snapshot(&self, snapshot_id: SnapshotId) -> bool {
        self.active.retains_handle(snapshot_id) || self.published.retains_handle(snapshot_id)
    }

    /// Every version an active handle currently retains, collected so no caller
    /// scans the registry while holding its lock.
    pub(crate) fn active_versions(&self) -> Vec<VersionId> {
        self.active.versions()
    }

    pub(crate) fn insert_published(&self, snapshot_id: SnapshotId, binding: SnapshotHandleBinding) {
        self.published.insert(snapshot_id, binding);
    }

    pub(crate) fn remove_published(
        &self,
        snapshot_id: SnapshotId,
    ) -> Option<SnapshotHandleBinding> {
        self.published.remove(snapshot_id)
    }

    pub(crate) fn published_versions(&self) -> Vec<VersionId> {
        self.published.versions()
    }

    pub(crate) fn published_binding(
        &self,
        snapshot_id: SnapshotId,
    ) -> Option<SnapshotHandleBinding> {
        self.published.binding(snapshot_id)
    }

    pub(crate) fn published_binding_for_version(
        &self,
        version_id: VersionId,
    ) -> Option<(SnapshotId, SnapshotHandleBinding)> {
        self.published.binding_for_version(version_id)
    }

    pub(crate) fn published_closeout(
        &self,
        capacity: Arc<PublishedSnapshotCapacityOwner>,
        snapshot_id: SnapshotId,
    ) -> Option<PublishedSnapshotCloseout> {
        self.published
            .retains_handle(snapshot_id)
            .then(|| PublishedSnapshotCloseout::new(&self.published, capacity, snapshot_id))
    }

    #[cfg(test)]
    pub(crate) fn registry_cost_counters(&self) -> SnapshotHandleRegistryCostCounters {
        let active = self.active.cost_counters();
        let published = self.published.cost_counters();
        SnapshotHandleRegistryCostCounters {
            active_entries: active.entries,
            active_key_lookups: active.key_lookups,
            active_mutations: active.mutations,
            published_entries: published.entries,
            published_key_lookups: published.key_lookups,
            published_mutations: published.mutations,
        }
    }
}
