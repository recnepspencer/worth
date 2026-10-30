use super::*;

pub fn durability(media: &MediaOwnedPhysicalRuntime) -> AdmittedPhysicalDurabilityPolicy {
    durability_with_wal_segment_bytes(media, 16 * 1024 * 1024)
}

pub fn durability_with_wal_segment_bytes(
    media: &MediaOwnedPhysicalRuntime,
    segment_bytes: u64,
) -> AdmittedPhysicalDurabilityPolicy {
    durability_with_limits(media, segment_bytes, 64 * 1024 * 1024)
}

pub fn durability_with_limits(
    media: &MediaOwnedPhysicalRuntime,
    segment_bytes: u64,
    retained_wal_tail_bytes: u64,
) -> AdmittedPhysicalDurabilityPolicy {
    let basis = media.physical_durability_admission_basis().unwrap();
    let wal = worth_store::physical_runtime::PhysicalWalPolicy::segmented(
        worth_store::physical_runtime::WalSegmentByteLimit::new(
            std::num::NonZeroU64::new(segment_bytes).unwrap(),
        ),
        worth_store::physical_runtime::WalSegmentInventoryLimit::new(
            std::num::NonZeroU32::new(1_024).unwrap(),
        ),
    );
    match worth_store::physical_runtime::PhysicalDurabilityDeclaration::builder()
        .group_commit(
            worth_store::physical_runtime::GroupCommitLimit::new(
                std::num::NonZeroU32::new(32).unwrap(),
            ),
            worth_store::physical_runtime::GroupCommitDelay::new(
                std::num::NonZeroU64::new(1).unwrap(),
            ),
        )
        .wal(wal)
        .idempotency(
            worth_store::physical_runtime::PhysicalIdempotencyPolicy::new(
                worth_store::physical_runtime::IdempotencyRetentionGenerations::new(
                    std::num::NonZeroU64::new(4).unwrap(),
                ),
                worth_store::physical_runtime::PendingUnresolvedMutationLimit::new(
                    std::num::NonZeroU32::new(1_024).unwrap(),
                ),
                worth_store::physical_runtime::LiveIdempotencyBindingLimit::new(
                    std::num::NonZeroU32::new(4_096).unwrap(),
                ),
            ),
        )
        .checkpoint(
            worth_store::physical_runtime::PhysicalCheckpointPolicy::fuzzy(
                worth_store::physical_runtime::CheckpointMemoryLimit::new(
                    std::num::NonZeroU64::new(16 * 1024 * 1024).unwrap(),
                ),
                worth_store::physical_runtime::RetainedWalTailLimit::new(
                    std::num::NonZeroU64::new(retained_wal_tail_bytes).unwrap(),
                ),
            ),
        )
        .admit(basis)
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(policy) => policy,
        _ => panic!("blob journey durability must admit"),
    }
}
