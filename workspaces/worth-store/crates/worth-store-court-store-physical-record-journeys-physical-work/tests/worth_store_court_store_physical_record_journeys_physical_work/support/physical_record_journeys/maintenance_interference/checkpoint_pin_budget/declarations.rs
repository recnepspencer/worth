pub(crate) use {std::fs, std::num::NonZeroU32, std::num::NonZeroU64, std::path::Path};

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::CheckpointMemoryLimit, worth_store::physical_runtime::GroupCommitDelay, worth_store::physical_runtime::GroupCommitLimit, worth_store::physical_runtime::IdempotencyRetentionGenerations, worth_store::physical_runtime::LiveIdempotencyBindingLimit, worth_store::physical_runtime::PendingUnresolvedMutationLimit, worth_store::physical_runtime::PhysicalCheckpointCaptureFailureKind, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointPolicy, worth_store::physical_runtime::PhysicalCheckpointProvenNoEffectCause, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalDurabilityDeclaration, worth_store::physical_runtime::PhysicalIdempotencyPolicy, worth_store::physical_runtime::PhysicalRecordAccessPolicy, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalWalPolicy, worth_store::physical_runtime::RetainedWalTailLimit, worth_store::physical_runtime::ServingPhysicalRuntime, worth_store::physical_runtime::WalSegmentByteLimit, worth_store::physical_runtime::WalSegmentInventoryLimit};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::maintenance_interference::append, worth_store_testkit_store::physical_record_journeys::maintenance_interference::checkpoint, worth_store_testkit_store::physical_record_journeys::maintenance_interference::initialize, worth_store_testkit_store::physical_record_journeys::maintenance_interference::placement};

pub(crate) const MAINTENANCE_LIMIT: u64 = 16 * 1024 * 1024;

pub(crate) fn initialize_with_checkpoint_budget(root: &Path, checkpoint_bytes: u64) -> ServingPhysicalRuntime {
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let media = worth_store_testkit_store::physical_record_journeys::media(root);
    let durability = match worth_store::physical_runtime::PhysicalDurabilityDeclaration::builder()
        .group_commit(
            worth_store::physical_runtime::GroupCommitLimit::new(std::num::NonZeroU32::new(32).unwrap()),
            worth_store::physical_runtime::GroupCommitDelay::new(std::num::NonZeroU64::new(1).unwrap()),
        )
        .wal(worth_store::physical_runtime::PhysicalWalPolicy::segmented(
            worth_store::physical_runtime::WalSegmentByteLimit::new(std::num::NonZeroU64::new(512 * 1024).unwrap()),
            worth_store::physical_runtime::WalSegmentInventoryLimit::new(std::num::NonZeroU32::new(64).unwrap()),
        ))
        .idempotency(worth_store::physical_runtime::PhysicalIdempotencyPolicy::new(
            worth_store::physical_runtime::IdempotencyRetentionGenerations::new(std::num::NonZeroU64::new(4).unwrap()),
            worth_store::physical_runtime::PendingUnresolvedMutationLimit::new(std::num::NonZeroU32::new(1_024).unwrap()),
            worth_store::physical_runtime::LiveIdempotencyBindingLimit::new(std::num::NonZeroU32::new(4_096).unwrap()),
        ))
        .checkpoint(worth_store::physical_runtime::PhysicalCheckpointPolicy::fuzzy(
            worth_store::physical_runtime::CheckpointMemoryLimit::new(std::num::NonZeroU64::new(checkpoint_bytes).unwrap()),
            worth_store::physical_runtime::RetainedWalTailLimit::new(std::num::NonZeroU64::new(64 * 1024 * 1024).unwrap()),
        ))
        .admit(media.physical_durability_admission_basis().unwrap())
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(policy) => policy,
        _ => panic!("low-budget durability must admit"),
    };
    worth_store_testkit_store::physical_record_journeys::success(
        media.initialize_record_store(
            worth_store::physical_runtime::PhysicalRecordInitialization::new(
                format,
                worth_store_testkit_store::physical_record_journeys::maintenance_interference::placement_for(format),
                access,
                durability,
            )
            .with_residency_policy(worth_store_testkit_store::physical_record_journeys::maintenance_interference::residency(format, worth_store_testkit_store::physical_record_journeys::maintenance_interference::RESIDENT_BYTES)),
        ),
    )
}

pub(crate) fn checkpoint_request(ordinal: u64) -> PhysicalCheckpointRequest {
    let mut key = [0x6C; 32];
    key[..8].copy_from_slice(&ordinal.to_le_bytes());
    worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new(key),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    )
}