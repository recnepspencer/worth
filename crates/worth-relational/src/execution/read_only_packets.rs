use std::collections::BTreeMap;

use worth_execution::{
    ChargedBytes, ExecutionMap, ExecutionResourceLease, KeylessPartition, MapKernelFailure,
    MapOutcome, MapStop,
};
use worth_foundational::PartitionIdentity;

/// A read-only preparation packet may inspect an immutable snapshot but must
/// publish its result only after the whole canonical batch settles.
#[derive(Debug)]
pub(crate) enum PacketExecutionStop {
    ReadRequest(super::unleased_read_only_packets::UnleasedReadStop),
    Admission(PacketAdmissionDenial),
    Execution {
        boundary: Option<PartitionIdentity>,
        reason: MapStop<PacketBudgetDenial>,
    },
}

/// Refusals representable by Relational's canonical keyless construction.
#[derive(Debug, Clone, Copy)]
pub(crate) enum PacketAdmissionDenial {
    ExpectedIdentitiesNotCanonical,
    MemoryOverflow,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum PacketBudgetDenial {
    ReadRequest(crate::execution::RelationalExecutionDenialCause),
    ScratchCapacityExceeded,
    UncheckedCustomKernel,
}

impl ChargedBytes for PacketBudgetDenial {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

pub(crate) use super::packet_memory::PacketKernelContext;

impl From<PacketExecutionStop> for crate::transactions::data::TransactionCommitError {
    fn from(stop: PacketExecutionStop) -> Self {
        Self::execution(stop.into())
    }
}

struct ChargedPacketResult<T> {
    value: T,
    owned_bytes: u64,
}

pub(crate) struct ReadOnlyPacket<T> {
    pub identity: PartitionIdentity,
    pub value: T,
    pub input_bytes: u64,
    pub memory: super::PacketMemory,
}

struct ChargedPacketInput<T> {
    value: T,
    owned_bytes: u64,
    memory: super::PacketMemory,
}

impl<T> ChargedBytes for ChargedPacketInput<T> {
    fn additional_charged_bytes(&self) -> u64 {
        self.owned_bytes
    }
}

impl<T> ChargedBytes for ChargedPacketResult<T> {
    fn additional_charged_bytes(&self) -> u64 {
        self.owned_bytes
    }
}

/// Input retention, kernel scratch, and result capacity are declared for each
/// stable packet before dispatch. Shared immutable authority is only borrowed.
pub(crate) fn execute_read_only_packets<P, T, F, C>(
    packets: Vec<ReadOnlyPacket<P>>,
    lease: &ExecutionResourceLease<'_>,
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
    execute_read_only_packets_with_budget(packets, lease, None, kernel, charged_bytes)
}

pub(crate) fn execute_read_only_packets_with_budget<P, T, F, C>(
    packets: Vec<ReadOnlyPacket<P>>,
    lease: &ExecutionResourceLease<'_>,
    budget: Option<&super::RequestWorkBudget>,
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
    admit_read_only_packet_order(&packets).map_err(PacketExecutionStop::Admission)?;
    if packets
        .iter()
        .any(|packet| matches!(packet.memory, super::PacketMemory::Tracked))
    {
        return super::tracked_packets::execute(packets, lease, budget, kernel, charged_bytes);
    }
    let partitions: BTreeMap<_, _> = packets
        .into_iter()
        .map(|packet| {
            let super::PacketMemory::Declared { scratch, result } = packet.memory else {
                unreachable!("tracked batches have their own owner carriage")
            };
            (
                packet.identity,
                KeylessPartition {
                    value: ChargedPacketInput {
                        value: packet.value,
                        owned_bytes: packet.input_bytes,
                        memory: packet.memory,
                    },
                    kernel_scratch_bytes: scratch,
                    max_result_bytes: result,
                },
            )
        })
        .collect();
    let batch = ExecutionMap::<_, u64>::from_keyless_partitions(partitions).map_err(
        |worth_execution::MapMemoryOverflow| {
            PacketExecutionStop::Admission(PacketAdmissionDenial::MemoryOverflow)
        },
    )?;
    let outcome = super::run_with_remaining_request_work(
        lease,
        budget,
        |lease| {
            batch.run(Some(lease), |packet, context| {
                context.checkpoint(1)?;
                let mut packet_context = PacketKernelContext::declared(context, packet.memory);
                let value = kernel(&packet.value, &mut packet_context)?;
                Ok(ChargedPacketResult {
                    owned_bytes: charged_bytes(&value),
                    value,
                })
            })
        },
        |outcome| match outcome {
            MapOutcome::Complete { report, .. } | MapOutcome::Stopped { report, .. } => *report,
        },
    );
    match outcome {
        MapOutcome::Complete { values, .. } => {
            Ok(values.into_iter().map(|result| result.value).collect())
        }
        MapOutcome::Stopped {
            boundary, reason, ..
        } => Err(PacketExecutionStop::Execution { boundary, reason }),
    }
}

#[cfg(test)]
mod tests;

pub(crate) fn admit_read_only_packet_order<P>(
    packets: &[ReadOnlyPacket<P>],
) -> Result<(), PacketAdmissionDenial> {
    if packets
        .windows(2)
        .any(|pair| pair[0].identity >= pair[1].identity)
    {
        Err(PacketAdmissionDenial::ExpectedIdentitiesNotCanonical)
    } else {
        Ok(())
    }
}
impl From<super::unleased_read_only_packets::UnleasedReadStop> for PacketExecutionStop {
    fn from(stop: super::unleased_read_only_packets::UnleasedReadStop) -> Self {
        Self::ReadRequest(stop)
    }
}
