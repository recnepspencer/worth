use std::num::{NonZeroU32, NonZeroU64};

use worth_store::physical_runtime::{
    CheckpointMemoryLimit, GroupCommitDelay, GroupCommitLimit,
    IdempotencyRetentionGenerations, LiveIdempotencyBindingLimit,
    PendingUnresolvedMutationLimit,
    PhysicalCheckpointPolicy, PhysicalDurabilityDeclaration, PhysicalIdempotencyPolicy,
    PhysicalWalPolicy, RetainedWalTailLimit, WalSegmentByteLimit, WalSegmentInventoryLimit,
};
use worth_store_physical_backend::BackendTargetProfile;

fn main() {
    let declaration = PhysicalDurabilityDeclaration::builder()
        .group_commit(
            GroupCommitLimit::new(NonZeroU32::new(32).unwrap()),
            GroupCommitDelay::new(NonZeroU64::new(1).unwrap()),
        )
        .wal(PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(NonZeroU64::new(8 * 1024 * 1024).unwrap()),
            WalSegmentInventoryLimit::new(NonZeroU32::new(1_024).unwrap()),
        ))
        .idempotency(PhysicalIdempotencyPolicy::new(
            IdempotencyRetentionGenerations::new(NonZeroU64::new(4).unwrap()),
            PendingUnresolvedMutationLimit::new(NonZeroU32::new(1_024).unwrap()),
            LiveIdempotencyBindingLimit::new(NonZeroU32::new(4_096).unwrap()),
        ))
        .checkpoint(PhysicalCheckpointPolicy::fuzzy(
            CheckpointMemoryLimit::new(NonZeroU64::new(16 * 1024 * 1024).unwrap()),
            RetainedWalTailLimit::new(NonZeroU64::new(64 * 1024 * 1024).unwrap()),
        ));
    let _ = declaration.admit(BackendTargetProfile::PosixFileFsyncDirSync);
}
