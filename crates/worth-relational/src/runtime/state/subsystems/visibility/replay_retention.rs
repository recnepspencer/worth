use std::collections::BTreeMap;

use crate::identity::data::VersionId;

#[derive(Debug, Default)]
pub(crate) struct ReplayRetentionIndex {
    pub(crate) entries: BTreeMap<VersionId, ReplayRetentionState>,
}

impl ReplayRetentionIndex {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn fork(&self) -> Self {
        Self::new()
    }

    pub(crate) fn retained_mut(
        &mut self,
        version_id: VersionId,
    ) -> Option<&mut ReplayRetentionState> {
        self.entries.get_mut(&version_id)
    }

    pub(crate) fn insert_retained(&mut self, version_id: VersionId, state: ReplayRetentionState) {
        self.entries.insert(version_id, state);
    }

    pub(crate) fn take_retained(&mut self, version_id: VersionId) -> Option<ReplayRetentionState> {
        self.entries.remove(&version_id)
    }

    pub(crate) fn versions(&self) -> impl Iterator<Item = VersionId> + '_ {
        self.entries.keys().copied()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ReplayRetentionState {
    pub(crate) ref_count: usize,
    pub(crate) pinned_state: crate::visibility::snapshot_states::SnapshotState,
    pub(crate) pin_leases: BTreeMap<
        crate::identity::data::PartitionId,
        crate::storage::residency::RelationalResidencyLease<crate::storage::residency::Full>,
    >,
}
