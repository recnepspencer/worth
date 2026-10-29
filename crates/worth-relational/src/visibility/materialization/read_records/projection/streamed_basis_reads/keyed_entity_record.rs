//! Charge and inspect one keyed entity after the caller grants the quote.

use super::ProjectionRecordSource;
use crate::identity::data::EntityId;
use crate::storage::data::EntityReadRecord;

pub(super) enum KeyedEntityInspectionStop {
    Request(crate::execution::RelationalExecutionDenialCause),
    Admission,
}

#[inline(never)]
pub(super) fn inspect(
    source: &(impl ProjectionRecordSource + ?Sized),
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    id: EntityId,
    before_candidate: &mut dyn FnMut(u64, u64) -> Result<(), ()>,
    inspect: &mut dyn FnMut(&EntityReadRecord),
) -> Result<bool, KeyedEntityInspectionStop> {
    source
        .demand_regions_full(execution, [id.partition_id])
        .map_err(KeyedEntityInspectionStop::Request)?;
    crate::execution::check_request(execution).map_err(KeyedEntityInspectionStop::Request)?;
    let bytes = source
        .candidate_entity_heap_for_id(id)
        .and_then(|heap| {
            heap.checked_add(std::mem::size_of::<EntityReadRecord>() as u64)
                .ok_or(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
        })
        .map_err(KeyedEntityInspectionStop::Request)?;
    before_candidate(super::super::super::read_prices::KEYED_RECORD, bytes)
        .map_err(|()| KeyedEntityInspectionStop::Admission)?;
    crate::execution::check_request(execution).map_err(KeyedEntityInspectionStop::Request)?;
    let record = super::preclaimed_records::entity(source, id);
    let actual = record
        .as_ref()
        .map_or(0, super::super::super::read_footprint::entity)
        .checked_add(std::mem::size_of::<EntityReadRecord>() as u64)
        .ok_or(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
        .map_err(KeyedEntityInspectionStop::Request)?;
    if actual > bytes {
        return Err(KeyedEntityInspectionStop::Request(
            crate::execution::RelationalExecutionDenialCause::ResultCapacityExceeded,
        ));
    }
    before_candidate(0, actual).map_err(|()| KeyedEntityInspectionStop::Admission)?;
    let produced = record.is_some();
    if let Some(record) = record.as_ref() {
        inspect(record);
    }
    drop(record);
    // The original caller may now restore its enclosing temporary claim.
    before_candidate(0, 0).map_err(|()| KeyedEntityInspectionStop::Admission)?;
    Ok(produced)
}
