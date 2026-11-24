pub mod cache;
pub mod replay_retention;
pub mod snapshot_handle_binding;
pub mod snapshot_handles;

pub use cache::{VisibilityCache, VisibilityResidency};
use replay_retention::ReplayRetentionIndex;
pub use snapshot_handle_binding::SnapshotHandleBinding;
pub use snapshot_handles::{
    PublishedSnapshotCapacityOwner, PublishedSnapshotCloseout, PublishedSnapshotReleaseObservation,
    PublishedSnapshotReservationDenial, PublishedSnapshotSlotReservation, SnapshotHandles,
    SnapshotIdentityAllocator,
};

use crate::config::data::runtime_config::RelationalRuntimeConfig;
use crate::snapshots::data::SnapshotId;

#[derive(Debug)]
pub struct VisibilitySubsystem {
    pub handles: SnapshotHandles,
    pub cache: std::sync::Arc<VisibilityCache>,
    /// Replay-pinned versions, owned behind their own lock so replay retention
    /// is taken and released without exclusive access to the runtime.
    replay_retention:
        crate::storage::substrate::owned_state::RuntimeOwnedState<ReplayRetentionIndex>,
    published_snapshot_capacity: std::sync::Arc<PublishedSnapshotCapacityOwner>,
}

impl VisibilitySubsystem {
    pub fn try_memory_read_registrations(&self) -> Result<(bool, bool, bool, bool), ()> {
        let (active, published) = self.handles.try_retained_registrations()?;
        let capacity = self.published_snapshot_capacity.occupied_handles() != 0;
        let replay = !self
            .replay_retention
            .try_read()
            .ok_or(())?
            .entries
            .is_empty();
        Ok((active, published, capacity, replay))
    }
    pub fn snapshot_identity_binding(&self) -> std::sync::Arc<SnapshotIdentityAllocator> {
        self.handles.snapshot_identity_binding()
    }

    pub fn published_snapshot_capacity_binding(
        &self,
    ) -> std::sync::Arc<PublishedSnapshotCapacityOwner> {
        std::sync::Arc::clone(&self.published_snapshot_capacity)
    }
    pub fn new(config: &RelationalRuntimeConfig) -> Self {
        Self {
            handles: SnapshotHandles::new(),
            cache: std::sync::Arc::new(VisibilityCache::new(config)),
            replay_retention: crate::storage::substrate::owned_state::RuntimeOwnedState::new(
                ReplayRetentionIndex::new(),
            ),
            published_snapshot_capacity: PublishedSnapshotCapacityOwner::new(
                config.publication.policy.max_published_snapshot_handles,
            ),
        }
    }

    pub fn fork(&self) -> Self {
        Self {
            handles: self.handles.fork(),
            cache: std::sync::Arc::new(self.cache.fork()),
            replay_retention: crate::storage::substrate::owned_state::RuntimeOwnedState::new(
                self.replay_retention.read().fork(),
            ),
            published_snapshot_capacity: PublishedSnapshotCapacityOwner::new(
                self.published_snapshot_capacity.maximum_handles(),
            ),
        }
    }
}

impl VisibilitySubsystem {
    pub fn cache_binding(&self) -> std::sync::Arc<VisibilityCache> {
        std::sync::Arc::clone(&self.cache)
    }

    pub fn active_snapshot_count(&self) -> usize {
        self.handles.active_count()
    }

    pub fn selected_snapshot_branch_bytes(
        &self,
        snapshot_id: crate::snapshots::data::SnapshotId,
    ) -> Option<usize> {
        self.handles.selected_branch_bytes(snapshot_id)
    }

    pub(crate) fn snapshot_handle_registry_cost_counters(
        &self,
    ) -> snapshot_handles::SnapshotHandleRegistryCostCounters {
        self.handles.registry_cost_counters()
    }

    pub(crate) fn visibility_cache_cost_counters(&self) -> cache::VisibilityCacheCostCounters {
        self.cache.cost_counters()
    }

    pub fn published_snapshot_handle_count(&self) -> usize {
        self.handles.published_count()
    }

    pub fn allocate_snapshot_id(&self) -> Option<SnapshotId> {
        self.handles.next_snapshot_id()
    }

    pub fn insert_active_handle(&self, snapshot_id: SnapshotId, binding: SnapshotHandleBinding) {
        self.handles.insert_active(snapshot_id, binding);
    }

    pub fn remove_active_handle(&self, snapshot_id: SnapshotId) -> Option<SnapshotHandleBinding> {
        self.handles.remove_active(snapshot_id)
    }

    pub fn active_handle_binding(&self, snapshot_id: SnapshotId) -> Option<SnapshotHandleBinding> {
        self.handles.active_binding(snapshot_id)
    }

    pub fn with_selected_snapshot_head<R>(
        &self,
        snapshot_id: SnapshotId,
        partition: crate::identity::data::PartitionId,
        read: impl FnOnce(crate::branch::projection_view::snapshot_head::SelectedSnapshotHead<'_>) -> R,
    ) -> Option<R> {
        self.handles
            .with_selected_head(snapshot_id, partition, read)
    }

    pub fn is_known_snapshot(&self, snapshot_id: SnapshotId) -> bool {
        self.handles.is_known_snapshot(snapshot_id)
    }

    pub fn retention_fence_version(
        &self,
        published_version: crate::identity::data::VersionId,
    ) -> crate::identity::data::VersionId {
        self.handles
            .oldest_active_version()
            .into_iter()
            .chain(self.replay_retention.read().oldest_version())
            .min()
            .unwrap_or(published_version)
    }

    pub fn historical_reconstruction_fence_version(
        &self,
        published_version: crate::identity::data::VersionId,
    ) -> crate::identity::data::VersionId {
        let retention_fence = self.retention_fence_version(published_version);
        self.handles
            .oldest_published_version()
            .map_or(retention_fence, |version| version.min(retention_fence))
    }

    pub fn insert_published_handle(&self, snapshot_id: SnapshotId, binding: SnapshotHandleBinding) {
        self.handles.insert_published(snapshot_id, binding);
    }

    pub fn remove_published_handle(&self, snapshot_id: SnapshotId) -> Option<()> {
        self.handles
            .remove_published_accounted(snapshot_id, &self.published_snapshot_capacity)
    }
    pub fn try_remove_original_result(
        &self,
        original: &std::sync::Arc<PublishedSnapshotReleaseObservation>,
    ) -> Result<Option<bool>, ()> {
        self.handles
            .try_remove_original(original, &self.published_snapshot_capacity)
    }

    pub fn published_release_observation(
        &self,
        snapshot_id: SnapshotId,
    ) -> Option<std::sync::Arc<PublishedSnapshotReleaseObservation>> {
        self.handles.published_release_observation(snapshot_id)
    }

    pub fn published_snapshot_binding(
        &self,
        snapshot_id: SnapshotId,
    ) -> Option<SnapshotHandleBinding> {
        self.handles.published_binding(snapshot_id)
    }

    pub fn published_snapshot_binding_for_version(
        &self,
        version_id: crate::identity::data::VersionId,
    ) -> Option<(SnapshotId, SnapshotHandleBinding)> {
        self.handles.published_binding_for_version(version_id)
    }

    pub fn published_snapshot_closeout(
        &self,
        snapshot_id: SnapshotId,
    ) -> Option<PublishedSnapshotCloseout> {
        self.handles.published_closeout(
            std::sync::Arc::clone(&self.published_snapshot_capacity),
            snapshot_id,
        )
    }

    pub fn cached_visibility_version_count(&self) -> usize {
        self.cache.cached_version_count()
    }

    pub fn protected_visibility_version_count(&self, protect_active_snapshots: bool) -> usize {
        self.cache
            .protected_state_keys(protect_active_snapshots)
            .len()
    }

    pub fn recent_visibility_cache_count(&self) -> usize {
        self.cache.recent_visibility_count()
    }

    pub fn tracked_branch_head_states(
        &self,
    ) -> Vec<crate::visibility::snapshot_states::VisibilitySnapshotStateKey> {
        self.cache.tracked_branch_head_states()
    }

    pub fn track_branch_head_state(
        &self,
        key: &crate::visibility::snapshot_states::VisibilitySnapshotStateKey,
    ) {
        self.cache.track_branch_head_state(key);
    }

    pub fn untrack_branch_head_state(
        &self,
        branch_id: &crate::history::data::BranchId,
    ) -> Option<crate::visibility::snapshot_states::VisibilitySnapshotStateKey> {
        self.cache.untrack_branch_head_state(branch_id)
    }

    pub fn clear_branch_head_residency(
        &self,
        tracked_states: &[crate::visibility::snapshot_states::VisibilitySnapshotStateKey],
    ) {
        self.cache.clear_branch_head_residency(tracked_states);
    }

    pub fn replay_retention_versions(&self) -> Vec<crate::identity::data::VersionId> {
        self.replay_retention
            .read()
            .entries
            .keys()
            .copied()
            .collect()
    }

    pub fn increment_replay_retention(
        &self,
        version_id: crate::identity::data::VersionId,
    ) -> Option<usize> {
        let mut retention = self.replay_retention.write();
        let retained = retention.retained_mut(version_id)?;
        retained.ref_count += 1;
        Some(retained.ref_count)
    }
}

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait VisibilitySubsystemFixturePort: Sized {
    fn snapshot_handle_registry_cost_counters_for_test(
        &self,
    ) -> snapshot_handles::SnapshotHandleRegistryCostCounters;
    fn visibility_cache_cost_counters_for_test(&self) -> cache::VisibilityCacheCostCounters;
}

impl VisibilitySubsystemFixturePort for VisibilitySubsystem {
    fn snapshot_handle_registry_cost_counters_for_test(
        &self,
    ) -> snapshot_handles::SnapshotHandleRegistryCostCounters {
        self.snapshot_handle_registry_cost_counters()
    }

    fn visibility_cache_cost_counters_for_test(&self) -> cache::VisibilityCacheCostCounters {
        self.visibility_cache_cost_counters()
    }
}
