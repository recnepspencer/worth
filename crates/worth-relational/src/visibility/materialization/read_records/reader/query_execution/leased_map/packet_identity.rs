//! Owns each packet's name and its binding before scheduling receives authority.
use super::{PacketizedQueryWork, QueryPacketMapInput, QueryReadExecutionStop};
use worth_execution::{ExecutionMap, MapPartition};
use worth_foundational::PartitionIdentity;

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

pub(super) fn declared_map(
    packets: Vec<NamedQueryPacket>,
    per_packet_ceiling: u64,
) -> Result<ExecutionMap<QueryPacketMapInput, u64>, QueryReadExecutionStop> {
    let partitions: Vec<_> = packets
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
    let identities = partitions
        .iter()
        .map(|partition| partition.identity)
        .collect();
    ExecutionMap::try_from_declared_partitions(identities, partitions)
        .map_err(QueryReadExecutionStop::PacketAdmission)
}
