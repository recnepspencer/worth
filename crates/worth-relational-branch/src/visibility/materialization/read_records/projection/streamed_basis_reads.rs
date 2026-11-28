pub mod kind_reads;
pub mod partition_stream;
pub mod preclaimed_records;
pub mod record_clone_quotes;
use worth_relational_contracts::internal_port::execution::request_or::RequestOr;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use crate::storage::overlay::PartitionAccess;
use crate::storage::overlay::PartitionState;
use crate::visibility::materialization::read_records::visibility::{
    visible_metadata, visible_relation_metadata,
};

/// Zero-work callback events prepare or settle the original caller's temporary
/// claim. Record bytes use the established owner-accounted payload measure;
/// that measure does not prove allocator or BTree node structural backing.
pub trait ProjectionRecordSource {
    fn demand_all_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::execution::denial_cause::RelationalExecutionDenialCause>;
    fn demand_regions_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), crate::execution::denial_cause::RelationalExecutionDenialCause>;

    fn demand_regions_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), crate::execution::denial_cause::RelationalExecutionDenialCause>;
    fn admitted_head_region(
        &self,
        id: PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>>;

    fn selected_storage(&self) -> Option<&dyn PartitionAccess>;
    fn is_exact_basis(&self) -> bool;
    fn version_id(&self) -> crate::identity::data::VersionId;
    fn selected_schema_registry(&self) -> Option<&crate::schema::data::RelationalSchemaRegistry>;

    fn try_for_each_selected_partition<E>(
        &self,
        visit: impl FnMut(PartitionId) -> Result<(), E>,
    ) -> Result<(), E>;
    fn selected_entity_ids(
        &self,
        kind: KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<Item = Result<EntityId, crate::storage::data::RelationalStorageUnavailable>> + '_;
    fn selected_relation_ids(
        &self,
        kind: KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<Item = Result<RelationId, crate::storage::data::RelationalStorageUnavailable>> + '_;
    fn retained_head_region(
        &self,
        partition: PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>>;

    fn with_entity_record<E, T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: EntityId,
        mut before_candidate: impl FnMut(u64, u64) -> Result<(), E>,
        inspect: impl FnOnce(&EntityReadRecord) -> T,
    ) -> Result<Option<T>, RequestOr<E>> {
        self.demand_regions_full(execution, [id.partition_id])
            .map_err(RequestOr::Request)?;
        crate::execution::request_custody::check_request(execution).map_err(RequestOr::Request)?;
        let bytes = self
            .candidate_entity_heap_for_id(id)
            .and_then(|heap| {
                heap.checked_add(std::mem::size_of::<EntityReadRecord>() as u64)
                    .ok_or(crate::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow)
            })
            .map_err(RequestOr::Request)?;
        before_candidate(super::super::read_prices::KEYED_RECORD, bytes)
            .map_err(RequestOr::Inner)?;
        crate::execution::request_custody::check_request(execution).map_err(RequestOr::Request)?;
        let record = preclaimed_records::entity(self, id);
        let actual = record
            .as_ref()
            .map_or(0, super::super::read_footprint::entity)
            .checked_add(std::mem::size_of::<EntityReadRecord>() as u64)
            .ok_or(crate::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow)
            .map_err(RequestOr::Request)?;
        if actual > bytes {
            return Err(RequestOr::Request(
                crate::execution::denial_cause::RelationalExecutionDenialCause::ResultCapacityExceeded,
            ));
        }
        before_candidate(0, actual).map_err(RequestOr::Inner)?;
        let result = record.as_ref().map(inspect);
        drop(record);
        // The original caller may now restore its enclosing temporary claim.
        before_candidate(0, 0).map_err(RequestOr::Inner)?;
        Ok(result)
    }

    fn try_for_each_entity_record_in<E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: PartitionId,
        kind_id: Option<KindId>,
        before_candidate: impl FnMut(u64, u64) -> Result<(), E>,
        visit: impl FnMut(&EntityReadRecord) -> Result<(), E>,
    ) -> Result<(), RequestOr<E>> {
        partition_stream::entity(
            self,
            execution,
            partition_id,
            kind_id,
            before_candidate,
            visit,
        )
    }

    fn try_for_each_relation_record_in<E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: PartitionId,
        kind_id: Option<KindId>,
        before_candidate: impl FnMut(u64, u64) -> Result<(), E>,
        visit: impl FnMut(&RelationReadRecord) -> Result<(), E>,
    ) -> Result<(), RequestOr<E>> {
        partition_stream::relation(
            self,
            execution,
            partition_id,
            kind_id,
            before_candidate,
            visit,
        )
    }

    fn try_for_each_entity_record<E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: Option<KindId>,
        mut before_candidate: impl FnMut(u64, u64) -> Result<(), E>,
        mut visit: impl FnMut(&EntityReadRecord) -> Result<(), E>,
    ) -> Result<(), RequestOr<E>> {
        if self.is_exact_basis()
            && let Some(kind) = kind_id
        {
            return kind_reads::entity(self, execution, kind, None, before_candidate, visit);
        }
        self.demand_all_full(execution)
            .map_err(RequestOr::Request)?;
        crate::execution::request_custody::check_request(execution).map_err(RequestOr::Request)?;
        self.try_for_each_selected_partition(|partition_id| {
            self.try_for_each_entity_record_in(
                execution,
                partition_id,
                kind_id,
                &mut before_candidate,
                &mut visit,
            )
        })
    }

    fn try_for_each_relation_record<E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: Option<KindId>,
        mut before_candidate: impl FnMut(u64, u64) -> Result<(), E>,
        mut visit: impl FnMut(&RelationReadRecord) -> Result<(), E>,
    ) -> Result<(), RequestOr<E>> {
        if self.is_exact_basis()
            && let Some(kind) = kind_id
        {
            return kind_reads::relation(self, execution, kind, None, before_candidate, visit);
        }
        self.demand_all_full(execution)
            .map_err(RequestOr::Request)?;
        crate::execution::request_custody::check_request(execution).map_err(RequestOr::Request)?;
        self.try_for_each_selected_partition(|partition_id| {
            self.try_for_each_relation_record_in(
                execution,
                partition_id,
                kind_id,
                &mut before_candidate,
                &mut visit,
            )
        })
    }

    fn candidate_entity_heap_for_id(
        &self,
        id: EntityId,
    ) -> Result<u64, crate::execution::denial_cause::RelationalExecutionDenialCause> {
        use super::super::reader::kind_scan_visibility::KindScanVisibility;
        let Some(registry) = self.selected_schema_registry() else {
            return Ok(0);
        };
        if self.is_exact_basis() {
            return self
                .retained_head_region(id.partition_id)
                .map_or(Ok(0), |head| {
                    super::super::read_work::record_quote::entity_head(
                        registry,
                        head,
                        id.slot_index(),
                    )
                });
        }
        let Some(storage) = self.selected_storage() else {
            return Ok(0);
        };
        super::super::read_work::record_quote::entity_for_id(
            storage,
            registry,
            id,
            KindScanVisibility::AtVersion(self.version_id()),
        )
    }

    fn candidate_relation_heap_for_id(
        &self,
        id: RelationId,
    ) -> Result<u64, crate::execution::denial_cause::RelationalExecutionDenialCause> {
        use super::super::reader::kind_scan_visibility::KindScanVisibility;
        let Some(registry) = self.selected_schema_registry() else {
            return Ok(0);
        };
        if self.is_exact_basis() {
            return self
                .retained_head_region(id.partition_id)
                .map_or(Ok(0), |head| {
                    super::super::read_work::record_quote::relation_head(
                        registry,
                        head,
                        id.slot_index(),
                    )
                });
        }
        let Some(storage) = self.selected_storage() else {
            return Ok(0);
        };
        super::super::read_work::record_quote::relation_for_id(
            storage,
            registry,
            id,
            KindScanVisibility::AtVersion(self.version_id()),
        )
    }

    fn candidate_entity_bytes_for_id(&self, entity_id: EntityId) -> u64 {
        self.selected_storage()
            .and_then(|storage| storage.get_partition(entity_id.partition_id))
            .map_or(0, |partition| {
                self.candidate_entity_bytes(partition, entity_id.slot_index())
            })
    }

    fn candidate_relation_bytes_for_id(&self, relation_id: RelationId) -> u64 {
        self.selected_storage()
            .and_then(|storage| storage.get_partition(relation_id.partition_id))
            .map_or(0, |partition| {
                self.candidate_relation_bytes(partition, relation_id.slot_index())
            })
    }

    fn candidate_entity_bytes(&self, partition: &PartitionState, slot: usize) -> u64 {
        record_clone_quotes::candidate_entity_bytes(self, partition, slot)
    }
    fn candidate_relation_bytes(&self, partition: &PartitionState, slot: usize) -> u64 {
        record_clone_quotes::candidate_relation_bytes(self, partition, slot)
    }
}
