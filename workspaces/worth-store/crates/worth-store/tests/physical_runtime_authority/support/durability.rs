use std::num::{NonZeroU32, NonZeroU64};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalDurabilityPolicy, CheckpointMemoryLimit, FilesystemMediaAdmission,
    GroupCommitDelay, GroupCommitLimit, IdempotencyRetentionGenerations,
    LiveIdempotencyBindingLimit, MediaOwnedPhysicalRuntime, PendingUnresolvedMutationLimit,
    PhysicalCheckpointPolicy, PhysicalDurabilityDeclaration, PhysicalIdempotencyPolicy,
    PhysicalRuntimeAdmission, PhysicalStore, PhysicalWalPolicy, RetainedWalTailLimit,
    WalSegmentByteLimit, WalSegmentInventoryLimit,
};
use worth_store_physical_backend::FilesystemAccessPosture;

pub fn media(root: &std::path::Path) -> MediaOwnedPhysicalRuntime {
    let runtime =
        PhysicalStore::admit(PhysicalRuntimeAdmission::new(root.to_path_buf()).unwrap()).unwrap();
    match runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        TransitionOutcome::Denied(denial) => panic!("media denied: {:?}", denial.reason()),
        TransitionOutcome::Deferred(deferred) => panic!("media deferred: {:?}", deferred.reason()),
        TransitionOutcome::Stale(stale) => panic!("media stale: {:?}", stale.reason()),
        TransitionOutcome::RebindRequired(rebind) => panic!("media rebind: {:?}", rebind.reason()),
        TransitionOutcome::Failed(failure) => panic!("media failed: {:?}", failure.cause()),
    }
}

pub fn admitted(media: &MediaOwnedPhysicalRuntime) -> AdmittedPhysicalDurabilityPolicy {
    let basis = media.physical_durability_admission_basis().unwrap();
    match PhysicalDurabilityDeclaration::builder()
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
        ))
        .admit(basis)
        .into_raw()
    {
        TransitionOutcome::Success(policy) => policy,
        _ => panic!("positive authority specimen requires admitted durability"),
    }
}
