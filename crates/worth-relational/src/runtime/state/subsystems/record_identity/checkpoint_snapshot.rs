//! Capture the actual global allocator image without replaying its authority.
use super::*;
use crate::durability::data::{
    DurabilityError, DurablePendingRecordReservation, DurableRecordGenerationClass,
    DurableRecordGenerationHighWater, DurableRecordIdentityState, DurableRecordReservationOrigin,
    DurableRecordSlotFrontier, DurableReusableRecordSlot, RecoveryFailureClass,
    RelationalRecoveryStop,
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
#[cfg(test)]
mod tests;

impl RecordIdentitySubsystem {
    /// Ordinary construction retains its existing unqualified allocation contract.
    pub(crate) fn checkpoint_identity(&self) -> DurableRecordIdentityState {
        snapshot(&self.lock(), None, None)
            .expect(/* Proof: Without a request the private fixed-row copy has no refusal branch; every source slot already fits Native's public u64 identity. */ "ordinary fixed allocator image")
    }

    pub(crate) fn checkpoint_identity_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<DurableRecordIdentityState, RelationalRecoveryStop> {
        crate::execution::check_request(request).map_err(RelationalRecoveryStop::Request)?;
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
    let generations =
        copy_rows(
            state.generation_high_water.iter().map(
                |(&(class, partition_id, slot), &generation)| DurableRecordGenerationHighWater {
                    class: durable_class(class),
                    partition_id,
                    slot: slot as u64,
                    generation,
                },
            ),
            request,
        )?;
    let reusable = copy_rows(
        state.reusable.iter().map(
            |(&(class, partition_id, slot), ())| DurableReusableRecordSlot {
                class: durable_class(class),
                partition_id,
                slot: slot as u64,
            },
        ),
        request,
    )?;
    let frontiers = copy_rows(
        state
            .next_slots
            .iter()
            .map(
                |(&(class, partition_id), &next_slot)| DurableRecordSlotFrontier {
                    class: durable_class(class),
                    partition_id,
                    next_slot: next_slot as u64,
                },
            ),
        request,
    )?;
    let pending = copy_rows(
        state
            .pending
            .iter()
            .map(
                |(&(class, partition_id, slot), reservation)| DurablePendingRecordReservation {
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
            crate::execution::check_request(request).map_err(RelationalRecoveryStop::Request)?;
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
