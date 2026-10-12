use super::*;

impl<'runtime> VisibilityReadContext<'runtime> {
    pub(crate) fn authoritative_entity_record_for_id_from_exact_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        entity_id: crate::identity::data::EntityId,
    ) -> Option<EntityReadRecord> {
        ProjectionRecordReader::new(
            self.runtime.current_version_id(),
            &self.runtime.services.instrumentation,
        )
        .authoritative_entity_record_for_id_from_exact_state(state, registry, entity_id)
    }

    pub(crate) fn authoritative_relation_record_for_id_from_exact_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        relation_id: crate::identity::data::RelationId,
    ) -> Option<RelationReadRecord> {
        ProjectionRecordReader::new(
            self.runtime.current_version_id(),
            &self.runtime.services.instrumentation,
        )
        .authoritative_relation_record_for_id_from_exact_state(
            state,
            registry,
            relation_id,
        )
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
        ProjectionRecordReader::new(
            self.runtime.current_version_id(),
            &self.runtime.services.instrumentation,
        )
        .authoritative_entity_record_for_id_at_version_with_registry(
            state, registry, entity_id, version_id,
        )
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
        ProjectionRecordReader::new(
            self.runtime.current_version_id(),
            &self.runtime.services.instrumentation,
        )
        .authoritative_relation_record_for_id_at_version_with_registry(
            state,
            registry,
            relation_id,
            version_id,
        )
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
        ProjectionRecordReader::new(
            self.runtime.current_version_id(),
            &self.runtime.services.instrumentation,
        )
        .visible_entities_of_kind_in_partition_from_state_with_registry(
            state,
            registry,
            partition_id,
            kind_id,
            version_id,
        )
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
        ProjectionRecordReader::new(
            self.runtime.current_version_id(),
            &self.runtime.services.instrumentation,
        )
        .visible_relations_of_kind_in_partition_from_state_with_registry(
            state,
            registry,
            partition_id,
            kind_id,
            version_id,
        )
    }

    pub(crate) fn visible_entity_slots_from_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        version_id: crate::identity::data::VersionId,
    ) -> Vec<(crate::identity::data::PartitionId, DenseSlotBitSet)> {
        ProjectionRecordReader::new(
            self.runtime.current_version_id(),
            &self.runtime.services.instrumentation,
        )
        .visible_entity_slots_from_state(state, version_id)
    }

    pub(crate) fn visible_relation_slots_from_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        version_id: crate::identity::data::VersionId,
    ) -> Vec<(crate::identity::data::PartitionId, DenseSlotBitSet)> {
        ProjectionRecordReader::new(
            self.runtime.current_version_id(),
            &self.runtime.services.instrumentation,
        )
        .visible_relation_slots_from_state(state, version_id)
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
