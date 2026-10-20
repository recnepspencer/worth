//! Canonical unit completions carry query fragments in owner-funded slots.
use super::{
    packet_identity, PacketizedQueryWork, QueryPacketMemory, QueryReadExecutionStop,
    QueryReadPacketDenial, QueryWorkerFragment,
};
use std::mem::size_of;

mod slot;
use slot::FragmentSlot;
use worth_execution::{
    ChargedBytes, ExecutionMemoryReservation, ExecutionResourceLease, MapKernelContext,
    MapKernelFailure, MapOutcome,
};
use worth_foundational::ExecutionReport;

pub(super) fn schedule(
    packets: Vec<packet_identity::NamedQueryPacket>,
    lease: Option<&ExecutionResourceLease<'_>>,
    execute: impl Fn(
            &PacketizedQueryWork,
            usize,
            &QueryPacketMemory,
            &mut MapKernelContext<'_, '_>,
        ) -> Result<QueryWorkerFragment, MapKernelFailure<QueryReadPacketDenial>>
        + Sync,
) -> Result<(Vec<QueryWorkerFragment>, ExecutionReport), QueryReadExecutionStop> {
    let count = packets.len();
    let owner_bytes = (count as u64)
        .checked_mul(
            (size_of::<FragmentSlot<QueryWorkerFragment>>() + size_of::<QueryWorkerFragment>())
                as u64,
        )
        .ok_or(QueryReadExecutionStop::PacketAdmission(
            worth_execution::MapDenial::MemoryOverflow,
        ))?;
    let _owner =
        ExecutionMemoryReservation::reserve_in_scope(lease, owner_bytes).map_err(|denial| {
            QueryReadExecutionStop::ReadRequest(crate::execution::denial::lease_denial(denial))
        })?;
    let mut slots = Vec::new();
    let mut values = Vec::new();
    slots.try_reserve_exact(count).map_err(|_| overflow())?;
    values.try_reserve_exact(count).map_err(|_| overflow())?;
    for _ in 0..count {
        let memory = QueryPacketMemory::tracked(lease).map_err(|denial| {
            QueryReadExecutionStop::ReadRequest(crate::execution::denial::lease_denial(denial))
        })?;
        slots.push(FragmentSlot::new(memory));
    }
    // Packet identity remains sealed in its owner. The map returns unit; the
    // slots and their mandatory tickets retain every fragment until settlement.
    let map = packet_identity::declared_map(packets, 0)?;
    match map.run(lease, |input, context| {
        context.checkpoint(0)?;
        let value = execute(
            &input.work,
            input.ordinal,
            &slots[input.ordinal].memory,
            context,
        )?;
        slots[input.ordinal]
            .memory
            .capture_result(value.additional_charged_bytes())?;
        *slots[input.ordinal]
            .value
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(value);
        Ok(())
    }) {
        MapOutcome::Complete { report, .. } => {
            for mut slot in slots {
                values.push(slot.take());
            }
            Ok((values, report))
        }
        MapOutcome::Stopped {
            boundary,
            reason,
            report,
            ..
        } => Err(QueryReadExecutionStop::PacketStopped {
            boundary,
            reason,
            report,
        }),
    }
}
fn overflow() -> QueryReadExecutionStop {
    QueryReadExecutionStop::PacketAdmission(worth_execution::MapDenial::MemoryOverflow)
}
