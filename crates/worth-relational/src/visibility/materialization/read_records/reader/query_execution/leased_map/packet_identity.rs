//! Names the canonical partition set before scheduling receives any authority.
use super::PacketizedQueryWork;
use worth_foundational::PartitionIdentity;

pub(super) fn identities(packets: &[PacketizedQueryWork]) -> Vec<PartitionIdentity> {
    packets
        .iter()
        .enumerate()
        .map(|(ordinal, _)| PartitionIdentity::new(ordinal as u64 + 1))
        .collect()
}
