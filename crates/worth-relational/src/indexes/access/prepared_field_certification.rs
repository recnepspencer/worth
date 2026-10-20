use super::PreparedEntityFieldLookup;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::indexes::data::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenial,
    BoundedEntityFieldLookupDenialKind, BoundedEntityFieldLookupOutcome,
};
use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::visibility::materialization::read_records::{
    entity_query_locus_value, ProjectionRecordSource,
};
use crate::visibility::materialization::read_records::{read_prices, read_work::record_memory};
use std::cell::{Cell, RefCell};
use worth_execution::{
    ChargedBytes, ExecutionMemoryReservation, ExecutionWorkCeiling, MapKernelFailure, ScopeStop,
};
use worth_foundational::facade::AspectValue;

/// Certification traverses the pinned authoritative storage. Its original
/// request pays each temporary clone/key and vector growth before allocation.
pub(super) fn certify<E: ChargedBytes>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    prepared: &PreparedEntityFieldLookup,
    value: &AspectValue,
    limit: usize,
    outcome: &BoundedEntityFieldLookupOutcome,
    admission: &mut impl FnMut() -> Result<(), E>,
) -> Result<(), Stop<E>> {
    execution
        .in_scope(|lease| {
            let mut held =
                ExecutionMemoryReservation::reserve_in_scope(lease, 0).map_err(|denial| {
                    native(prepared, crate::execution::denial::lease_denial(denial))
                })?;
            let scope = execution.run_measured_result::<(), Stop<E>>(
                ExecutionWorkCeiling::new(execution.work_ceiling()),
                0,
                &mut held,
                |context, memory| {
                    certify_with_memory(
                        execution, prepared, value, limit, outcome, admission, context, memory,
                    )
                    .map_err(MapKernelFailure::Domain)
                },
            );
            match scope.result {
                Ok(()) => Ok(()),
                Err(ScopeStop::Admission(denial)) => Err(native(
                    prepared,
                    crate::execution::denial::lease_denial(denial),
                )),
                Err(ScopeStop::Failure(MapKernelFailure::Domain(error))) => Err(error),
                Err(ScopeStop::Failure(MapKernelFailure::Stop(stop))) => Err(native(
                    prepared,
                    crate::execution::denial::kernel_stop(stop),
                )),
                Err(ScopeStop::Failure(MapKernelFailure::Panic)) => {
                    Err(native(prepared, Cause::WorkerFailed))
                }
                Err(ScopeStop::Failure(MapKernelFailure::ResultCapacityExceeded)) => {
                    Err(native(prepared, Cause::ResultCapacityExceeded))
                }
            }
        })
        .map_err(|denial| native(prepared, denial.into()))?
}

fn certify_with_memory<E: ChargedBytes>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    prepared: &PreparedEntityFieldLookup,
    value: &AspectValue,
    limit: usize,
    outcome: &BoundedEntityFieldLookupOutcome,
    admission: &mut impl FnMut() -> Result<(), E>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
) -> Result<(), Stop<E>> {
    let expected_bytes = AuthoritativeFieldComparisonKey::required_encoded_capacity_bytes(value)
        .ok_or_else(|| native(prepared, Cause::ChargedBytesOverflow))?;
    memory
        .resize(expected_bytes)
        .map_err(|denial| memory_stop(prepared, denial))?;
    let expected = AuthoritativeFieldComparisonKey::from_aspect_value(value);
    let retained = Cell::new(expected.owned_allocation_capacity_bytes());
    let temporary = Cell::new(0_u64);
    let memory = RefCell::new(memory);
    let context = RefCell::new(context);
    let mut storage_ids = Vec::new();
    prepared
        .try_for_each_entity_record(
            execution,
            Some(prepared.entity_kind()),
            |units, bytes| {
                // Certification historically paid the keyed materialization, while
                // ordinary packet streams also paid their separate candidate visit.
                let units = if units == 0 {
                    0
                } else {
                    units - read_prices::EXAMINE_SLOT
                };
                context.borrow_mut().checkpoint(units).map_err(|stop| {
                    native(prepared, crate::execution::denial::kernel_stop(stop))
                })?;
                if units != 0 {
                    super::prepared_lookup_admission::admit(
                        execution,
                        prepared.index_id(),
                        admission,
                    )?;
                }
                let total = add(prepared, retained.get(), bytes)?;
                memory
                    .borrow_mut()
                    .resize(total)
                    .map_err(|denial| memory_stop(prepared, denial))?;
                temporary.set(bytes);
                Ok(())
            },
            |record| {
                let Some(value) = entity_query_locus_value(record, prepared.field_locator()) else {
                    return Ok(());
                };
                let encoded =
                    AuthoritativeFieldComparisonKey::required_encoded_capacity_bytes(value)
                        .ok_or_else(|| native(prepared, Cause::ChargedBytesOverflow))?;
                let live = add(prepared, retained.get(), temporary.get())?;
                let peak = add(prepared, live, encoded)?;
                memory
                    .borrow_mut()
                    .resize(peak)
                    .map_err(|denial| memory_stop(prepared, denial))?;
                let key = AuthoritativeFieldComparisonKey::from_aspect_value(value);
                let matches = key == expected;
                drop(key);
                memory
                    .borrow_mut()
                    .resize(live)
                    .map_err(|denial| memory_stop(prepared, denial))?;
                if matches {
                    let old = record_memory::vector_bytes::<crate::identity::data::EntityId>(
                        storage_ids.capacity(),
                    )
                    .map_err(|cause| native(prepared, cause))?;
                    record_memory::grow(&mut memory.borrow_mut(), &mut storage_ids, 1)
                        .map_err(|cause| native(prepared, cause))?;
                    let new = record_memory::vector_bytes::<crate::identity::data::EntityId>(
                        storage_ids.capacity(),
                    )
                    .map_err(|cause| native(prepared, cause))?;
                    retained.set(add(prepared, retained.get() - old, new)?);
                    storage_ids.push(record.entity_id);
                }
                Ok(())
            },
        )
        .map_err(|stop| match stop {
            crate::visibility::materialization::read_records::read_stop::ReadStop::Admission(
                error,
            ) => error,
            crate::visibility::materialization::read_records::read_stop::ReadStop::Request(
                cause,
            ) => native(prepared, cause),
        })?;
    storage_ids.sort_unstable();
    super::prepared_lookup_admission::admit(execution, prepared.index_id(), admission)?;
    let overflowed = storage_ids.len() > limit;
    storage_ids.truncate(limit);
    if storage_ids == outcome.candidate_entity_ids() && overflowed == outcome.overflowed() {
        Ok(())
    } else {
        Err(Stop::Lookup(
            BoundedEntityFieldLookupDenial::new(
                BoundedEntityFieldLookupDenialKind::StorageParityMismatch,
                prepared.index_id(),
            )
            .with_examined_entry_count(outcome.examined_entry_count()),
        ))
    }
}
fn native<E>(prepared: &PreparedEntityFieldLookup, cause: Cause) -> Stop<E> {
    Stop::Lookup(super::prepared_entity_field_lookup::denied(
        BoundedEntityFieldLookupDenialKind::Execution(cause),
        prepared.index_id(),
    ))
}
fn memory_stop<E>(
    prepared: &PreparedEntityFieldLookup,
    denial: worth_execution::MemoryLimitDenial,
) -> Stop<E> {
    native(
        prepared,
        crate::execution::denial::lease_denial(worth_execution::LeaseDenial::MemoryExhausted(
            denial,
        )),
    )
}
fn add<E>(prepared: &PreparedEntityFieldLookup, held: u64, bytes: u64) -> Result<u64, Stop<E>> {
    held.checked_add(bytes)
        .ok_or_else(|| native(prepared, Cause::ChargedBytesOverflow))
}
