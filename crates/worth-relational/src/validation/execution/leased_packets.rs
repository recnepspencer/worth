//! Canonical invariant dispatch with live allocations on its exact child lease.
use super::{packets::InvariantWorkPacket, InvariantWorkerEnvelope};
use crate::execution::{
    InvariantPacketAllocations, PacketBudgetDenial, PacketExecutionStop, PacketKernelContext,
    ReadOnlyPacket, RequestWorkBudget,
};
use crate::validation::engine::InvariantRuntimeView;
use std::collections::BTreeMap;
use worth_execution::{
    ChargedBytes, ExecutionMap, ExecutionMemoryReservation, ExecutionResourceLease,
    KeylessPartition, MapKernelFailure, MapOutcome,
};

struct Input<'state> {
    packet: InvariantWorkPacket<'state>,
    bytes: u64,
}
impl ChargedBytes for Input<'_> {
    fn additional_charged_bytes(&self) -> u64 {
        self.bytes
    }
}
struct Output {
    envelope: InvariantWorkerEnvelope,
    custody: ExecutionMemoryReservation,
}
impl ChargedBytes for Output {
    // This heap already has live ledger custody; the map admits its fixed slots.
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

pub(crate) struct InvariantEnvelopeCustody {
    _payloads: Vec<ExecutionMemoryReservation>,
    _vectors: ExecutionMemoryReservation,
}
pub(crate) struct LeasedInvariantEnvelopes {
    envelopes: Vec<InvariantWorkerEnvelope>,
    custody: InvariantEnvelopeCustody,
}
impl LeasedInvariantEnvelopes {
    pub(crate) fn into_reduction(self) -> (Vec<InvariantWorkerEnvelope>, InvariantEnvelopeCustody) {
        (self.envelopes, self.custody)
    }
}

pub(crate) fn execute_leased_invariant_packets<'state>(
    packets: Vec<ReadOnlyPacket<InvariantWorkPacket<'state>>>,
    lease: &ExecutionResourceLease<'_>,
    work_budget: Option<&RequestWorkBudget>,
    runtime: &InvariantRuntimeView<'state>,
) -> Result<LeasedInvariantEnvelopes, PacketExecutionStop> {
    let count = packets.len();
    let map = prepare_dispatch(packets)?;
    crate::execution::run_with_remaining_request_work(
        lease,
        work_budget,
        |child| {
            let vectors = match reserve_returned_vectors(child, count) {
                Ok(hold) => hold,
                Err(stop) => {
                    return (
                        Err(stop),
                        worth_foundational::ExecutionReport::new(
                            child.resolved_posture(),
                            0,
                            0,
                            Default::default(),
                        ),
                    )
                }
            };
            match dispatch(&map, child, runtime) {
                MapOutcome::Complete { values, report } => {
                    (Ok(retain_canonical_envelopes(values, vectors)), report)
                }
                MapOutcome::Stopped {
                    boundary,
                    reason,
                    report,
                    ..
                } => (
                    Err(PacketExecutionStop::Execution { boundary, reason }),
                    report,
                ),
            }
        },
        |(_, report)| *report,
    )
    .0
}

fn prepare_dispatch<'state>(
    packets: Vec<ReadOnlyPacket<InvariantWorkPacket<'state>>>,
) -> Result<ExecutionMap<Input<'state>, u64>, PacketExecutionStop> {
    crate::execution::admit_read_only_packet_order(&packets)
        .map_err(PacketExecutionStop::Admission)?;
    let partitions = packets
        .into_iter()
        .map(|packet| {
            (
                packet.identity,
                KeylessPartition {
                    value: Input {
                        packet: packet.value,
                        bytes: packet.input_bytes,
                    },
                    kernel_scratch_bytes: 0,
                    max_result_bytes: 0,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    ExecutionMap::from_keyless_partitions(partitions).map_err(|_| {
        PacketExecutionStop::Admission(crate::execution::PacketAdmissionDenial::MemoryOverflow)
    })
}

fn reserve_returned_vectors(
    lease: &ExecutionResourceLease<'_>,
    count: usize,
) -> Result<ExecutionMemoryReservation, PacketExecutionStop> {
    let bytes = count
        .checked_mul(
            std::mem::size_of::<Output>()
                + std::mem::size_of::<InvariantWorkerEnvelope>()
                + std::mem::size_of::<ExecutionMemoryReservation>(),
        )
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(PacketExecutionStop::Admission(
            crate::execution::PacketAdmissionDenial::MemoryOverflow,
        ))?;
    lease
        .reserve_memory(bytes)
        .map_err(|cause| PacketExecutionStop::Execution {
            boundary: None,
            reason: worth_execution::MapStop::Admission(
                worth_execution::LeaseDenial::MemoryExhausted(cause),
            ),
        })
}

fn dispatch<'state>(
    map: &ExecutionMap<Input<'state>, u64>,
    child: &ExecutionResourceLease<'_>,
    runtime: &InvariantRuntimeView<'state>,
) -> MapOutcome<Output, PacketBudgetDenial> {
    map.run(Some(child), |input, context| {
        context.checkpoint(1)?;
        let mut allocations =
            InvariantPacketAllocations::new(child).map_err(MapKernelFailure::Domain)?;
        let envelope = super::evaluate_invariant_packet_checked(
            runtime,
            &input.packet,
            &mut PacketKernelContext::with_allocations(context, &mut allocations),
            child.status(),
        )?;
        Ok(Output {
            envelope,
            custody: allocations.into_result_custody(),
        })
    })
}

fn retain_canonical_envelopes(
    values: Vec<Output>,
    vectors: ExecutionMemoryReservation,
) -> LeasedInvariantEnvelopes {
    let mut envelopes = Vec::with_capacity(values.len());
    let mut payloads = Vec::with_capacity(values.len());
    for value in values {
        envelopes.push(value.envelope);
        payloads.push(value.custody);
    }
    LeasedInvariantEnvelopes {
        envelopes,
        custody: InvariantEnvelopeCustody {
            _payloads: payloads,
            _vectors: vectors,
        },
    }
}
