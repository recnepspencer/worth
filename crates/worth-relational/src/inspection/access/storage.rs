use crate::identity::data::{
    EntityId, LineageId, PartitionId, RelationId, StructuralFingerprint, VersionId,
};
use crate::inspection::data::InspectionScope;
use crate::storage::access::RecordSlotSurface;
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::{EntityRecordKind, RelationRecordKind};

use super::InspectionAccess;

impl<'runtime> InspectionAccess<'runtime> {
    pub(crate) fn current_partition_ids(&self) -> Vec<PartitionId> {
        self.runtime.storage_access().partition_ids()
    }

    pub(crate) fn current_partition_state(
        &self,
        partition_id: PartitionId,
    ) -> Option<std::sync::Arc<PartitionState>> {
        self.runtime.partitions.partition(partition_id)
    }

    pub(crate) fn current_entity_slot_surface(
        &self,
        entity_id: EntityId,
    ) -> Option<RecordSlotSurface> {
        self.runtime
            .storage_access()
            .record_slot_surface::<EntityRecordKind>(entity_id.partition_id, entity_id.slot_index())
    }

    pub(crate) fn current_relation_slot_surface(
        &self,
        relation_id: RelationId,
    ) -> Option<RecordSlotSurface> {
        self.runtime
            .storage_access()
            .record_slot_surface::<RelationRecordKind>(
                relation_id.partition_id,
                relation_id.slot_index(),
            )
    }

    pub(crate) fn inspect_snapshot(
        &self,
        handle: &crate::snapshots::data::SnapshotHandle,
    ) -> Option<crate::snapshots::data::SnapshotInspectionSummary> {
        self.runtime.read_truth().inspect_snapshot(handle)
    }

    pub(crate) fn active_snapshot_count(&self) -> usize {
        self.runtime.visibility.active_snapshot_count()
    }

    pub(crate) fn retention_fence_version(&self) -> VersionId {
        self.runtime
            .visibility
            .retention_fence_version(self.runtime.current_version_id())
    }

    pub(crate) fn auto_reclaim_deleted_records(&self) -> bool {
        self.runtime
            .config
            .storage
            .mvcc
            .auto_reclaim_deleted_records
    }

    pub(crate) fn scoped_authoritative_entity_record(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        scope: &InspectionScope,
        entity_id: EntityId,
    ) -> Option<EntityReadRecord> {
        match scope {
            InspectionScope::Current => self
                .runtime
                .read_truth()
                .authoritative_entity_record_for_id_at_version(
                    &self.runtime.storage_access().current_edition(),
                    entity_id,
                    self.runtime.current_version_id(),
                ),
            InspectionScope::Version(_) | InspectionScope::Snapshot(_) => self
                .read_view_for_scope(execution, scope)?
                .get_entity(entity_id)
                .cloned(),
        }
    }

    pub(crate) fn scoped_authoritative_relation_record(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        scope: &InspectionScope,
        relation_id: RelationId,
    ) -> Option<RelationReadRecord> {
        match scope {
            InspectionScope::Current => self
                .runtime
                .read_truth()
                .authoritative_relation_record_for_id_at_version(
                    &self.runtime.storage_access().current_edition(),
                    relation_id,
                    self.runtime.current_version_id(),
                ),
            InspectionScope::Version(_) | InspectionScope::Snapshot(_) => self
                .read_view_for_scope(execution, scope)?
                .get_relation(relation_id)
                .cloned(),
        }
    }

    pub(crate) fn scoped_relation_ids_for_entity(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        scope: &InspectionScope,
        entity_id: EntityId,
    ) -> Vec<RelationId> {
        match scope {
            InspectionScope::Current => self
                .runtime
                .storage_access()
                .all_relations_for_entity(entity_id, self.runtime.current_version_id()),
            InspectionScope::Version(_) | InspectionScope::Snapshot(_) => self
                .read_view_for_scope(execution, scope)
                .map(|read_view| {
                    read_view
                        .relations()
                        .iter()
                        .filter(|record| record.source == entity_id || record.target == entity_id)
                        .map(|record| record.relation_id)
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    pub(crate) fn scoped_relation_endpoints(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        scope: &InspectionScope,
        relation_id: RelationId,
    ) -> Option<(EntityId, EntityId)> {
        match scope {
            InspectionScope::Current => self
                .runtime
                .partitions
                .partition(relation_id.partition_id)
                .and_then(|partition| {
                    partition
                        .relation_arena
                        .get(&relation_id)
                        .and_then(|slot_view| slot_view.extra().endpoints.clone())
                })
                .map(|endpoints| (endpoints.source, endpoints.target)),
            InspectionScope::Version(_) | InspectionScope::Snapshot(_) => self
                .scoped_authoritative_relation_record(execution, scope, relation_id)
                .map(|record| (record.source, record.target)),
        }
    }

    pub(crate) fn entity_structural_sidecars(
        &self,
        entity_id: EntityId,
    ) -> (Option<LineageId>, Option<StructuralFingerprint>) {
        self.runtime
            .partitions
            .partition(entity_id.partition_id)
            .and_then(|partition| {
                partition.entity_arena.get(&entity_id).map(|slot_view| {
                    let extra = slot_view.extra().clone();
                    (extra.lineage_id, extra.structural_fingerprint)
                })
            })
            .unwrap_or((None, None))
    }
}
