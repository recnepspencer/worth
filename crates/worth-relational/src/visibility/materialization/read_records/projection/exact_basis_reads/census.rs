//! Whole-result enumeration retained only as a court oracle.
use super::*;
impl ExactProjectionReader<'_> {
    pub(super) fn collect_entity_records(
        &self,
        kind: Option<KindId>,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        let mut records = Vec::new();
        let (partitions, partition_bytes) =
            super::super::super::read_work::record_memory::partition_ids(
                self.root.as_ref(),
                memory,
            )?;
        for partition in partitions {
            let incoming =
                self.entity_records_in_selected_partition(partition, kind, charge, memory)?;
            super::super::super::read_work::record_memory::append(memory, &mut records, incoming)?;
        }
        super::super::super::read_work::record_memory::release(memory, partition_bytes)?;
        debug_assert!(
            /* Proof: Exact projection collects from the immutable owner walk in (partition, slot, generation) order; the owner iterator cannot emit the same live generation twice and the projection preserves its order. */
            super::super::read_record_identity_ordering::authoritative_entity_records_are_identity_ordered(
                &records
            )
        );
        Ok(records)
    }
    fn entity_records_in_selected_partition(
        &self,
        partition: PartitionId,
        kind: Option<KindId>,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        self.view.reader().paid_entities_in_partition(
            self.root.as_ref(),
            self.root.schema_authority().registry(),
            partition,
            kind,
            super::super::super::reader::KindScanVisibility::Live,
            charge,
            memory,
        )
    }
    pub(super) fn collect_relation_records(
        &self,
        kind: Option<KindId>,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<RelationReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        let mut records = Vec::new();
        let (partitions, partition_bytes) =
            super::super::super::read_work::record_memory::partition_ids(
                self.root.as_ref(),
                memory,
            )?;
        for partition in partitions {
            let incoming =
                self.relation_records_in_selected_partition(partition, kind, charge, memory)?;
            super::super::super::read_work::record_memory::append(memory, &mut records, incoming)?;
        }
        super::super::super::read_work::record_memory::release(memory, partition_bytes)?;
        records.sort_unstable_by_key(relation_identity_order_key);
        debug_assert!(/* Proof: Exact projection collects from the immutable owner walk in (partition, slot, generation) order; the owner iterator cannot emit the same live generation twice and the projection preserves its order. */ super::super::read_record_identity_ordering::authoritative_relation_records_are_identity_ordered(&records));
        Ok(records)
    }
    fn relation_records_in_selected_partition(
        &self,
        partition: PartitionId,
        kind: Option<KindId>,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<RelationReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        self.view.reader().paid_relations_in_partition(
            self.root.as_ref(),
            self.root.schema_authority().registry(),
            partition,
            kind,
            super::super::super::reader::KindScanVisibility::Live,
            charge,
            memory,
        )
    }
}
