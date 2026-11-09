//! Stream only the selected root's typed kind candidates, admitting Head on demand.
use super::*;
use crate::execution::RelationalExecutionDenialCause as Cause;
fn corrupt(partition: PartitionId) -> Cause {
    Cause::StorageUnavailable(crate::storage::data::RelationalStorageUnavailable {
        subject: crate::storage::data::RelationalStorageSubject::Region(partition),
        cause: crate::storage::data::RelationalStorageUnavailableCause::IntegrityMismatch,
    })
}
pub(super) fn entity<E>(
    source: &(impl ProjectionRecordSource + ?Sized),
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    kind: KindId,
    partition: Option<PartitionId>,
    mut prepare: impl FnMut(u64, u64) -> Result<(), E>,
    mut visit: impl FnMut(&EntityReadRecord) -> Result<(), E>,
) -> Result<(), ReadStop<E>> {
    crate::execution::check_request(execution).map_err(ReadStop::Request)?;
    let Some(root) = source.selected_root() else {
        return Ok(());
    };
    let registry = root.schema_authority().registry();
    for id in root.entity_ids_of_kind(kind, partition) {
        source
            .demand_regions_head(execution, [id.partition_id])
            .map_err(ReadStop::Request)?;
        let head = source
            .admitted_head_region(id.partition_id)
            .ok_or_else(|| ReadStop::Request(corrupt(id.partition_id)))?;
        if head
            .entity_arena()
            .get(&id)
            .is_none_or(|record| record.kind_id() != Some(kind))
        {
            return Err(ReadStop::Request(corrupt(id.partition_id)));
        }
        let bytes = super::super::super::read_work::record_quote::entity_head(
            registry,
            head,
            id.slot_index(),
        )
        .and_then(|bytes| {
            bytes
                .checked_add(std::mem::size_of::<EntityReadRecord>() as u64)
                .ok_or(Cause::ChargedBytesOverflow)
        })
        .map_err(ReadStop::Request)?;
        prepare(
            super::super::super::read_prices::EXAMINE_SLOT
                + super::super::super::read_prices::KEYED_RECORD,
            bytes,
        )
        .map_err(ReadStop::Admission)?;
        crate::execution::check_request(execution).map_err(ReadStop::Request)?;
        let record = head.entity_record(registry, id);
        let actual = record
            .as_ref()
            .map_or(0, super::super::super::read_footprint::entity)
            .checked_add(std::mem::size_of::<EntityReadRecord>() as u64)
            .ok_or(Cause::ChargedBytesOverflow)
            .map_err(ReadStop::Request)?;
        if actual > bytes {
            return Err(ReadStop::Request(Cause::ResultCapacityExceeded));
        }
        prepare(0, actual).map_err(ReadStop::Admission)?;
        if let Some(record) = record {
            visit(&record).map_err(ReadStop::Admission)?;
        }
    }
    prepare(0, 0).map_err(ReadStop::Admission)?;
    Ok(())
}
pub(super) fn relation<E>(
    source: &(impl ProjectionRecordSource + ?Sized),
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    kind: KindId,
    partition: Option<PartitionId>,
    mut prepare: impl FnMut(u64, u64) -> Result<(), E>,
    mut visit: impl FnMut(&RelationReadRecord) -> Result<(), E>,
) -> Result<(), ReadStop<E>> {
    crate::execution::check_request(execution).map_err(ReadStop::Request)?;
    let Some(root) = source.selected_root() else {
        return Ok(());
    };
    let registry = root.schema_authority().registry();
    for id in root.relation_ids_of_kind(kind, partition) {
        source
            .demand_regions_head(execution, [id.partition_id])
            .map_err(ReadStop::Request)?;
        let head = source
            .admitted_head_region(id.partition_id)
            .ok_or_else(|| ReadStop::Request(corrupt(id.partition_id)))?;
        if head
            .relation_arena()
            .get(&id)
            .is_none_or(|record| record.kind_id() != Some(kind))
        {
            return Err(ReadStop::Request(corrupt(id.partition_id)));
        }
        let bytes = super::super::super::read_work::record_quote::relation_head(
            registry,
            head,
            id.slot_index(),
        )
        .and_then(|bytes| {
            bytes
                .checked_add(std::mem::size_of::<RelationReadRecord>() as u64)
                .ok_or(Cause::ChargedBytesOverflow)
        })
        .map_err(ReadStop::Request)?;
        prepare(
            super::super::super::read_prices::EXAMINE_SLOT
                + super::super::super::read_prices::KEYED_RECORD,
            bytes,
        )
        .map_err(ReadStop::Admission)?;
        crate::execution::check_request(execution).map_err(ReadStop::Request)?;
        let record = head.relation_record(registry, id);
        let actual = record
            .as_ref()
            .map_or(0, super::super::super::read_footprint::relation)
            .checked_add(std::mem::size_of::<RelationReadRecord>() as u64)
            .ok_or(Cause::ChargedBytesOverflow)
            .map_err(ReadStop::Request)?;
        if actual > bytes {
            return Err(ReadStop::Request(Cause::ResultCapacityExceeded));
        }
        prepare(0, actual).map_err(ReadStop::Admission)?;
        if let Some(record) = record {
            visit(&record).map_err(ReadStop::Admission)?;
        }
    }
    prepare(0, 0).map_err(ReadStop::Admission)?;
    Ok(())
}
