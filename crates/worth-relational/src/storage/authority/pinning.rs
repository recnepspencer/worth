use crate::identity::data::VersionId;
use crate::storage::residency::{Full, RelationalResidencyLease};
use crate::storage::substrate::{PinClass, RecordKind};

use super::StorageAuthority;

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
        resident: &RelationalResidencyLease<Full>,
        slot: usize,
        class: PinClass,
        delta: i32,
        retention_fence: VersionId,
    ) {
        let partition_id = resident.full_view().partition().partition_id;
        let mut writer = self.runtime.edit_partitions();
        let Some(partition) = writer.partition_mut(partition_id) else {
            return;
        };
        let arena = K::arena_mut(partition);
        if arena.snapshot_pin_count(slot).is_none() {
            return;
        }
        arena.adjust_named_pin(slot, class, delta);
        let retired_at = arena.retired_at_for_slot(slot);
        self.refresh_retention_state::<K>(partition, slot, retired_at, retention_fence);
    }
}
