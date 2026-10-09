use std::num::{NonZeroU32, NonZeroU64};
use std::path::Path;

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, CheckpointMemoryLimit, FilesystemAccessPosture,
    FilesystemMediaAdmission, GroupCommitDelay, GroupCommitLimit, IdempotencyRetentionGenerations,
    LiveIdempotencyBindingLimit, ManifestEntryCapacity, PendingUnresolvedMutationLimit,
    PhysicalCheckpointPolicy, PhysicalDurabilityDeclaration, PhysicalIdempotencyPolicy,
    PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalRecordAccessPolicy,
    PhysicalRecordFormatDeclaration, PhysicalRecordOpen, PhysicalRecordPlacementPolicy,
    PhysicalRuntimeAdmission, PhysicalStore, PhysicalWalPolicy, RecordAppendBatch, RecordByteLimit,
    RetainedWalTailLimit, SegmentPageCount, ServingPhysicalRuntime, WalSegmentByteLimit,
    WalSegmentInventoryLimit,
};

pub fn advance_with_ordinary_appends(root: &Path, span: bool) {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let placement = if span {
        PhysicalRecordPlacementPolicy::builder()
            .segment_pages(SegmentPageCount::new(16).unwrap())
            .extent_threshold(RecordByteLimit::new(8_000).unwrap())
            .page_fill(worth_store::physical_runtime::PageFillPercent::new(50).unwrap())
            .manifest_capacity(ManifestEntryCapacity::new(128).unwrap())
            .admit(format)
            .unwrap()
    } else {
        PhysicalRecordPlacementPolicy::builder()
            .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
            .admit(format)
            .unwrap()
    };
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("ordinary media admission must succeed");
    };
    let TransitionOutcome::Success(durability) = PhysicalDurabilityDeclaration::builder()
        .group_commit(
            GroupCommitLimit::new(NonZeroU32::new(32).unwrap()),
            GroupCommitDelay::new(NonZeroU64::new(1).unwrap()),
        )
        .wal(PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(NonZeroU64::new(16 << 20).unwrap()),
            WalSegmentInventoryLimit::new(NonZeroU32::new(1024).unwrap()),
        ))
        .idempotency(PhysicalIdempotencyPolicy::new(
            IdempotencyRetentionGenerations::new(NonZeroU64::new(4).unwrap()),
            PendingUnresolvedMutationLimit::new(NonZeroU32::new(1024).unwrap()),
            LiveIdempotencyBindingLimit::new(NonZeroU32::new(4096).unwrap()),
        ))
        .checkpoint(PhysicalCheckpointPolicy::fuzzy(
            CheckpointMemoryLimit::new(NonZeroU64::new(16 << 20).unwrap()),
            RetainedWalTailLimit::new(NonZeroU64::new(64 << 20).unwrap()),
        ))
        .admit(media.physical_durability_admission_basis().unwrap())
        .into_raw()
    else {
        panic!("ordinary durability admission must succeed");
    };
    let TransitionOutcome::Success(serving) = media
        .open_record_store(PhysicalRecordOpen::new(format, access, durability))
        .into_raw()
    else {
        panic!("recovered store must reopen for ordinary appends");
    };
    append(&serving, placement, [0xc1; 32], b"later-root-one");
    append(&serving, placement, [0xc2; 32], b"later-root-two");
    serving.close();
}

pub fn selected_catalog_generation(root: &Path) -> u64 {
    let catalog = std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap();
    u64::from_le_bytes(catalog[64..72].try_into().unwrap())
}

fn append(
    serving: &ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    idempotency: [u8; 32],
    payload: &[u8],
) {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(idempotency))
        .unwrap();
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                RecordAppendBatch::try_from_iter([payload]).unwrap(),
                placement,
                PhysicalMutationRequest::platform_durable(
                    key,
                    PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                ),
            )
            .into_raw()
    else {
        panic!("ordinary later append must prepare");
    };
    assert!(matches!(
        prepared.execute(),
        PhysicalMutationOutcome::Completed(_)
    ));
}
