use crate::identity::data::{RecordId, VersionId};
use crate::storage::substrate::{PinClass, RecordKind};

use super::{partition_of, slot_of, StorageAuthority};

impl StorageAuthority<'_> {
    #[cfg(test)]
    pub(crate) fn clear_named_pins(&self, class: PinClass) {
        let mut writer = self.runtime.edit_partitions();
        for partition in writer.resident_partitions_mut() {
            partition.entity_arena.clear_named_pins(class);
            partition.relation_arena.clear_named_pins(class);
        }
    }

    pub(crate) fn adjust_named_pin<K: RecordKind>(
        &self,
        residency: &crate::storage::residency::RelationalResidencyLease<
            crate::storage::residency::Full,
        >,
        record_id: RecordId<K::Domain>,
        class: PinClass,
        delta: i32,
        retention_fence: VersionId,
    ) -> Result<(), crate::storage::data::RelationalStorageUnavailable> {
        let slot = slot_of::<K>(&record_id);
        let partition_id = partition_of::<K>(&record_id);
        let unavailable = crate::storage::data::RelationalStorageUnavailable {
            subject: crate::storage::data::RelationalStorageSubject::Region(partition_id),
            cause: crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable,
        };
        if residency.full_view().partition().partition_id != partition_id {
            return Err(unavailable);
        }
        let retired_at = {
            let mut writer = self.runtime.edit_partitions();
            let partition = writer.partition_mut(partition_id).ok_or(unavailable)?;
            let arena = K::arena_mut(partition);
            if arena.snapshot_pin_count(slot).is_none() {
                return Ok(());
            }
            arena.adjust_named_pin(slot, class, delta);
            arena.retired_at_for_slot(slot)
        };
        self.refresh_retention_state::<K>(partition_id, slot, retired_at, retention_fence);
        Ok(())
    }
}
