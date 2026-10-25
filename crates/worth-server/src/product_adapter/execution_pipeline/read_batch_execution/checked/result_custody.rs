use super::{AdapterReadResult, PreparedReadPacket};
use std::sync::Mutex;
use worth_execution::{ExecutionMemoryReservation, LeaseDenial, MapKernelFailure};
use worth_foundational::PartitionIdentity;

pub(super) struct ReservedReadResult {
    pub(super) result: AdapterReadResult,
    pub(super) _memory: ExecutionMemoryReservation,
}

pub(super) type ReadResultSlot = Mutex<Option<Result<ReservedReadResult, LeaseDenial>>>;

pub(super) fn execute_packet(
    packet: &PreparedReadPacket,
    lease: &worth_execution::ExecutionResourceLease<'_>,
) -> Result<ReservedReadResult, worth_execution::ScopeStop<()>> {
    let mut memory = ExecutionMemoryReservation::reserve_in_scope(Some(lease), 0)
        .map_err(worth_execution::ScopeStop::Admission)?;
    let outcome = worth_execution::ExecutionRequest::leased(lease).run_measured_result(
        worth_execution::ExecutionWorkCeiling::new(None),
        0,
        &mut memory,
        |context, _memory| {
            context.checkpoint(0)?;
            let outcome =
                packet
                    .slot
                    .adapter
                    .execute_with_lease(&packet.slot.scheduled, lease, context)?;
            context.checkpoint(0)?;
            Ok::<_, MapKernelFailure<()>>(AdapterReadResult { outcome })
        },
    );
    outcome.result.map(|result| ReservedReadResult {
        result,
        _memory: memory,
    })
}

pub(super) fn result_memory_stop(
    reason: worth_execution::MapStop<()>,
    results: &[ReadResultSlot],
) -> worth_execution::MapStop<()> {
    if let worth_execution::MapStop::Failure {
        identity,
        cause: MapKernelFailure::Domain(()),
    } = &reason
        && let Some(slot) = results.iter().enumerate().find_map(|(ordinal, slot)| {
            (PartitionIdentity::new(ordinal as u64 + 1) == *identity).then_some(slot)
        })
    {
        let mut result = slot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if matches!(*result, Some(Err(_))) {
            let Some(Err(denial)) = result.take() else {
                unreachable!()
            };
            return worth_execution::MapStop::Admission(denial);
        }
    }
    reason
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_result_slot_preserves_the_exact_memory_refusal() {
        let denial = LeaseDenial::MemoryExhausted(
            worth_execution::SerialMemoryBudget::new(Some(0))
                .reserve(1)
                .unwrap_err(),
        );
        let results = [Mutex::new(None), Mutex::new(Some(Err(denial)))];
        let reason = worth_execution::MapStop::Failure {
            identity: PartitionIdentity::new(2),
            cause: MapKernelFailure::Domain(()),
        };
        assert_eq!(
            result_memory_stop(reason, &results),
            worth_execution::MapStop::Admission(denial)
        );
        assert!(results[1].lock().unwrap().is_none());
    }
}
