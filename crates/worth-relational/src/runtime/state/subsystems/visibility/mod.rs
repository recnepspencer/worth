mod cache;
mod replay_retention;
mod snapshot_handle_binding;
mod snapshot_handles;

pub(crate) use cache::{VisibilityCache, VisibilityResidency};
pub(crate) use replay_retention::{ReplayRetentionIndex, ReplayRetentionState};
pub(crate) use snapshot_handle_binding::SnapshotHandleBinding;
pub(crate) use snapshot_handles::{
    PublishedSnapshotCapacityOwner, PublishedSnapshotCloseout, PublishedSnapshotSlotReservation,
    SnapshotHandles,
};

use crate::runtime::state::subsystems::RuntimeSubsystem;
use crate::runtime::RelationalRuntimeConfig;
use crate::snapshots::data::SnapshotId;

#[derive(Debug)]
pub(crate) struct VisibilitySubsystem {
    pub(crate) handles: SnapshotHandles,
    pub(crate) cache: std::sync::Arc<VisibilityCache>,
    /// Replay-pinned versions, owned behind their own lock so replay retention
    /// is taken and released without exclusive access to the runtime.
    replay_retention: super::RuntimeOwnedState<ReplayRetentionIndex>,
    published_snapshot_capacity: std::sync::Arc<PublishedSnapshotCapacityOwner>,
}

impl VisibilitySubsystem {
    pub(crate) fn snapshot_identity_binding(&self) -> std::sync::Arc<std::sync::atomic::AtomicU64> {
        self.handles.snapshot_identity_binding()
    }

    pub(crate) fn published_snapshot_capacity_binding(
        &self,
    ) -> std::sync::Arc<PublishedSnapshotCapacityOwner> {
        std::sync::Arc::clone(&self.published_snapshot_capacity)
    }
    fn build_from_config(config: &RelationalRuntimeConfig) -> Self {
        Self {
            handles: SnapshotHandles::new(),
            cache: std::sync::Arc::new(VisibilityCache::new(config)),
            replay_retention: super::RuntimeOwnedState::new(ReplayRetentionIndex::new()),
            published_snapshot_capacity: PublishedSnapshotCapacityOwner::new(
                config.publication.policy.max_published_snapshot_handles,
            ),
        }
    }
}

impl RuntimeSubsystem for VisibilitySubsystem {
    type Config = RelationalRuntimeConfig;

    fn new(config: &Self::Config) -> Self {
        Self::build_from_config(config)
    }

    fn fork(&self) -> Self {
        Self {
            handles: self.handles.fork(),
            cache: std::sync::Arc::new(self.cache.fork()),
            replay_retention: super::RuntimeOwnedState::new(self.replay_retention.read().fork()),
            published_snapshot_capacity: PublishedSnapshotCapacityOwner::new(
                self.published_snapshot_capacity.maximum_handles(),
            ),
        }
    }
}

impl VisibilitySubsystem {
    pub(crate) fn cache_binding(&self) -> std::sync::Arc<VisibilityCache> {
        std::sync::Arc::clone(&self.cache)
    }

    pub(crate) fn active_snapshot_count(&self) -> usize {
        self.handles.active_count()
    }

    #[cfg(test)]
    pub(crate) fn snapshot_handle_registry_cost_counters(
        &self,
    ) -> snapshot_handles::SnapshotHandleRegistryCostCounters {
        self.handles.registry_cost_counters()
    }

    #[cfg(test)]
    pub(crate) fn visibility_cache_cost_counters(&self) -> cache::VisibilityCacheCostCounters {
        self.cache.cost_counters()
    }

    pub(crate) fn published_snapshot_handle_count(&self) -> usize {
        self.handles.published_count()
    }

    pub(crate) fn allocate_snapshot_id(&self) -> Option<SnapshotId> {
        self.handles.next_snapshot_id()
    }

    pub(crate) fn insert_active_handle(
        &self,
        snapshot_id: SnapshotId,
        binding: SnapshotHandleBinding,
    ) {
        self.handles.insert_active(snapshot_id, binding);
    }

    pub(crate) fn remove_active_handle(
        &self,
        snapshot_id: SnapshotId,
    ) -> Option<SnapshotHandleBinding> {
        self.handles.remove_active(snapshot_id)
    }

    pub(crate) fn active_handle_binding(
        &self,
        snapshot_id: SnapshotId,
    ) -> Option<SnapshotHandleBinding> {
        self.handles.active_binding(snapshot_id)
    }

    pub(crate) fn is_known_snapshot(&self, snapshot_id: SnapshotId) -> bool {
        self.handles.is_known_snapshot(snapshot_id)
    }

    pub(crate) fn active_versions(&self) -> Vec<crate::identity::data::VersionId> {
        self.handles.active_versions()
    }

    pub(crate) fn retention_fence_version(
        &self,
        published_version: crate::identity::data::VersionId,
    ) -> crate::identity::data::VersionId {
        self.active_versions()
            .into_iter()
            .chain(self.replay_retention_versions())
            .min()
            .unwrap_or(published_version)
    }

    pub(crate) fn historical_reconstruction_fence_version(
        &self,
        published_version: crate::identity::data::VersionId,
    ) -> crate::identity::data::VersionId {
        let retention_fence = self.retention_fence_version(published_version);
        self.handles
            .published_versions()
            .into_iter()
            .min()
            .map_or(retention_fence, |version| version.min(retention_fence))
    }

    pub(crate) fn insert_published_handle(
        &self,
        snapshot_id: SnapshotId,
        binding: SnapshotHandleBinding,
    ) {
        self.handles.insert_published(snapshot_id, binding);
    }

    pub(crate) fn remove_published_handle(
        &self,
        snapshot_id: SnapshotId,
    ) -> Option<SnapshotHandleBinding> {
        let removed = self.handles.remove_published(snapshot_id);
        if removed.is_some() {
            self.published_snapshot_capacity.release();
        }
        removed
    }

    pub(crate) fn published_snapshot_binding(
        &self,
        snapshot_id: SnapshotId,
    ) -> Option<SnapshotHandleBinding> {
        self.handles.published_binding(snapshot_id)
    }

    pub(crate) fn published_snapshot_binding_for_version(
        &self,
        version_id: crate::identity::data::VersionId,
    ) -> Option<(SnapshotId, SnapshotHandleBinding)> {
        self.handles.published_binding_for_version(version_id)
    }

    pub(crate) fn published_snapshot_closeout(
        &self,
        snapshot_id: SnapshotId,
    ) -> Option<PublishedSnapshotCloseout> {
        self.handles.published_closeout(
            std::sync::Arc::clone(&self.published_snapshot_capacity),
            snapshot_id,
        )
    }

    pub(crate) fn cached_visibility_version_count(&self) -> usize {
        self.cache.cached_version_count()
    }

    pub(crate) fn protected_visibility_version_count(
        &self,
        protect_active_snapshots: bool,
    ) -> usize {
        self.cache
            .protected_state_keys(protect_active_snapshots)
            .len()
    }

    pub(crate) fn recent_visibility_cache_count(&self) -> usize {
        self.cache.recent_visibility_count()
    }

    pub(crate) fn tracked_branch_head_states(
        &self,
    ) -> Vec<crate::visibility::snapshot_states::VisibilitySnapshotStateKey> {
        self.cache.tracked_branch_head_states()
    }

    pub(crate) fn track_branch_head_state(
        &self,
        key: &crate::visibility::snapshot_states::VisibilitySnapshotStateKey,
    ) {
        self.cache.track_branch_head_state(key);
    }

    pub(crate) fn untrack_branch_head_state(
        &self,
        branch_id: &crate::history::data::BranchId,
    ) -> Option<crate::visibility::snapshot_states::VisibilitySnapshotStateKey> {
        self.cache.untrack_branch_head_state(branch_id)
    }

    pub(crate) fn clear_branch_head_residency(
        &self,
        tracked_states: &[crate::visibility::snapshot_states::VisibilitySnapshotStateKey],
    ) {
        self.cache.clear_branch_head_residency(tracked_states);
    }

    /// Every version a replay pin currently retains, collected so the fence
    /// computation never scans the index while holding its lock.
    pub(crate) fn replay_retention_versions(&self) -> Vec<crate::identity::data::VersionId> {
        self.replay_retention.read().versions().collect()
    }

    pub(crate) fn increment_replay_retention(
        &self,
        version_id: crate::identity::data::VersionId,
    ) -> Option<usize> {
        let mut retention = self.replay_retention.write();
        let retained = retention.retained_mut(version_id)?;
        retained.ref_count += 1;
        Some(retained.ref_count)
    }

    pub(crate) fn insert_replay_retention(
        &self,
        version_id: crate::identity::data::VersionId,
        state: ReplayRetentionState,
    ) {
        self.replay_retention
            .write()
            .insert_retained(version_id, state);
    }

    pub(crate) fn take_replay_retention(
        &self,
        version_id: crate::identity::data::VersionId,
    ) -> Option<ReplayRetentionState> {
        self.replay_retention.write().take_retained(version_id)
    }

    pub(crate) fn restore_replay_retention(
        &self,
        version_id: crate::identity::data::VersionId,
        state: ReplayRetentionState,
    ) {
        self.replay_retention
            .write()
            .insert_retained(version_id, state);
    }
}
