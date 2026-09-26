use super::*;

impl<'runtime> VisibilityReadContext<'runtime> {
    pub(crate) fn authoritative_entity_record_for_id_from_exact_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        entity_id: crate::identity::data::EntityId,
    ) -> Option<EntityReadRecord> {
        let slot = entity_id.slot_index();
        materialize_current_authoritative_entity_record(
            registry,
            state.get_partition(entity_id.partition_id)?,
            entity_id.partition_id,
            slot,
        )
        .filter(|record| {
            entity_id.generation.is_zero() || record.entity_id.generation == entity_id.generation
        })
    }

    pub(crate) fn authoritative_relation_record_for_id_from_exact_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        relation_id: crate::identity::data::RelationId,
    ) -> Option<RelationReadRecord> {
        let slot = relation_id.slot_index();
        materialize_current_authoritative_relation_record(
            registry,
            state.get_partition(relation_id.partition_id)?,
            relation_id.partition_id,
            slot,
        )
        .filter(|record| {
            relation_id.generation.is_zero()
                || record.relation_id.generation == relation_id.generation
        })
    }

    pub(crate) fn authoritative_entity_record_at_version(
        &self,
        entity_id: crate::identity::data::EntityId,
        version_id: crate::identity::data::VersionId,
    ) -> Option<EntityReadRecord> {
        let state = self.runtime.storage_access().current_edition();
        self.authoritative_entity_record_for_id_at_version(&state, entity_id, version_id)
    }

    pub(crate) fn authoritative_relation_record_at_version(
        &self,
        relation_id: crate::identity::data::RelationId,
        version_id: crate::identity::data::VersionId,
    ) -> Option<RelationReadRecord> {
        let state = self.runtime.storage_access().current_edition();
        self.authoritative_relation_record_for_id_at_version(&state, relation_id, version_id)
    }

    pub(crate) fn authoritative_entity_record_for_id_at_version(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        entity_id: crate::identity::data::EntityId,
        version_id: crate::identity::data::VersionId,
    ) -> Option<EntityReadRecord> {
        self.authoritative_entity_record_for_id_at_version_with_registry(
            state,
            &self.runtime.config.schema.registry,
            entity_id,
            version_id,
        )
    }

    pub(crate) fn authoritative_entity_record_for_id_at_version_with_registry(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        entity_id: crate::identity::data::EntityId,
        version_id: crate::identity::data::VersionId,
    ) -> Option<EntityReadRecord> {
        let partition = state.get_partition(entity_id.partition_id)?;
        let slot = entity_id.slot_index();
        if version_id == self.runtime.current_version_id() {
            materialize_current_authoritative_entity_record(
                registry,
                partition,
                entity_id.partition_id,
                slot,
            )
            .filter(|record| {
                entity_id.generation.is_zero()
                    || record.entity_id.generation == entity_id.generation
            })
        } else {
            materialize_authoritative_entity_record_at_version(
                registry,
                partition,
                entity_id.partition_id,
                slot,
                version_id,
            )
            .filter(|record| {
                entity_id.generation.is_zero()
                    || record.entity_id.generation == entity_id.generation
            })
        }
    }

    pub(crate) fn authoritative_relation_record_for_id_at_version(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        relation_id: crate::identity::data::RelationId,
        version_id: crate::identity::data::VersionId,
    ) -> Option<RelationReadRecord> {
        self.authoritative_relation_record_for_id_at_version_with_registry(
            state,
            &self.runtime.config.schema.registry,
            relation_id,
            version_id,
        )
    }

    pub(crate) fn authoritative_relation_record_for_id_at_version_with_registry(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        relation_id: crate::identity::data::RelationId,
        version_id: crate::identity::data::VersionId,
    ) -> Option<RelationReadRecord> {
        let partition = state.get_partition(relation_id.partition_id)?;
        let slot = relation_id.slot_index();
        if version_id == self.runtime.current_version_id() {
            materialize_current_authoritative_relation_record(
                registry,
                partition,
                relation_id.partition_id,
                slot,
            )
            .filter(|record| {
                relation_id.generation.is_zero()
                    || record.relation_id.generation == relation_id.generation
            })
        } else {
            materialize_authoritative_relation_record_at_version(
                registry,
                partition,
                relation_id.partition_id,
                slot,
                version_id,
            )
            .filter(|record| {
                relation_id.generation.is_zero()
                    || record.relation_id.generation == relation_id.generation
            })
        }
    }

    pub(crate) fn visible_entities_of_kind_in_partition_from_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        partition_id: crate::identity::data::PartitionId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
    ) -> Vec<EntityReadRecord> {
        self.visible_entities_of_kind_in_partition_from_state_with_registry(
            state,
            &self.runtime.config.schema.registry,
            partition_id,
            kind_id,
            version_id,
        )
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
        let mut scan = super::truth_kind_scan::EntityKindScan::new(usize::MAX);
        self.scan_entity_kind_in_partition(
            state,
            registry,
            partition_id,
            kind_id,
            version_id,
            &mut scan,
        )
        .expect("an unbounded kind scan cannot exhaust usize::MAX work");
        scan.into_records()
    }

    pub(crate) fn visible_relations_of_kind_in_partition_from_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        partition_id: crate::identity::data::PartitionId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
    ) -> Vec<RelationReadRecord> {
        self.visible_relations_of_kind_in_partition_from_state_with_registry(
            state,
            &self.runtime.config.schema.registry,
            partition_id,
            kind_id,
            version_id,
        )
    }

    pub(crate) fn visible_relations_of_kind_in_partition_from_state_with_registry(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        partition_id: crate::identity::data::PartitionId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
    ) -> Vec<RelationReadRecord> {
        let mut scan = super::truth_relation_kind_scan::RelationKindScan::new(usize::MAX, false);
        self.scan_relation_kind_in_partition(
            state,
            registry,
            partition_id,
            kind_id,
            version_id,
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
            self.runtime,
            state,
            partition_id,
            version_id,
            |runtime, scanned| {
                runtime.services.instrumentation.count(|counters| {
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
            self.runtime,
            state,
            partition_id,
            version_id,
            |runtime, scanned| {
                runtime.services.instrumentation.count(|counters| {
                    counters.visibility_relation_slot_scans += scanned;
                });
            },
        )
    }

    /// Whether a relation is visible, resolved against a pinned edition.
    ///
    /// There is deliberately no acquiring twin of this: a per-candidate
    /// visibility test that acquires its own substrate turns an answer of size
    /// R into R substrate acquisitions against a moving substrate.
    pub(crate) fn relation_visible_in_edition(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        relation_id: crate::identity::data::RelationId,
        version_id: crate::identity::data::VersionId,
    ) -> bool {
        self.authoritative_relation_record_for_id_at_version(state, relation_id, version_id)
            .is_some()
    }
}

pub(super) fn sort_authoritative_relation_records(records: &mut [RelationReadRecord]) {
    records.sort_by_key(|record| {
        (
            record.source.partition_id.0,
            record.source.local_slot.0,
            record.target.partition_id.0,
            record.target.local_slot.0,
            record.relation_id.partition_id.0,
            record.relation_id.local_slot.0,
        )
    });
}
