use std::sync::Arc;

use crate::branch::RelationalBranchRoot;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};

use super::read_record_identity_ordering::relation_identity_order_key;
use super::VisibilityProjectionView;

pub(super) struct ExactProjectionReader<'view> {
    view: &'view VisibilityProjectionView,
    root: &'view Arc<RelationalBranchRoot>,
}

impl<'view> ExactProjectionReader<'view> {
    pub(super) const fn new(
        view: &'view VisibilityProjectionView,
        root: &'view Arc<RelationalBranchRoot>,
    ) -> Self {
        Self { view, root }
    }

    pub(super) fn entity_records(
        &self,
        kind: KindId,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        self.collect_entity_records(Some(kind), charge, memory)
    }
    pub(super) fn all_entity_records(
        &self,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        self.collect_entity_records(None, charge, memory)
    }
    fn collect_entity_records(
        &self,
        kind: Option<KindId>,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        let mut records = Vec::new();
        let (partitions, partition_bytes) =
            super::super::read_work::record_memory::partition_ids(self.root.as_ref(), memory)?;
        for partition in partitions {
            let incoming =
                self.entity_records_in_selected_partition(partition, kind, charge, memory)?;
            super::super::read_work::record_memory::append(memory, &mut records, incoming)?;
        }
        super::super::read_work::record_memory::release(memory, partition_bytes)?;
        debug_assert!(
            super::read_record_identity_ordering::authoritative_entity_records_are_identity_ordered(
                &records
            )
        );
        Ok(records)
    }
    pub(super) fn entity_records_in(
        &self,
        partition: PartitionId,
        kind: KindId,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        self.entity_records_in_selected_partition(partition, Some(kind), charge, memory)
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
            super::super::reader::KindScanVisibility::Live,
            charge,
            memory,
        )
    }
    pub(super) fn entity_record(&self, entity_id: EntityId) -> Option<EntityReadRecord> {
        self.view
            .reader()
            .authoritative_entity_record_for_id_from_exact_state(
                self.root.as_ref(),
                self.root.schema_authority().registry(),
                entity_id,
            )
    }

    pub(super) fn relation_records(
        &self,
        kind: KindId,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<RelationReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        self.collect_relation_records(Some(kind), charge, memory)
    }
    pub(super) fn all_relation_records(
        &self,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<RelationReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        self.collect_relation_records(None, charge, memory)
    }
    fn collect_relation_records(
        &self,
        kind: Option<KindId>,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<RelationReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        let mut records = Vec::new();
        let (partitions, partition_bytes) =
            super::super::read_work::record_memory::partition_ids(self.root.as_ref(), memory)?;
        for partition in partitions {
            let incoming =
                self.relation_records_in_selected_partition(partition, kind, charge, memory)?;
            super::super::read_work::record_memory::append(memory, &mut records, incoming)?;
        }
        super::super::read_work::record_memory::release(memory, partition_bytes)?;
        records.sort_unstable_by_key(relation_identity_order_key);
        debug_assert!(super::read_record_identity_ordering::authoritative_relation_records_are_identity_ordered(&records));
        Ok(records)
    }
    pub(super) fn relation_records_in(
        &self,
        partition: PartitionId,
        kind: KindId,
        charge: &mut dyn FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<Vec<RelationReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        self.relation_records_in_selected_partition(partition, Some(kind), charge, memory)
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
            super::super::reader::KindScanVisibility::Live,
            charge,
            memory,
        )
    }
    pub(super) fn relation_record(&self, relation_id: RelationId) -> Option<RelationReadRecord> {
        self.view
            .reader()
            .authoritative_relation_record_for_id_from_exact_state(
                self.root.as_ref(),
                self.root.schema_authority().registry(),
                relation_id,
            )
    }
}
