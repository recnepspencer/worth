mod preclaimed_records;
mod record_clone_quotes;
use super::super::read_stop::ReadStop;
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
pub(crate) trait ProjectionRecordSource {
    fn demand_all_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause>;
    fn demand_regions_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause>;

    fn selected_root(&self) -> Option<&std::sync::Arc<crate::branch::RelationalBranchRoot>>;
    fn is_exact_basis(&self) -> bool;
    fn version_id(&self) -> crate::identity::data::VersionId;
    fn selected_schema_authority(
        &self,
    ) -> Option<&crate::branch::RelationalBranchRootSchemaAuthority> {
        self.selected_root().map(|root| root.schema_authority())
    }

    fn with_entity_record<E, T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: EntityId,
        mut before_candidate: impl FnMut(u64, u64) -> Result<(), E>,
        inspect: impl FnOnce(&EntityReadRecord) -> T,
    ) -> Result<Option<T>, ReadStop<E>> {
        self.demand_regions_full(execution, [id.partition_id])
            .map_err(ReadStop::Request)?;
        crate::execution::check_request(execution).map_err(ReadStop::Request)?;
        let bytes = self
            .candidate_entity_heap_for_id(id)
            .and_then(|heap| {
                heap.checked_add(std::mem::size_of::<EntityReadRecord>() as u64)
                    .ok_or(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
            })
            .map_err(ReadStop::Request)?;
        before_candidate(super::super::read_prices::KEYED_RECORD, bytes)
            .map_err(ReadStop::Admission)?;
        crate::execution::check_request(execution).map_err(ReadStop::Request)?;
        let record = preclaimed_records::entity(self, id);
        let actual = record
            .as_ref()
            .map_or(0, super::super::read_footprint::entity)
            .checked_add(std::mem::size_of::<EntityReadRecord>() as u64)
            .ok_or(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
            .map_err(ReadStop::Request)?;
        assert!(
            actual <= bytes,
            "the owner-accounted payload quote bounds the clone payload"
        );
        before_candidate(0, actual).map_err(ReadStop::Admission)?;
        let result = record.as_ref().map(inspect);
        drop(record);
        // The original caller may now restore its enclosing temporary claim.
        before_candidate(0, 0).map_err(ReadStop::Admission)?;
        Ok(result)
    }

    fn try_for_each_entity_record_in<E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: PartitionId,
        kind_id: Option<KindId>,
        mut before_candidate: impl FnMut(u64, u64) -> Result<(), E>,
        mut visit: impl FnMut(&EntityReadRecord) -> Result<(), E>,
    ) -> Result<(), ReadStop<E>> {
        self.demand_regions_full(execution, [partition_id])
            .map_err(ReadStop::Request)?;
        crate::execution::check_request(execution).map_err(ReadStop::Request)?;
        let Some(root) = self.selected_root() else {
            return Ok(());
        };
        let Some(partition) = root.get_partition(partition_id) else {
            return Ok(());
        };
        let mut iterator_bytes = 0;
        let (live, historical) = {
            let mut prepare = |_navigation, bytes| {
                before_candidate(0, bytes).map_err(ReadStop::Admission)?;
                iterator_bytes = bytes;
                Ok(())
            };
            let live = if self.is_exact_basis() {
                Some(
                    partition
                        .entity_arena
                        .live_bitset
                        .try_set_slots(&mut prepare)?,
                )
            } else {
                None
            };
            let historical = if self.is_exact_basis() {
                None
            } else {
                Some(partition.entity_arena.try_occupied_slots(&mut prepare)?)
            };
            (live, historical)
        };
        let slots = live
            .into_iter()
            .flatten()
            .chain(historical.into_iter().flatten());
        for slot in slots {
            let id = EntityId::new(partition_id, slot as u64, 0);
            let bytes =
                self.candidate_entity_heap_for_id(id)
                    .and_then(|heap| {
                        heap.checked_add(std::mem::size_of::<EntityReadRecord>() as u64)
                    .and_then(|row| row.checked_add(iterator_bytes))
                    .ok_or(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
                    })
                    .map_err(ReadStop::Request)?;
            before_candidate(
                super::super::read_prices::EXAMINE_SLOT + super::super::read_prices::KEYED_RECORD,
                bytes,
            )
            .map_err(ReadStop::Admission)?;
            crate::execution::check_request(execution).map_err(ReadStop::Request)?;
            let record = preclaimed_records::entity(self, id);
            let actual = record
                .as_ref()
                .map_or(0, super::super::read_footprint::entity)
                .checked_add(std::mem::size_of::<EntityReadRecord>() as u64)
                .and_then(|row| row.checked_add(iterator_bytes))
                .ok_or(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
                .map_err(ReadStop::Request)?;
            assert!(
                actual <= bytes,
                "the owner-accounted payload quote bounds the clone payload"
            );
            before_candidate(0, actual).map_err(ReadStop::Admission)?;
            if let Some(record) = record
                && kind_id.is_none_or(|kind| record.kind.kind_id == kind)
            {
                visit(&record).map_err(ReadStop::Admission)?;
            }
        }
        // The iterator has dropped before the caller removes its temporary claim.
        before_candidate(0, 0).map_err(ReadStop::Admission)?;
        Ok(())
    }

    fn try_for_each_relation_record_in<E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: PartitionId,
        kind_id: Option<KindId>,
        mut before_candidate: impl FnMut(u64, u64) -> Result<(), E>,
        mut visit: impl FnMut(&RelationReadRecord) -> Result<(), E>,
    ) -> Result<(), ReadStop<E>> {
        self.demand_regions_full(execution, [partition_id])
            .map_err(ReadStop::Request)?;
        crate::execution::check_request(execution).map_err(ReadStop::Request)?;
        let Some(root) = self.selected_root() else {
            return Ok(());
        };
        let Some(partition) = root.get_partition(partition_id) else {
            return Ok(());
        };
        let mut iterator_bytes = 0;
        let (live, historical) = {
            let mut prepare = |_navigation, bytes| {
                before_candidate(0, bytes).map_err(ReadStop::Admission)?;
                iterator_bytes = bytes;
                Ok(())
            };
            let live = if self.is_exact_basis() {
                Some(
                    partition
                        .relation_arena
                        .live_bitset
                        .try_set_slots(&mut prepare)?,
                )
            } else {
                None
            };
            let historical = if self.is_exact_basis() {
                None
            } else {
                Some(partition.relation_arena.try_occupied_slots(&mut prepare)?)
            };
            (live, historical)
        };
        let slots = live
            .into_iter()
            .flatten()
            .chain(historical.into_iter().flatten());
        for slot in slots {
            let id = RelationId::new(partition_id, slot as u64, 0);
            let bytes =
                self.candidate_relation_heap_for_id(id)
                    .and_then(|heap| {
                        heap.checked_add(std::mem::size_of::<RelationReadRecord>() as u64)
                    .and_then(|row| row.checked_add(iterator_bytes))
                    .ok_or(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
                    })
                    .map_err(ReadStop::Request)?;
            before_candidate(
                super::super::read_prices::EXAMINE_SLOT + super::super::read_prices::KEYED_RECORD,
                bytes,
            )
            .map_err(ReadStop::Admission)?;
            crate::execution::check_request(execution).map_err(ReadStop::Request)?;
            let record = preclaimed_records::relation(self, id);
            let actual = record
                .as_ref()
                .map_or(0, super::super::read_footprint::relation)
                .checked_add(std::mem::size_of::<RelationReadRecord>() as u64)
                .and_then(|row| row.checked_add(iterator_bytes))
                .ok_or(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
                .map_err(ReadStop::Request)?;
            assert!(
                actual <= bytes,
                "the owner-accounted payload quote bounds the clone payload"
            );
            before_candidate(0, actual).map_err(ReadStop::Admission)?;
            if let Some(record) = record
                && kind_id.is_none_or(|kind| record.kind.kind_id == kind)
            {
                visit(&record).map_err(ReadStop::Admission)?;
            }
        }
        // The iterator has dropped before the caller removes its temporary claim.
        before_candidate(0, 0).map_err(ReadStop::Admission)?;
        Ok(())
    }

    fn try_for_each_entity_record<E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: Option<KindId>,
        mut before_candidate: impl FnMut(u64, u64) -> Result<(), E>,
        mut visit: impl FnMut(&EntityReadRecord) -> Result<(), E>,
    ) -> Result<(), ReadStop<E>> {
        self.demand_all_full(execution).map_err(ReadStop::Request)?;
        crate::execution::check_request(execution).map_err(ReadStop::Request)?;
        let Some(root) = self.selected_root() else {
            return Ok(());
        };
        root.try_for_each_partition_id(|partition_id| {
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
    ) -> Result<(), ReadStop<E>> {
        self.demand_all_full(execution).map_err(ReadStop::Request)?;
        crate::execution::check_request(execution).map_err(ReadStop::Request)?;
        let Some(root) = self.selected_root() else {
            return Ok(());
        };
        root.try_for_each_partition_id(|partition_id| {
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
    ) -> Result<u64, crate::execution::RelationalExecutionDenialCause> {
        let Some(root) = self.selected_root() else {
            return Ok(0);
        };
        let visibility = if self.is_exact_basis() {
            super::super::reader::KindScanVisibility::Live
        } else {
            super::super::reader::KindScanVisibility::AtVersion(self.version_id())
        };
        super::super::read_work::record_quote::entity_for_id(
            root.as_ref(),
            root.schema_authority().registry(),
            id,
            visibility,
        )
    }

    fn candidate_relation_heap_for_id(
        &self,
        id: RelationId,
    ) -> Result<u64, crate::execution::RelationalExecutionDenialCause> {
        let Some(root) = self.selected_root() else {
            return Ok(0);
        };
        let visibility = if self.is_exact_basis() {
            super::super::reader::KindScanVisibility::Live
        } else {
            super::super::reader::KindScanVisibility::AtVersion(self.version_id())
        };
        super::super::read_work::record_quote::relation_for_id(
            root.as_ref(),
            root.schema_authority().registry(),
            id,
            visibility,
        )
    }

    fn candidate_entity_bytes_for_id(&self, entity_id: EntityId) -> u64 {
        self.selected_root()
            .and_then(|root| root.get_partition(entity_id.partition_id))
            .map_or(0, |partition| {
                self.candidate_entity_bytes(partition, entity_id.slot_index())
            })
    }

    fn candidate_relation_bytes_for_id(&self, relation_id: RelationId) -> u64 {
        self.selected_root()
            .and_then(|root| root.get_partition(relation_id.partition_id))
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
