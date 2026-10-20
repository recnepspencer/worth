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

    pub(crate) fn visible_entity_slots_from_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        version_id: crate::identity::data::VersionId,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<
        Vec<(crate::identity::data::PartitionId, DenseSlotBitSet)>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        let mut partitions = Vec::new();
        let (partition_ids, partition_bytes) =
            super::super::super::read_work::record_memory::partition_ids(state, memory)?;
        for partition_id in partition_ids {
            if let Some(entity_slots) = self.visible_entity_slots_in_partition_from_state(
                state,
                partition_id,
                version_id,
                charge,
                memory,
            )? {
                super::super::super::read_work::record_memory::prepare_push(
                    memory,
                    &mut partitions,
                    0,
                )?;
                partitions.push((partition_id, entity_slots));
            }
        }
        super::super::super::read_work::record_memory::release(memory, partition_bytes)?;
        Ok(partitions)
    }

    pub(crate) fn visible_entity_slots_in_partition_from_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        partition_id: crate::identity::data::PartitionId,
        version_id: crate::identity::data::VersionId,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Option<DenseSlotBitSet>, crate::execution::RelationalExecutionDenialCause> {
        visible_slots_in_partition_from_state::<crate::storage::substrate::EntityRecordKind>(
            self.current_version,
            state,
            partition_id,
            version_id,
            charge,
            |scanned| {
                self.instrumentation.count(|counters| {
                    counters.visibility_entity_slot_scans += scanned;
                });
            },
            memory,
        )
    }

    pub(crate) fn visible_relation_slots_from_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        version_id: crate::identity::data::VersionId,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<
        Vec<(crate::identity::data::PartitionId, DenseSlotBitSet)>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        let mut partitions = Vec::new();
        let (partition_ids, partition_bytes) =
            super::super::super::read_work::record_memory::partition_ids(state, memory)?;
        for partition_id in partition_ids {
            if let Some(relation_slots) = self.visible_relation_slots_in_partition_from_state(
                state,
                partition_id,
                version_id,
                charge,
                memory,
            )? {
                super::super::super::read_work::record_memory::prepare_push(
                    memory,
                    &mut partitions,
                    0,
                )?;
                partitions.push((partition_id, relation_slots));
            }
        }
        super::super::super::read_work::record_memory::release(memory, partition_bytes)?;
        Ok(partitions)
    }

    pub(crate) fn visible_relation_slots_in_partition_from_state(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        partition_id: crate::identity::data::PartitionId,
        version_id: crate::identity::data::VersionId,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Option<DenseSlotBitSet>, crate::execution::RelationalExecutionDenialCause> {
        visible_relation_slots_in_partition_from_state(
            self.current_version,
            state,
            partition_id,
            version_id,
            charge,
            |scanned| {
                self.instrumentation.count(|counters| {
                    counters.visibility_relation_slot_scans += scanned;
                });
            },
            memory,
        )
    }
}
