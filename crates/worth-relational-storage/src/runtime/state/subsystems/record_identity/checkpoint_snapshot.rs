//! Capture the actual global allocator image without replaying its authority.
use super::*;
use crate::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use crate::durability::data::recovery_stop::RelationalRecoveryStop;
use crate::storage::data::record_identity_image::{
    DurablePendingRecordReservation, DurableRecordGenerationClass,
    DurableRecordGenerationHighWater, DurableRecordIdentityState, DurableRecordReservationOrigin,
    DurableRecordSlotFrontier, DurableReusableRecordSlot,
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
#[cfg(test)]
mod tests;

impl RecordIdentitySubsystem {
    #[cfg(test)]
    pub fn checkpoint_identity(&self) -> DurableRecordIdentityState {
        self.try_checkpoint_identity()
            .expect("test fixed allocator image")
    }

    /// Fallible capture of the global allocator image; returns
    /// `ColdStorageUnavailable` if any paged identity table cannot be read.
    pub fn try_checkpoint_identity(&self) -> Result<DurableRecordIdentityState, DurabilityError> {
        match snapshot(&self.lock(), None, None) {
            Ok(image) => Ok(image),
            Err(RelationalRecoveryStop::Durability(error)) => Err(error),
            Err(stop) => Err(DurabilityError::new(
                RecoveryFailureClass::ColdStorageUnavailable,
                format!("paged record-identity capture stopped: {stop:?}"),
            )),
        }
    }

    /// Materializes a carried paged base for callers that require a resident
    /// `DurableRecordIdentityState` (such as in-memory checkpoint images).
    pub fn snapshot_paged_base_for_request(
        base: &super::paged_base::PagedIdentityBase,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<DurableRecordIdentityState, RelationalRecoveryStop> {
        crate::execution::request_custody::check_request(request)
            .map_err(RelationalRecoveryStop::Request)?;
        let count = base.entry_count().ok_or_else(overflow)?;
        let bytes = DurableRecordIdentityState::control_storage_bytes()
            .and_then(|bytes| {
                bytes.checked_add(array_bytes::<DurablePendingRecordReservation>(
                    usize::try_from(count).ok()?,
                )?)
            })
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(overflow)?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(|stop| RelationalRecoveryStop::Request(stop.into()))?;
        let rows = base.all_rows().map_err(paged_unavailable)?;
        image(
            rows.generations
                .iter()
                .map(|(&key, &generation)| (key, generation)),
            rows.reusable.iter().copied(),
            rows.frontiers.iter().map(|(&key, &next)| (key, next)),
            rows.pending.iter().copied(),
            Some(request),
            memory,
        )
    }

    pub fn checkpoint_identity_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<DurableRecordIdentityState, RelationalRecoveryStop> {
        crate::execution::request_custody::check_request(request)
            .map_err(RelationalRecoveryStop::Request)?;
        let state = self.lock();
        let bytes = DurableRecordIdentityState::control_storage_bytes()
            .and_then(|bytes| {
                bytes.checked_add(array_bytes::<DurableRecordGenerationHighWater>(
                    state.generation_high_water.len(),
                )?)
            })
            .and_then(|bytes| {
                bytes.checked_add(array_bytes::<DurableReusableRecordSlot>(
                    state.reusable.len(),
                )?)
            })
            .and_then(|bytes| {
                bytes.checked_add(array_bytes::<DurableRecordSlotFrontier>(
                    state.next_slots.len(),
                )?)
            })
            .and_then(|bytes| {
                bytes.checked_add(array_bytes::<DurablePendingRecordReservation>(
                    state.pending.len(),
                )?)
            })
            .and_then(|bytes| {
                // Unfaulted checkpoint rows join the image (largest row size).
                let base = match state.paged.as_ref() {
                    Some(base) => base.entry_count()?,
                    None => 0,
                };
                bytes.checked_add(array_bytes::<DurablePendingRecordReservation>(
                    usize::try_from(base).ok()?,
                )?)
            })
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(overflow)?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(|stop| RelationalRecoveryStop::Request(stop.into()))?;
        snapshot(&state, Some(request), memory)
    }
}

fn array_bytes<T>(count: usize) -> Option<usize> {
    Some(std::alloc::Layout::array::<T>(count).ok()?.size())
}
fn overflow() -> RelationalRecoveryStop {
    DurabilityError::new(
        RecoveryFailureClass::CheckpointSizeOverflow,
        "global record-identity image backing exceeds addressable capacity",
    )
    .into()
}

fn snapshot(
    state: &RecordIdentityState,
    request: Option<&ExecutionRequest<'_, '_>>,
    memory: Option<ExecutionMemoryReservation>,
) -> Result<DurableRecordIdentityState, RelationalRecoveryStop> {
    match state.merged_rows().map_err(paged_unavailable)? {
        Some(rows) => image(
            rows.generations
                .iter()
                .map(|(&key, &generation)| (key, generation)),
            rows.reusable.iter().copied(),
            rows.frontiers.iter().map(|(&key, &next)| (key, next)),
            rows.pending.iter().map(|(&key, &pending)| (key, pending)),
            request,
            memory,
        ),
        None => image(
            state
                .generation_high_water
                .iter()
                .map(|(&key, &generation)| (key, generation)),
            state.reusable.iter().map(|(&key, ())| key),
            state.next_slots.iter().map(|(&key, &next)| (key, next)),
            state.pending.iter().map(|(&key, &pending)| (key, pending)),
            request,
            memory,
        ),
    }
}

fn paged_unavailable(
    cause: crate::storage::data::RelationalStorageUnavailableCause,
) -> RelationalRecoveryStop {
    DurabilityError::new(
        RecoveryFailureClass::ColdStorageUnavailable,
        format!("paged record-identity table unavailable: {cause:?}"),
    )
    .into()
}

fn image(
    generations: impl ExactSizeIterator<Item = (RecordSlotKey, u32)>,
    reusable: impl ExactSizeIterator<Item = RecordSlotKey>,
    frontiers: impl ExactSizeIterator<Item = (RecordFrontierKey, usize)>,
    pending: impl ExactSizeIterator<Item = (RecordSlotKey, PendingRecordReservation)>,
    request: Option<&ExecutionRequest<'_, '_>>,
    memory: Option<ExecutionMemoryReservation>,
) -> Result<DurableRecordIdentityState, RelationalRecoveryStop> {
    let generations = copy_rows(
        generations.map(|((class, partition_id, slot), generation)| {
            DurableRecordGenerationHighWater {
                class: durable_class(class),
                partition_id,
                slot: slot as u64,
                generation,
            }
        }),
        request,
    )?;
    let reusable = copy_rows(
        reusable.map(|(class, partition_id, slot)| DurableReusableRecordSlot {
            class: durable_class(class),
            partition_id,
            slot: slot as u64,
        }),
        request,
    )?;
    let frontiers = copy_rows(
        frontiers.map(
            |((class, partition_id), next_slot)| DurableRecordSlotFrontier {
                class: durable_class(class),
                partition_id,
                next_slot: next_slot as u64,
            },
        ),
        request,
    )?;
    let pending = copy_rows(
        pending.map(
            |((class, partition_id, slot), reservation)| DurablePendingRecordReservation {
                class: durable_class(class),
                partition_id,
                slot: slot as u64,
                generation: reservation.generation,
                origin: match reservation.origin {
                    RecordAllocationOrigin::AppendFrontier => {
                        DurableRecordReservationOrigin::AppendFrontier
                    }
                    RecordAllocationOrigin::Reclaimed { prior_generation } => {
                        DurableRecordReservationOrigin::Reclaimed { prior_generation }
                    }
                },
            },
        ),
        request,
    )?;
    Ok(DurableRecordIdentityState::current_with_memory(
        generations,
        reusable,
        frontiers,
        pending,
        memory,
    ))
}

fn copy_rows<T>(
    rows: impl ExactSizeIterator<Item = T>,
    request: Option<&ExecutionRequest<'_, '_>>,
) -> Result<Vec<T>, RelationalRecoveryStop> {
    let mut output = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(request) = request {
            crate::execution::request_custody::check_request(request)
                .map_err(RelationalRecoveryStop::Request)?;
        }
        output.push(row);
    }
    Ok(output)
}

fn durable_class(class: RecordAllocationClass) -> DurableRecordGenerationClass {
    match class {
        RecordAllocationClass::Entity => DurableRecordGenerationClass::Entity,
        RecordAllocationClass::Relation => DurableRecordGenerationClass::Relation,
    }
}
