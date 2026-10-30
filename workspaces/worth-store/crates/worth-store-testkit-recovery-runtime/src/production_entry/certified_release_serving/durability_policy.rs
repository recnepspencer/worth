use std::num::NonZeroU64;

/// The durability policy these worlds serve under, which recovery before the
/// first checkpoint must be configured with.
pub fn serving_durability(
    wal_segment_bytes: NonZeroU64,
) -> worth_store::physical_runtime::ConfiguredPhysicalDurabilityDeclaration {
    use std::num::NonZeroU32;
    use {
        worth_store::physical_runtime::CheckpointMemoryLimit,
        worth_store::physical_runtime::GroupCommitDelay,
        worth_store::physical_runtime::GroupCommitLimit,
        worth_store::physical_runtime::IdempotencyRetentionGenerations,
        worth_store::physical_runtime::LiveIdempotencyBindingLimit,
        worth_store::physical_runtime::PendingUnresolvedMutationLimit,
        worth_store::physical_runtime::PhysicalCheckpointPolicy,
        worth_store::physical_runtime::PhysicalDurabilityDeclaration,
        worth_store::physical_runtime::PhysicalIdempotencyPolicy,
        worth_store::physical_runtime::PhysicalWalPolicy,
        worth_store::physical_runtime::RetainedWalTailLimit,
        worth_store::physical_runtime::WalSegmentByteLimit,
        worth_store::physical_runtime::WalSegmentInventoryLimit,
    };

    PhysicalDurabilityDeclaration::builder()
        .group_commit(
            GroupCommitLimit::new(NonZeroU32::new(32).unwrap()),
            GroupCommitDelay::new(std::num::NonZeroU64::new(1).unwrap()),
        )
        .wal(PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(wal_segment_bytes),
            WalSegmentInventoryLimit::new(NonZeroU32::new(1024).unwrap()),
        ))
        .idempotency(PhysicalIdempotencyPolicy::new(
            IdempotencyRetentionGenerations::new(std::num::NonZeroU64::new(4).unwrap()),
            PendingUnresolvedMutationLimit::new(NonZeroU32::new(1024).unwrap()),
            LiveIdempotencyBindingLimit::new(NonZeroU32::new(4096).unwrap()),
        ))
        .checkpoint(PhysicalCheckpointPolicy::fuzzy(
            CheckpointMemoryLimit::new(std::num::NonZeroU64::new(16 << 20).unwrap()),
            RetainedWalTailLimit::new(std::num::NonZeroU64::new(64 << 20).unwrap()),
        ))
}
