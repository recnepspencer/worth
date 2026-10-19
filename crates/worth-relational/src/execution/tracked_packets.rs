//! Variable-shape packet output stays with its owner and live allocation tickets.
use super::{
    PacketAdmissionDenial, PacketBudgetDenial, PacketExecutionStop, PacketKernelContext,
    PacketMemory, ReadOnlyPacket, RequestWorkBudget,
};
use std::{collections::BTreeMap, mem::size_of, sync::Mutex};
use worth_execution::{
    ChargedBytes, ExecutionMap, ExecutionMemoryReservation, ExecutionResourceLease,
    KeylessPartition, MapKernelFailure, MapOutcome,
};

struct Input<P> {
    value: P,
    bytes: u64,
    ordinal: usize,
    memory: PacketMemory,
}
impl<P> ChargedBytes for Input<P> {
    fn additional_charged_bytes(&self) -> u64 {
        self.bytes
    }
}

pub(super) fn execute<P, T, F, C>(
    packets: Vec<ReadOnlyPacket<P>>,
    lease: &ExecutionResourceLease<'_>,
    budget: Option<&RequestWorkBudget>,
    kernel: F,
    charged_bytes: C,
) -> Result<Vec<T>, PacketExecutionStop>
where
    P: Sync,
    T: Send,
    F: Fn(
            &P,
            &mut PacketKernelContext<'_, '_, '_>,
        ) -> Result<T, MapKernelFailure<PacketBudgetDenial>>
        + Sync,
    C: Fn(&T) -> u64 + Sync,
{
    type Slot<T> = Mutex<Option<(T, ExecutionMemoryReservation)>>;
    let count = packets.len();
    let owner_bytes = (count as u64)
        .checked_mul(
            (size_of::<Slot<T>>() + size_of::<T>() + size_of::<ExecutionMemoryReservation>())
                as u64,
        )
        .ok_or(PacketExecutionStop::Admission(
            PacketAdmissionDenial::MemoryOverflow,
        ))?;
    let _owner_memory = lease.reserve_memory(owner_bytes).map_err(admission)?;
    let mut slots = Vec::<Slot<T>>::new();
    let mut values = Vec::new();
    let mut tickets = Vec::new();
    slots.try_reserve_exact(count).map_err(|_| overflow())?;
    values.try_reserve_exact(count).map_err(|_| overflow())?;
    tickets.try_reserve_exact(count).map_err(|_| overflow())?;
    slots.extend((0..count).map(|_| Mutex::new(None)));
    let partitions: BTreeMap<_, _> = packets
        .into_iter()
        .enumerate()
        .map(|(ordinal, packet)| {
            (
                packet.identity,
                KeylessPartition {
                    value: Input {
                        value: packet.value,
                        bytes: packet.input_bytes,
                        ordinal,
                        memory: packet.memory,
                    },
                    // The owner slots and exact live tickets hold output and scratch;
                    // this map's declared result is the unit completion marker.
                    kernel_scratch_bytes: 0,
                    max_result_bytes: 0,
                },
            )
        })
        .collect();
    let batch =
        ExecutionMap::<_, u64>::from_keyless_partitions(partitions).map_err(|_| overflow())?;
    let outcome = super::run_with_remaining_request_work(
        lease,
        budget,
        |child| {
            batch.run(Some(child), |packet, meter| {
                meter.checkpoint(1)?;
                let mut context = PacketKernelContext::tracked(meter, packet.memory, child)?;
                let value = kernel(&packet.value, &mut context)?;
                let ticket = context.capture_result(charged_bytes(&value))?;
                *slots[packet.ordinal]
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some((value, ticket));
                Ok(())
            })
        },
        |outcome| match outcome {
            MapOutcome::Complete { report, .. } | MapOutcome::Stopped { report, .. } => *report,
        },
    );
    match outcome {
        MapOutcome::Complete { .. } => {
            for slot in slots {
                let (value, ticket) = slot
                    .into_inner()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .expect("every canonical packet completed into its own slot");
                values.push(value);
                tickets.push(ticket);
            }
            Ok(values)
        }
        MapOutcome::Stopped {
            boundary, reason, ..
        } => Err(PacketExecutionStop::Execution { boundary, reason }),
    }
}
fn admission(denial: worth_execution::MemoryLimitDenial) -> PacketExecutionStop {
    PacketExecutionStop::Execution {
        boundary: None,
        reason: worth_execution::MapStop::Admission(worth_execution::LeaseDenial::MemoryExhausted(
            denial,
        )),
    }
}
fn overflow() -> PacketExecutionStop {
    PacketExecutionStop::Admission(PacketAdmissionDenial::MemoryOverflow)
}
