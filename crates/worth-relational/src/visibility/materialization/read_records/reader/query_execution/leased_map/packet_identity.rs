//! Owns each packet's name and its binding to the canonical partition set.
use super::{
    PacketizedQueryWork, QueryPacketMapInput, QueryReadExecutionStop, QueryReadPacketDenial,
};
use crate::query::data::QueryWorkerFragment;
use worth_execution::{
    ExecutionMap, ExecutionResourceLease, MapKernelContext, MapKernelFailure, MapOutcome,
    MapPartition,
};
use worth_foundational::{ExecutionReport, PartitionIdentity};

// Only this module can create a packet name; scheduling has no constructor,
// conversion, arithmetic, default, or field access with which to rename it.
pub(super) struct PacketIdentity(u64);

pub(super) struct NamedQueryPacket {
    identity: PacketIdentity,
    input: QueryPacketMapInput,
}

pub(super) fn identities(packets: Vec<PacketizedQueryWork>) -> Vec<NamedQueryPacket> {
    packets
        .into_iter()
        .enumerate()
        .map(|(ordinal, work)| NamedQueryPacket {
            identity: PacketIdentity(
                u64::try_from(ordinal).expect("a packet ordinal fits u64") + 1,
            ),
            input: QueryPacketMapInput { ordinal, work },
        })
        .collect()
}

pub(super) fn run_packets(
    packets: Vec<NamedQueryPacket>,
    per_packet_ceiling: u64,
    lease: &ExecutionResourceLease<'_>,
    execute: impl Fn(
            &PacketizedQueryWork,
            usize,
            u64,
            &mut MapKernelContext<'_, '_>,
        ) -> Result<QueryWorkerFragment, MapKernelFailure<QueryReadPacketDenial>>
        + Sync,
) -> Result<(Vec<QueryWorkerFragment>, ExecutionReport), QueryReadExecutionStop> {
    let identities = packets
        .iter()
        .map(|packet| PartitionIdentity::new(packet.identity.0))
        .collect();
    let partitions = packets
        .into_iter()
        .map(|packet| MapPartition {
            identity: PartitionIdentity::new(packet.identity.0),
            value: packet.input,
            read_keys: Vec::<u64>::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: per_packet_ceiling,
            max_result_bytes: per_packet_ceiling,
        })
        .collect();
    let map = ExecutionMap::try_from_declared_partitions(identities, partitions)
        .map_err(QueryReadExecutionStop::PacketAdmission)?;
    match map.run(Some(lease), |input, context| {
        context.checkpoint(0)?;
        execute(&input.work, input.ordinal, per_packet_ceiling, context)
    }) {
        MapOutcome::Complete { values, report } => Ok((values, report)),
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
