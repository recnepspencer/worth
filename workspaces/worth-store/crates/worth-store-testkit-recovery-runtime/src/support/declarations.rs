pub use worth_proof::TransitionOutcome;
pub use {std::num::NonZeroU32, std::num::NonZeroU64, std::path::Path, std::process::Command};
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
    worth_store::physical_runtime::MediaOwnedPhysicalRuntime,
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
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::PhysicalRuntimeAdmission,
    worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::PhysicalWalPolicy,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordCountLimit,
    worth_store::physical_runtime::RecordReadLimits,
    worth_store::physical_runtime::RecordScanOutcome,
    worth_store::physical_runtime::RecordScanRequest,
    worth_store::physical_runtime::RetainedWalTailLimit,
    worth_store::physical_runtime::ServingPhysicalRuntime,
    worth_store::physical_runtime::WalSegmentByteLimit,
    worth_store::physical_runtime::WalSegmentInventoryLimit,
};
pub use {
    worth_store_recovery_runtime::PhysicalRecoveryLimitDeclaration,
    worth_store_recovery_runtime::PhysicalRecoveryLimits,
    worth_store_recovery_runtime::PhysicalRecoveryOpenRequest,
    worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority,
    worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration,
};

pub const EXTENT_BYTES: usize = 40_000;

pub fn observe_arena_in_separate_process(
    root: &Path,
    report_path: &Path,
    run: &str,
    scenario: &str,
) -> serde_json::Value {
    let executable = worth_store_test_support::OFFLINE_ROOT_OBSERVER
        .path()
        .expect("runner must prebuild the independent executable");
    let output = std::process::Command::new(executable)
        .args(["observe", "--store-root"])
        .arg(root)
        .arg("--report")
        .arg(report_path)
        .args([
            "--max-entries",
            "10000",
            "--max-bytes",
            "16777216",
            "--max-open-files",
            "8",
            "--max-depth",
            "12",
            "--max-symlinks",
            "0",
            "--max-elapsed-ms",
            "120000",
            "--max-report-bytes",
            "8388608",
            "--run",
            run,
            "--scenario",
            scenario,
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "independent observer failed: status={} stdout={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(report_path).unwrap()).unwrap();
    assert_eq!(report["role"], "offline-root-observer");
    assert_eq!(report["completeness"], "complete");
    assert_eq!(report["run"], run);
    assert_eq!(report["scenario"], scenario);
    let observer_pid: u32 = report["process"].as_str().unwrap().parse().unwrap();
    assert_ne!(observer_pid, std::process::id());
    let declared = &report["declared_limits"];
    assert_eq!(declared["entries"], 10000);
    assert_eq!(declared["bytes"], 16 * 1024 * 1024);
    assert_eq!(declared["open_files"], 8);
    assert_eq!(declared["depth"], 12);
    assert_eq!(declared["symlinks"], 0);
    assert_eq!(declared["elapsed_ms"], 120000);
    assert_eq!(declared["report_bytes"], 8 * 1024 * 1024);
    report
}

pub fn generation(catalog: &[u8]) -> u64 {
    u64::from_le_bytes(catalog[64..72].try_into().unwrap())
}

pub fn recovery_request(root: &Path) -> PhysicalRecoveryOpenRequest {
    let limits = worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(
        PhysicalRecoveryLimitDeclaration {
            selector_candidates: 4,
            checkpoint_candidates: 8,
            manifest_bytes: 16 << 20,
            manifest_entries: 4096,
            wal_segments: 8,
            wal_frames: 4096,
            wal_bytes: 16 << 20,
            redo_targets: 4096,
            redo_bytes: 64 << 20,
            distinct_pages_and_extents: 4096,
            operation_bindings: 4096,
            staging_bytes: 64 << 20,
            recovery_memory_bytes: 64 << 20,
            dirty_frames: 4096,
            concurrent_commands: 8,
            publication_effects: 64,
            cleanup_candidates: 4096,
            cleanup_bytes: 64 << 20,
            observation_bytes: 64 << 20,
        },
    )
    .unwrap();
    let configuration =
        worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current();
    let authority = worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority::acquire(
        root.to_path_buf(),
        configuration.clone(),
        limits,
    )
    .unwrap();
    let profile = authority.qualified_backend_profile().clone();
    worth_store_recovery_runtime::PhysicalRecoveryOpenRequest::declare(
        root.to_path_buf(),
        configuration,
        profile,
        limits,
        authority,
    )
}

pub fn configuration() -> (
    AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    worth_store::physical_runtime::AdmittedRecordAccessPolicy,
) {
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap())
        .admit(format)
        .unwrap();
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .unwrap();
    (format, placement, access)
}

pub fn media(root: &Path) -> MediaOwnedPhysicalRuntime {
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
        panic!("physical media must admit");
    };
    media
}

pub fn durability(
    media: &MediaOwnedPhysicalRuntime,
) -> worth_store::physical_runtime::AdmittedPhysicalDurabilityPolicy {
    let worth_proof::TransitionOutcome::Success(policy) =
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
        panic!("durability policy must admit");
    };
    policy
}

pub fn initialize(root: &Path) -> ServingPhysicalRuntime {
    let (format, placement, access) = configuration();
    let media = media(root);
    let durability = durability(&media);
    let worth_proof::TransitionOutcome::Success(serving) = media
        .initialize_record_store(
            worth_store::physical_runtime::PhysicalRecordInitialization::new(
                format, placement, access, durability,
            ),
        )
        .into_raw()
    else {
        panic!("store initialization must admit");
    };
    serving
}

pub fn open(root: &Path) -> ServingPhysicalRuntime {
    let (format, _, access) = configuration();
    let media = media(root);
    let durability = durability(&media);
    let worth_proof::TransitionOutcome::Success(serving) = media
        .open_record_store(worth_store::physical_runtime::PhysicalRecordOpen::new(
            format, access, durability,
        ))
        .into_raw()
    else {
        panic!("store open must admit");
    };
    serving
}

pub fn append(
    serving: &ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: [u8; 32],
    payload: &[u8],
) -> worth_store::physical_runtime::CompletedPhysicalMutation {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(material),
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
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(1_000)
                    .unwrap(),
            ),
        )
        .into_raw()
    else {
        panic!("append must prepare");
    };
    let worth_store::physical_runtime::PhysicalMutationOutcome::Completed(completed) =
        prepared.execute()
    else {
        panic!("append must complete");
    };
    completed
}

pub fn scan(
    serving: &ServingPhysicalRuntime,
) -> Vec<(worth_store::physical_runtime::PhysicalRecordId, Vec<u8>)> {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            worth_store::physical_runtime::RecordScanRequest::from_start()
                .with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(2).unwrap()),
        )
        .unwrap();
    let mut scratch = vec![0; 128_000];
    let mut records = Vec::new();
    while let worth_store::physical_runtime::RecordScanOutcome::Batch(batch) =
        scan.read_next_into(&mut scratch).unwrap()
    {
        for index in 0..batch.records().len() {
            records.push(batch.records()[index].record_id());
        }
        if batch.is_complete() {
            break;
        }
    }
    drop(scan);
    records
        .into_iter()
        .map(|record| (record, read_record(serving, record)))
        .collect()
}

pub fn read_record(
    serving: &ServingPhysicalRuntime,
    record: worth_store::physical_runtime::PhysicalRecordId,
) -> Vec<u8> {
    let reader = serving.records().unwrap();
    let mut session = reader
        .open(
            record,
            worth_store::physical_runtime::RecordReadLimits::new(
                worth_store::physical_runtime::RecordByteLimit::new(EXTENT_BYTES as u32).unwrap(),
            ),
        )
        .unwrap();
    let mut payload = Vec::new();
    let mut scratch = [0; 4096];
    loop {
        let count = session.read_next(&mut scratch).unwrap();
        if count == 0 {
            break;
        }
        payload.extend_from_slice(&scratch[..count]);
    }
    payload
}
