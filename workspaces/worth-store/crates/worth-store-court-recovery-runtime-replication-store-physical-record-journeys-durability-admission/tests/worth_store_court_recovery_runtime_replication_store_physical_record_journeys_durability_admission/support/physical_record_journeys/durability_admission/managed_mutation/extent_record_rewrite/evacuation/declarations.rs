pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::produced_retirement_payloads, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction};
pub(crate) use {worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::ServingPhysicalRuntime, worth_signal::facade::TemporalDuration, worth_proof::TransitionOutcome, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::manifest_fixture::current_extent_route, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::prepare_extent_rewrite, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::read_record, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::request};
pub(crate) use {std::process::Command};
pub(crate) use {worth_store::physical_runtime::ArenaEvacuationThreshold, worth_store::physical_runtime::ExtentArenaCapacity, worth_store::physical_runtime::PhysicalArenaEvacuationPreparationOutcome, worth_store::physical_runtime::PhysicalExtentCopyPhase, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::PhysicalWorkCounterStage, worth_store::physical_runtime::PhysicalWorkOperationFamily, worth_store::physical_runtime::PhysicalWorkPressureClass};
pub(crate) use {worth_store_physical_format::DURABLE_EXTENT_FRAME_HEADER_BYTES, worth_store_physical_format::EXTENT_CHUNK_METADATA_BYTES};




pub(crate) const READER_CHILD: &str = "durability_admission::managed_mutation::extent_record_rewrite::evacuation::evacuation_reader::evacuation_reader_child";
pub(crate) const REOPEN_DIRECTORY: &str = "C11_EVACUATION_REOPEN_DIRECTORY";
pub(crate) const LIVE_PAYLOAD_BYTES: usize = 2 * 1024 * 1024 + 1;

pub(crate) fn copy_checkpoint(
    serving: &ServingPhysicalRuntime,
    ordinal: u8,
) -> worth_store::physical_runtime::CompletedPhysicalCheckpoint {
    use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest};
    let mut key = [214; 32];
    key[0] = ordinal;
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::at(worth_signal::facade::TemporalDuration::temporal_duration(30_000).unwrap()),
    );
    let worth_proof::TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("copy checkpoint must admit");
    };
    let PhysicalCheckpointOutcome::Completed(completed) = handle.wait() else {
        panic!("copy checkpoint must complete");
    };
    completed
}

pub(crate) fn material(ordinal: u64) -> [u8; 32] {
    let mut material = [197; 32];
    material[..8].copy_from_slice(&ordinal.to_le_bytes());
    material
}

pub(crate) fn fill_durability(
    media: &worth_store::physical_runtime::MediaOwnedPhysicalRuntime,
) -> worth_store::physical_runtime::AdmittedPhysicalDurabilityPolicy {
    use {std::num::NonZeroU32, std::num::NonZeroU64};
    use {worth_store::physical_runtime::CheckpointMemoryLimit, worth_store::physical_runtime::GroupCommitDelay, worth_store::physical_runtime::GroupCommitLimit, worth_store::physical_runtime::IdempotencyRetentionGenerations, worth_store::physical_runtime::LiveIdempotencyBindingLimit, worth_store::physical_runtime::PendingUnresolvedMutationLimit, worth_store::physical_runtime::PhysicalCheckpointPolicy, worth_store::physical_runtime::PhysicalDurabilityDeclaration, worth_store::physical_runtime::PhysicalIdempotencyPolicy, worth_store::physical_runtime::PhysicalWalPolicy, worth_store::physical_runtime::RetainedWalTailLimit, worth_store::physical_runtime::WalSegmentByteLimit, worth_store::physical_runtime::WalSegmentInventoryLimit};
    // The scale journey retains every publication's real WAL image. Admit that
    // budget publicly instead of bypassing the retention owner.
    let outcome = PhysicalDurabilityDeclaration::builder()
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
            RetainedWalTailLimit::new(NonZeroU64::new(512 << 20).unwrap()),
        ))
        .admit(media.physical_durability_admission_basis().unwrap())
        .into_raw();
    let worth_proof::TransitionOutcome::Success(policy) = outcome else {
        panic!("scale WAL policy must admit");
    };
    policy
}