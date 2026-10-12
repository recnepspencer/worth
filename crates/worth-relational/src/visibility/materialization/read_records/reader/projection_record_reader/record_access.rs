//! Read records and visibility slots using carried version metadata.
use super::super::*;

impl ProjectionRecordReader<'_> {
    pub(crate) fn authoritative_entity_record_for_id_from_exact_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        entity_id: crate::identity::data::EntityId,
    ) -> Option<EntityReadRecord> {
        super::super::record_identity_materialization::authoritative_entity_record_for_id_from_exact_state(
            state, registry, entity_id,
        )
    }

    pub(crate) fn authoritative_relation_record_for_id_from_exact_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        relation_id: crate::identity::data::RelationId,
    ) -> Option<RelationReadRecord> {
        super::super::record_identity_materialization::authoritative_relation_record_for_id_from_exact_state(state, registry, relation_id)
    }

    pub(crate) fn authoritative_entity_record_for_id_at_version_with_registry(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        entity_id: crate::identity::data::EntityId,
        version_id: crate::identity::data::VersionId,
    ) -> Option<EntityReadRecord> {
        super::super::record_identity_materialization::authoritative_entity_record_for_id_at_version_with_registry(state, registry, entity_id, version_id, self.current_version)
    }

    pub(crate) fn authoritative_relation_record_for_id_at_version_with_registry(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        relation_id: crate::identity::data::RelationId,
        version_id: crate::identity::data::VersionId,
    ) -> Option<RelationReadRecord> {
        super::super::record_identity_materialization::authoritative_relation_record_for_id_at_version_with_registry(state, registry, relation_id, version_id, self.current_version)
    }

    pub(crate) fn visible_entities_of_kind_in_partition_from_state_with_registry(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        partition_id: crate::identity::data::PartitionId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
    ) -> Vec<EntityReadRecord> {
        // One scan implementation. An unbounded kind read is the bounded one
        // with a budget nothing can reach, so the two can never drift apart in
        // what they consider visible.
        let mut scan = super::super::truth_kind_scan::EntityKindScan::new(usize::MAX);
        self.scan_entity_kind_in_partition(
            state,
            registry,
            partition_id,
            kind_id,
            super::super::KindScanVisibility::for_selected_version(
                self.current_version,
                version_id,
            ),
            &mut scan,
        )
        .expect("an unbounded kind scan cannot exhaust usize::MAX work");
        scan.into_records()
    }

    pub(crate) fn visible_relations_of_kind_in_partition_from_state_with_registry(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        partition_id: crate::identity::data::PartitionId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
    ) -> Vec<RelationReadRecord> {
        let mut scan =
            super::super::truth_relation_kind_scan::RelationKindScan::new(usize::MAX, false);
        self.scan_relation_kind_in_partition(
            state,
            registry,
            partition_id,
            kind_id,
            super::super::KindScanVisibility::for_selected_version(
                self.current_version,
                version_id,
            ),
            &mut scan,
        )
        .expect("an unbounded kind scan cannot exhaust usize::MAX work");
        scan.into_records()
    }

    pub(crate) fn visible_entity_slots_from_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        version_id: crate::identity::data::VersionId,
    ) -> Vec<(crate::identity::data::PartitionId, DenseSlotBitSet)> {
        let mut partitions = Vec::new();
        for partition_id in state.partition_ids() {
            if let Some(entity_slots) =
                self.visible_entity_slots_in_partition_from_state(state, partition_id, version_id)
            {
                partitions.push((partition_id, entity_slots));
            }
        }
        partitions
    }

    pub(crate) fn visible_entity_slots_in_partition_from_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        partition_id: crate::identity::data::PartitionId,
        version_id: crate::identity::data::VersionId,
    ) -> Option<DenseSlotBitSet> {
        visible_slots_in_partition_from_state::<crate::storage::substrate::EntityRecordKind>(
            self.current_version,
            state,
            partition_id,
            version_id,
            |scanned| {
                self.instrumentation.count(|counters| {
                    counters.visibility_entity_slot_scans += scanned;
                });
            },
        )
    }

    pub(crate) fn visible_relation_slots_from_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        version_id: crate::identity::data::VersionId,
    ) -> Vec<(crate::identity::data::PartitionId, DenseSlotBitSet)> {
        let mut partitions = Vec::new();
        for partition_id in state.partition_ids() {
            if let Some(relation_slots) =
                self.visible_relation_slots_in_partition_from_state(state, partition_id, version_id)
            {
                partitions.push((partition_id, relation_slots));
            }
        }
        partitions
    }

    pub(crate) fn visible_relation_slots_in_partition_from_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        partition_id: crate::identity::data::PartitionId,
        version_id: crate::identity::data::VersionId,
    ) -> Option<DenseSlotBitSet> {
        visible_relation_slots_in_partition_from_state(
            self.current_version,
            state,
            partition_id,
            version_id,
            |scanned| {
                self.instrumentation.count(|counters| {
                    counters.visibility_relation_slot_scans += scanned;
                });
            },
        )
    }
}
