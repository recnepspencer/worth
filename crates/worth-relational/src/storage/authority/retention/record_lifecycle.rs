use crate::identity::data::VersionId;
use crate::storage::data::RecordLifecycleState;
use crate::storage::substrate::RecordKind;

use super::super::StorageAuthority;

impl StorageAuthority<'_> {
    pub(crate) fn refresh_retention_state<K: RecordKind>(
        &self,
        partition: &mut crate::storage::overlay::PartitionState,
        slot: usize,
        retired_at: Option<VersionId>,
        retention_fence: VersionId,
    ) {
        crate::storage::record_retention::refresh_slot::<K>(
            self.runtime.config.storage.retention.backend,
            partition,
            slot,
            retired_at,
            retention_fence,
        )
    }

    pub(in crate::storage::authority) fn retention_lifecycle<K: RecordKind>(
        &self,
        arena: &crate::storage::substrate::RecordArena<K>,
        slot: usize,
        retired_at: Option<VersionId>,
        retention_fence: VersionId,
    ) -> RecordLifecycleState {
        crate::storage::record_retention::retention_lifecycle::<K>(
            self.runtime.config.storage.retention.backend,
            arena,
            slot,
            retired_at,
            retention_fence,
        )
    }

    pub(crate) fn reclaim_record_if_reclaimable<K: RecordKind>(
        &self,
        class: crate::history::data::RecordAllocationClass,
        partition: &mut crate::storage::overlay::PartitionState,
        slot: usize,
    ) -> Option<crate::runtime::ReclaimedRecordSlot> {
        let partition_id = partition.partition_id;
        let arena = K::arena_mut(partition);
        if arena.get_slot(slot)?.lifecycle() != RecordLifecycleState::Reclaimable {
            return None;
        }
        arena.set_lifecycle_for_slot(slot, RecordLifecycleState::Reusable);
        arena.reset_slot(slot)?;
        Some(crate::runtime::ReclaimedRecordSlot::new(
            class,
            partition_id,
            slot,
        ))
    }
}
