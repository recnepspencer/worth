use crate::history::retention::RetentionBusy;
pub mod active_registry;
pub mod head_selection;
pub mod identity;
pub use identity::SnapshotIdentityAllocator;
pub mod published_capacity;
pub mod published_closeout;
pub mod published_registry;
pub mod published_release;
pub use published_release::PublishedSnapshotReleaseObservation;
pub mod version_counts;

use std::sync::Arc;

use crate::identity::data::VersionId;
use crate::snapshots::data::SnapshotId;

use super::SnapshotHandleBinding;
use active_registry::ActiveSnapshotHandleRegistry;
use published_registry::PublishedSnapshotHandleRegistry;
pub use published_registry::PublishedSnapshotRegistrationMismatch;

pub use published_capacity::{
    PublishedSnapshotCapacityOwner, PublishedSnapshotReservationDenial,
    PublishedSnapshotSlotReservation,
};
pub use published_closeout::PublishedSnapshotCloseout;

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
pub struct SnapshotHandles {
    active: ActiveSnapshotHandleRegistry,
    /// Shared ownership so a release obligation can hold a weak reference to the
    /// registry it must close against without keeping it alive.
    published: Arc<PublishedSnapshotHandleRegistry>,
    next_snapshot_id: Arc<SnapshotIdentityAllocator>,
}

/// One registry's cost accounting, observed under that registry's lock alone.
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
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SnapshotHandleRegistryCostCounters {
    pub active_entries: u64,
    pub active_key_lookups: u64,
    pub active_mutations: u64,
    pub published_entries: u64,
    pub published_key_lookups: u64,
    pub published_mutations: u64,
}

impl SnapshotHandles {
    pub fn try_retained_registrations(&self) -> Result<(bool, bool), RetentionBusy> {
        Ok((!self.active.try_empty()?, !self.published.try_empty()?))
    }
    pub fn snapshot_identity_binding(&self) -> Arc<SnapshotIdentityAllocator> {
        Arc::clone(&self.next_snapshot_id)
    }

    /// A forked runtime starts from empty registries and its own identity
    /// allocator; it shares no snapshot authority with the runtime it came from.
    pub fn fork(&self) -> Self {
        Self::default()
    }

    pub fn active_count(&self) -> usize {
        self.active.count()
    }

    pub fn published_release_observation(
        &self,
        snapshot_id: SnapshotId,
    ) -> Option<Arc<PublishedSnapshotReleaseObservation>> {
        self.published.release_observation(snapshot_id)
    }
    pub fn owns_published_release(
        &self,
        observation: &PublishedSnapshotReleaseObservation,
    ) -> bool {
        observation.belongs_to(&self.published)
    }

    pub fn published_count(&self) -> usize {
        self.published.count()
    }

    pub fn next_snapshot_id(&self) -> Option<SnapshotId> {
        self.next_snapshot_id.allocate_active()
    }

    pub fn insert_active(&self, snapshot_id: SnapshotId, binding: SnapshotHandleBinding) {
        self.active.insert(snapshot_id, binding);
    }

    pub fn remove_active(&self, snapshot_id: SnapshotId) -> Option<SnapshotHandleBinding> {
        self.active.remove(snapshot_id)
    }

    /// The binding for one live active handle, copied out of the registry lock.
    pub fn active_binding(&self, snapshot_id: SnapshotId) -> Option<SnapshotHandleBinding> {
        self.active.binding(snapshot_id)
    }

    /// Read one immutable issued binding's branch width without copying its
    /// owned visibility basis. The independent locks are visited in canonical
    /// active-before-published order and never held together.
    pub fn selected_branch_bytes(&self, snapshot_id: SnapshotId) -> Option<usize> {
        self.active
            .selected_branch_bytes(snapshot_id)
            .or_else(|| self.published.selected_branch_bytes(snapshot_id))
    }

    /// Whether either registry still knows this identity, answered by one
    /// registry observation at a time. The published registry is consulted only
    /// when the active registry has already released its lock and answered no.
    pub fn is_known_snapshot(&self, snapshot_id: SnapshotId) -> bool {
        self.active.retains_handle(snapshot_id) || self.published.retains_handle(snapshot_id)
    }

    /// Copy the oldest active version without collecting the registry.
    pub fn oldest_active_version(&self) -> Option<VersionId> {
        self.active.oldest_version()
    }

    pub fn oldest_published_version(&self) -> Option<VersionId> {
        self.published.oldest_version()
    }

    pub fn insert_published(&self, snapshot_id: SnapshotId, binding: SnapshotHandleBinding) {
        self.published.insert(snapshot_id, binding);
    }

    pub(crate) fn remove_published(&self, snapshot_id: SnapshotId) -> Option<()> {
        self.published.remove(snapshot_id, None)
    }
    pub fn remove_published_accounted(
        &self,
        snapshot_id: SnapshotId,
        capacity: &PublishedSnapshotCapacityOwner,
    ) -> Option<()> {
        self.published.remove(snapshot_id, Some(capacity))
    }
    pub fn try_remove_original(
        &self,
        original: &Arc<PublishedSnapshotReleaseObservation>,
        capacity: &PublishedSnapshotCapacityOwner,
    ) -> Result<Option<bool>, PublishedSnapshotRegistrationMismatch> {
        self.published.try_remove_original(original, capacity)
    }

    pub fn published_binding(&self, snapshot_id: SnapshotId) -> Option<SnapshotHandleBinding> {
        self.published.binding(snapshot_id)
    }

    pub fn published_binding_for_version(
        &self,
        version_id: VersionId,
    ) -> Option<(SnapshotId, SnapshotHandleBinding)> {
        self.published.binding_for_version(version_id)
    }

    pub fn published_closeout(
        &self,
        capacity: Arc<PublishedSnapshotCapacityOwner>,
        snapshot_id: SnapshotId,
    ) -> Option<PublishedSnapshotCloseout> {
        self.published
            .release_observation(snapshot_id)
            .map(|original| {
                PublishedSnapshotCloseout::new(&self.published, capacity, snapshot_id, original)
            })
    }

    pub fn registry_cost_counters(&self) -> SnapshotHandleRegistryCostCounters {
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

impl SnapshotHandles {
    pub(crate) fn observe_published_root(
        &self,
        id: crate::snapshots::data::SnapshotId,
    ) -> Option<std::sync::Weak<crate::branch::root::RelationalBranchRoot>> {
        self.published
            .binding(id)
            .map(|binding| std::sync::Arc::downgrade(&binding.basis.retain_root()))
    }

    pub(crate) fn while_published_registry_locked<R>(
        &self,
        expected: usize,
        action: impl FnOnce() -> R,
    ) -> R {
        self.published.while_locked(expected, action)
    }
}

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait SnapshotHandlesFixturePort: Sized {
    fn observe_published_root_for_test(
        &self,
        id: crate::snapshots::data::SnapshotId,
    ) -> Option<std::sync::Weak<crate::branch::root::RelationalBranchRoot>>;
    fn remove_published_for_test(&self, snapshot_id: SnapshotId) -> Option<()>;
    fn while_published_registry_locked_for_test<R>(
        &self,
        expected: usize,
        action: impl FnOnce() -> R,
    ) -> R;
}

impl SnapshotHandlesFixturePort for SnapshotHandles {
    fn observe_published_root_for_test(
        &self,
        id: crate::snapshots::data::SnapshotId,
    ) -> Option<std::sync::Weak<crate::branch::root::RelationalBranchRoot>> {
        self.observe_published_root(id)
    }

    fn remove_published_for_test(&self, snapshot_id: SnapshotId) -> Option<()> {
        self.remove_published(snapshot_id)
    }

    fn while_published_registry_locked_for_test<R>(
        &self,
        expected: usize,
        action: impl FnOnce() -> R,
    ) -> R {
        self.while_published_registry_locked(expected, action)
    }
}

impl Default for SnapshotHandles {
    fn default() -> Self {
        Self {
            active: ActiveSnapshotHandleRegistry::new(),
            published: Arc::new(PublishedSnapshotHandleRegistry::new()),
            next_snapshot_id: Arc::new(SnapshotIdentityAllocator::new()),
        }
    }
}
