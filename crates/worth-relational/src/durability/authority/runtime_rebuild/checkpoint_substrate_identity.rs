//! Pointer equality for the authenticated partition substrate reused by recovery.
use super::{partition_images, PartitionId};

pub(super) fn shares_first_generation(
    left: &partition_images::RestoredPartitions,
    right: &partition_images::RestoredPartitions,
) -> bool {
    shares_partition_generation(left, right, PartitionId::main())
}

pub(super) fn shares_partition_generation(
    left: &partition_images::RestoredPartitions,
    right: &partition_images::RestoredPartitions,
    partition_id: PartitionId,
) -> bool {
    let left = left.get(&partition_id).unwrap();
    let right = right.get(&partition_id).unwrap();
    std::ptr::eq(
        left.entity_arena.generations.get(0).unwrap(),
        right.entity_arena.generations.get(0).unwrap(),
    )
}
