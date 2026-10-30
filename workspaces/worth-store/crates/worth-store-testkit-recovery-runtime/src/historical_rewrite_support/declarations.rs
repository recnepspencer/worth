pub use std::path::Path;
pub use {std::num::NonZeroU32, std::num::NonZeroU64};

pub use worth_proof::TransitionOutcome;
pub use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::CheckpointMemoryLimit,
    worth_store::physical_runtime::FilesystemAccessPosture,
    worth_store::physical_runtime::FilesystemMediaAdmission,
    worth_store::physical_runtime::GroupCommitDelay,
    worth_store::physical_runtime::GroupCommitLimit,
    worth_store::physical_runtime::IdempotencyRetentionGenerations,
    worth_store::physical_runtime::LiveIdempotencyBindingLimit,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::PendingUnresolvedMutationLimit,
    worth_store::physical_runtime::PhysicalCheckpointPolicy,
    worth_store::physical_runtime::PhysicalDurabilityDeclaration,
    worth_store::physical_runtime::PhysicalIdempotencyPolicy,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalRecordAccessPolicy,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::PhysicalRuntimeAdmission,
    worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::PhysicalWalPolicy,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RetainedWalTailLimit,
    worth_store::physical_runtime::SegmentPageCount,
    worth_store::physical_runtime::ServingPhysicalRuntime,
    worth_store::physical_runtime::WalSegmentByteLimit,
    worth_store::physical_runtime::WalSegmentInventoryLimit,
};

pub fn advance_with_ordinary_appends(root: &Path, span: bool) {
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .unwrap();
    let placement = if span {
        worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
            .segment_pages(worth_store::physical_runtime::SegmentPageCount::new(16).unwrap())
            .extent_threshold(worth_store::physical_runtime::RecordByteLimit::new(8_000).unwrap())
            .page_fill(worth_store::physical_runtime::PageFillPercent::new(50).unwrap())
            .manifest_capacity(
                worth_store::physical_runtime::ManifestEntryCapacity::new(128).unwrap(),
            )
            .admit(format)
            .unwrap()
    } else {
        worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
            .manifest_capacity(
                worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap(),
            )
            .admit(format)
            .unwrap()
    };
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(
        worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap(),
    )
    .unwrap();
    let worth_proof::TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(
            worth_store::physical_runtime::FilesystemMediaAdmission::production(
                worth_store::physical_runtime::FilesystemAccessPosture::CoordinatedServiceAccount,
            ),
        )
        .into_raw()
    else {
        panic!("ordinary media admission must succeed");
    };
    let worth_proof::TransitionOutcome::Success(durability) =
        worth_store::physical_runtime::PhysicalDurabilityDeclaration::builder()
            .group_commit(
                worth_store::physical_runtime::GroupCommitLimit::new(
                    std::num::NonZeroU32::new(32).unwrap(),
                ),
                worth_store::physical_runtime::GroupCommitDelay::new(
                    std::num::NonZeroU64::new(1).unwrap(),
                ),
            )
            .wal(worth_store::physical_runtime::PhysicalWalPolicy::segmented(
                worth_store::physical_runtime::WalSegmentByteLimit::new(
                    std::num::NonZeroU64::new(16 << 20).unwrap(),
                ),
                worth_store::physical_runtime::WalSegmentInventoryLimit::new(
                    std::num::NonZeroU32::new(1024).unwrap(),
                ),
            ))
            .idempotency(
                worth_store::physical_runtime::PhysicalIdempotencyPolicy::new(
                    worth_store::physical_runtime::IdempotencyRetentionGenerations::new(
                        std::num::NonZeroU64::new(4).unwrap(),
                    ),
                    worth_store::physical_runtime::PendingUnresolvedMutationLimit::new(
                        std::num::NonZeroU32::new(1024).unwrap(),
                    ),
                    worth_store::physical_runtime::LiveIdempotencyBindingLimit::new(
                        std::num::NonZeroU32::new(4096).unwrap(),
                    ),
                ),
            )
            .checkpoint(
                worth_store::physical_runtime::PhysicalCheckpointPolicy::fuzzy(
                    worth_store::physical_runtime::CheckpointMemoryLimit::new(
                        std::num::NonZeroU64::new(16 << 20).unwrap(),
                    ),
                    worth_store::physical_runtime::RetainedWalTailLimit::new(
                        std::num::NonZeroU64::new(64 << 20).unwrap(),
                    ),
                ),
            )
            .admit(media.physical_durability_admission_basis().unwrap())
            .into_raw()
    else {
        panic!("ordinary durability admission must succeed");
    };
    let worth_proof::TransitionOutcome::Success(serving) = media
        .open_record_store(worth_store::physical_runtime::PhysicalRecordOpen::new(
            format, access, durability,
        ))
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

pub fn append(
    serving: &ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    idempotency: [u8; 32],
    payload: &[u8],
) {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(idempotency),
        )
        .unwrap();
    let worth_proof::TransitionOutcome::Success(
        worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
    ) = submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([payload]).unwrap(),
            placement,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
            ),
        )
        .into_raw()
    else {
        panic!("ordinary later append must prepare");
    };
    assert!(matches!(
        prepared.execute(),
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_)
    ));
}
