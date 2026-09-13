use std::{
    num::{NonZeroU32, NonZeroU64},
    path::Path,
};

use worth_proof::TransitionOutcome;

use super::durability_profile::WriterDurabilityProfile;
use worth_store::physical_runtime::{
    AdmittedPhysicalDurabilityPolicy, CheckpointMemoryLimit, FilesystemMediaAdmission,
    GroupCommitDelay, GroupCommitLimit, IdempotencyRetentionGenerations,
    LiveIdempotencyBindingLimit, MediaOwnedPhysicalRuntime, PendingUnresolvedMutationLimit,
    PhysicalCheckpointPolicy, PhysicalDurabilityDeclaration, PhysicalIdempotencyPolicy,
    PhysicalRuntimeAdmission, PhysicalStore, PhysicalWalPolicy, RecordServingAdmissionOutcome,
    RetainedWalTailLimit, ServingPhysicalRuntime, WalSegmentByteLimit, WalSegmentInventoryLimit,
};
use worth_store_physical_backend::FilesystemAccessPosture;

const CHECKPOINT_MEMORY_BUDGET_BYTES: u64 = 512 * 1024;

pub(super) fn admit_media(root: &Path) -> Result<MediaOwnedPhysicalRuntime, String> {
    let runtime = PhysicalStore::admit(
        PhysicalRuntimeAdmission::new(root)
            .map_err(|denial| format!("ordinary writer root denied: {denial:?}"))?,
    )
    .map_err(|denial| format!("ordinary writer runtime denied: {denial:?}"))?;
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    match runtime.try_admit_filesystem_media(admission).into_raw() {
        TransitionOutcome::Success(media) => Ok(media),
        TransitionOutcome::Denied(_) => {
            Err("ordinary writer media admission was denied".to_owned())
        }
        TransitionOutcome::Deferred(_) => {
            Err("ordinary writer media admission was deferred".to_owned())
        }
        TransitionOutcome::Stale(_) => Err("ordinary writer media admission was stale".to_owned()),
        TransitionOutcome::RebindRequired(_) => {
            Err("ordinary writer media admission required rebinding".to_owned())
        }
        TransitionOutcome::Failed(_) => {
            Err("ordinary writer media admission required inspection".to_owned())
        }
    }
}

pub(super) fn require_serving<Denial>(
    outcome: RecordServingAdmissionOutcome<Denial>,
    operation: &str,
) -> Result<ServingPhysicalRuntime, String> {
    match outcome.into_raw() {
        TransitionOutcome::Success(serving) => Ok(serving),
        TransitionOutcome::Denied(_) => Err(format!("{operation} was denied")),
        TransitionOutcome::Deferred(_) => Err(format!("{operation} was deferred")),
        TransitionOutcome::Stale(_) => Err(format!("{operation} was stale")),
        TransitionOutcome::RebindRequired(_) => Err(format!("{operation} required rebinding")),
        TransitionOutcome::Failed(inspection) => Err(format!(
            "{operation} required inspection: {:?}",
            inspection.cause()
        )),
    }
}

pub(super) fn admit_durability(
    media: &MediaOwnedPhysicalRuntime,
    profile: WriterDurabilityProfile,
) -> Result<AdmittedPhysicalDurabilityPolicy, String> {
    let basis = media
        .physical_durability_admission_basis()
        .map_err(|denial| format!("ordinary writer durability basis denied: {denial:?}"))?;
    match PhysicalDurabilityDeclaration::builder()
        .group_commit(
            GroupCommitLimit::new(NonZeroU32::new(32).unwrap()),
            GroupCommitDelay::new(NonZeroU64::new(1).unwrap()),
        )
        .wal(PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(NonZeroU64::new(profile.wal_segment_byte_limit()).unwrap()),
            WalSegmentInventoryLimit::new(NonZeroU32::new(64).unwrap()),
        ))
        .idempotency(PhysicalIdempotencyPolicy::new(
            IdempotencyRetentionGenerations::new(NonZeroU64::new(8).unwrap()),
            PendingUnresolvedMutationLimit::new(NonZeroU32::new(1_024).unwrap()),
            LiveIdempotencyBindingLimit::new(NonZeroU32::new(4_096).unwrap()),
        ))
        .checkpoint(PhysicalCheckpointPolicy::fuzzy(
            CheckpointMemoryLimit::new(NonZeroU64::new(CHECKPOINT_MEMORY_BUDGET_BYTES).unwrap()),
            RetainedWalTailLimit::new(NonZeroU64::new(64 * 1024 * 1024).unwrap()),
        ))
        .admit(basis)
        .into_raw()
    {
        TransitionOutcome::Success(policy) => Ok(policy),
        TransitionOutcome::Denied(denial) => Err(format!(
            "ordinary writer durability policy denied: {denial:?}"
        )),
        TransitionOutcome::Deferred(_) => {
            Err("ordinary writer durability policy was deferred".to_owned())
        }
        TransitionOutcome::Stale(_) => {
            Err("ordinary writer durability policy was stale".to_owned())
        }
        TransitionOutcome::RebindRequired(_) => {
            Err("ordinary writer durability policy required rebinding".to_owned())
        }
        TransitionOutcome::Failed(_) => {
            Err("ordinary writer durability policy required inspection".to_owned())
        }
    }
}
