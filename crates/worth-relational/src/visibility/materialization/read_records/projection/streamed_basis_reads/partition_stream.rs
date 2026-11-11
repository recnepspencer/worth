//! Stream one declared partition when the caller has not selected a kind directory.
use super::*;
pub(super) fn entity<E>(
    source: &(impl ProjectionRecordSource + ?Sized),
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    partition_id: PartitionId,
    kind_id: Option<KindId>,
    mut before_candidate: impl FnMut(u64, u64) -> Result<(), E>,
    mut visit: impl FnMut(&EntityReadRecord) -> Result<(), E>,
) -> Result<(), ReadStop<E>> {
    if source.is_exact_basis()
        && let Some(kind) = kind_id
    {
        return kind_reads::entity(
            source,
            execution,
            kind,
            Some(partition_id),
            before_candidate,
            visit,
        );
    }
    source
        .demand_regions_full(execution, [partition_id])
        .map_err(ReadStop::Request)?;
    crate::execution::check_request(execution).map_err(ReadStop::Request)?;
    let Some(_root) = source.selected_root() else {
        return Ok(());
    };
    let Some(partition) = source
        .selected_storage()
        .and_then(|storage| storage.get_partition(partition_id))
    else {
        return Ok(());
    };
    let mut iterator_bytes = 0;
    let (live, historical) = {
        let mut prepare = |_navigation, bytes| {
            before_candidate(0, bytes).map_err(ReadStop::Admission)?;
            iterator_bytes = bytes;
            Ok(())
        };
        let live = if source.is_exact_basis() {
            Some(
                partition
                    .entity_arena
                    .live_bitset
                    .try_set_slots(&mut prepare)?,
            )
        } else {
            None
        };
        let historical = if source.is_exact_basis() {
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
        let bytes = source
            .candidate_entity_heap_for_id(id)
            .and_then(|heap| {
                heap.checked_add(std::mem::size_of::<EntityReadRecord>() as u64)
                    .and_then(|row| row.checked_add(iterator_bytes))
                    .ok_or(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
            })
            .map_err(ReadStop::Request)?;
        before_candidate(
            super::super::super::read_prices::EXAMINE_SLOT
                + super::super::super::read_prices::KEYED_RECORD,
            bytes,
        )
        .map_err(ReadStop::Admission)?;
        crate::execution::check_request(execution).map_err(ReadStop::Request)?;
        let record = preclaimed_records::entity(source, id);
        let actual = record
            .as_ref()
            .map_or(0, super::super::super::read_footprint::entity)
            .checked_add(std::mem::size_of::<EntityReadRecord>() as u64)
            .and_then(|row| row.checked_add(iterator_bytes))
            .ok_or(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
            .map_err(ReadStop::Request)?;
        if actual > bytes {
            return Err(ReadStop::Request(
                crate::execution::RelationalExecutionDenialCause::ResultCapacityExceeded,
            ));
        }
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

pub(super) fn relation<E>(
    source: &(impl ProjectionRecordSource + ?Sized),
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    partition_id: PartitionId,
    kind_id: Option<KindId>,
    mut before_candidate: impl FnMut(u64, u64) -> Result<(), E>,
    mut visit: impl FnMut(&RelationReadRecord) -> Result<(), E>,
) -> Result<(), ReadStop<E>> {
    if source.is_exact_basis()
        && let Some(kind) = kind_id
    {
        return kind_reads::relation(
            source,
            execution,
            kind,
            Some(partition_id),
            before_candidate,
            visit,
        );
    }
    source
        .demand_regions_full(execution, [partition_id])
        .map_err(ReadStop::Request)?;
    crate::execution::check_request(execution).map_err(ReadStop::Request)?;
    let Some(_root) = source.selected_root() else {
        return Ok(());
    };
    let Some(partition) = source
        .selected_storage()
        .and_then(|storage| storage.get_partition(partition_id))
    else {
        return Ok(());
    };
    let mut iterator_bytes = 0;
    let (live, historical) = {
        let mut prepare = |_navigation, bytes| {
            before_candidate(0, bytes).map_err(ReadStop::Admission)?;
            iterator_bytes = bytes;
            Ok(())
        };
        let live = if source.is_exact_basis() {
            Some(
                partition
                    .relation_arena
                    .live_bitset
                    .try_set_slots(&mut prepare)?,
            )
        } else {
            None
        };
        let historical = if source.is_exact_basis() {
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
        let bytes = source
            .candidate_relation_heap_for_id(id)
            .and_then(|heap| {
                heap.checked_add(std::mem::size_of::<RelationReadRecord>() as u64)
                    .and_then(|row| row.checked_add(iterator_bytes))
                    .ok_or(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
            })
            .map_err(ReadStop::Request)?;
        before_candidate(
            super::super::super::read_prices::EXAMINE_SLOT
                + super::super::super::read_prices::KEYED_RECORD,
            bytes,
        )
        .map_err(ReadStop::Admission)?;
        crate::execution::check_request(execution).map_err(ReadStop::Request)?;
        let record = preclaimed_records::relation(source, id);
        let actual = record
            .as_ref()
            .map_or(0, super::super::super::read_footprint::relation)
            .checked_add(std::mem::size_of::<RelationReadRecord>() as u64)
            .and_then(|row| row.checked_add(iterator_bytes))
            .ok_or(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
            .map_err(ReadStop::Request)?;
        if actual > bytes {
            return Err(ReadStop::Request(
                crate::execution::RelationalExecutionDenialCause::ResultCapacityExceeded,
            ));
        }
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
