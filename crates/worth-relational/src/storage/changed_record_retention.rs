//! Reconcile just the published mirror slots using the visibility owner's fence.
use crate::config::data::RetentionBackend;
use crate::identity::data::{PartitionId, VersionId};
use crate::performance::data::runtime_complexity_counters::RuntimeComplexityCounters;
use crate::runtime::state::subsystems::storage::{PartitionEditionCopyLane, StorageSubsystem};
use crate::storage::residency::RelationalRegionHeadView;
use crate::storage::substrate::{EntityRecordKind, RecordKind, RelationRecordKind};
use crate::transactions::data::primitives::RecordRef;
use std::sync::Mutex;

pub(crate) struct ChangedRecordRetention<'a> {
    pub(crate) partitions: &'a StorageSubsystem,
    pub(crate) counters: &'a Mutex<RuntimeComplexityCounters>,
    pub(crate) backend: RetentionBackend,
}

impl ChangedRecordRetention<'_> {
    pub(crate) fn reconcile(&self, changed_records: &[RecordRef], fence: VersionId) {
        for record in changed_records {
            match record {
                RecordRef::Entity(entity) => self.reconcile_slot::<EntityRecordKind>(
                    entity.partition_id,
                    entity.slot_index(),
                    fence,
                    |head| head.entity(*entity).and_then(|slot| slot.retired_at()),
                ),
                RecordRef::Relation(relation) => self.reconcile_slot::<RelationRecordKind>(
                    relation.partition_id,
                    relation.slot_index(),
                    fence,
                    |head| head.relation(*relation).and_then(|slot| slot.retired_at()),
                ),
            }
        }
    }

    fn reconcile_slot<K: RecordKind>(
        &self,
        partition_id: PartitionId,
        slot: usize,
        fence: VersionId,
        retirement: impl FnOnce(RelationalRegionHeadView<'_>) -> Option<VersionId>,
    ) {
        let retired_at = self
            .partitions
            .edit(self.counters, PartitionEditionCopyLane::Ordinary)
            .current_lease(partition_id)
            .and_then(|lease| retirement(lease.head_view()));
        if retired_at.is_some() {
            let mut writer = self
                .partitions
                .edit(self.counters, PartitionEditionCopyLane::Ordinary);
            if let Some(partition) = writer.partition_metadata_mut(partition_id) {
                super::record_retention::refresh_slot::<K>(
                    self.backend,
                    partition,
                    slot,
                    retired_at,
                    fence,
                );
            }
        }
    }
}
