use std::{
    num::{NonZeroU32, NonZeroU64},
    path::Path,
    process::Command,
};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, CheckpointMemoryLimit, FilesystemAccessPosture,
    FilesystemMediaAdmission, GroupCommitDelay, GroupCommitLimit, IdempotencyRetentionGenerations,
    LiveIdempotencyBindingLimit, ManifestEntryCapacity, MediaOwnedPhysicalRuntime,
    PendingUnresolvedMutationLimit, PhysicalCheckpointPolicy, PhysicalDurabilityDeclaration,
    PhysicalIdempotencyPolicy, PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationSuccess, PhysicalMutationRequest,
    PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration, PhysicalRecordInitialization,
    PhysicalRecordOpen, PhysicalRecordPlacementPolicy, PhysicalRuntimeAdmission, PhysicalStore,
    PhysicalWalPolicy, RecordAppendBatch, RecordByteLimit, RecordCountLimit, RecordReadLimits,
    RecordScanOutcome, RecordScanRequest, RetainedWalTailLimit, ServingPhysicalRuntime,
    WalSegmentByteLimit, WalSegmentInventoryLimit,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimits, PhysicalRecoveryOpenRequest,
    PhysicalRecoveryPlatformAuthority, PhysicalRecoveryStaticConfiguration,
};

pub(crate) const EXTENT_BYTES: usize = 40_000;

pub(crate) fn observe_arena_in_separate_process(
    root: &Path,
    report_path: &Path,
    run: &str,
    scenario: &str,
) -> serde_json::Value {
    const OBSERVER: &str = "WORTH_C9_OBSERVER_EXECUTABLE";
    let executable = std::env::var_os(OBSERVER)
        .unwrap_or_else(|| panic!("{OBSERVER} must name the independent observer executable"));
    let output = Command::new(executable)
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

pub(crate) fn generation(catalog: &[u8]) -> u64 {
    u64::from_le_bytes(catalog[64..72].try_into().unwrap())
}

pub(crate) fn recovery_request(root: &Path) -> PhysicalRecoveryOpenRequest {
    let limits = PhysicalRecoveryLimits::admit(PhysicalRecoveryLimitDeclaration {
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
    })
    .unwrap();
    let configuration = PhysicalRecoveryStaticConfiguration::current();
    let authority = PhysicalRecoveryPlatformAuthority::acquire(
        root.to_path_buf(),
        configuration.clone(),
        limits,
    )
    .unwrap();
    let profile = authority.qualified_backend_profile().clone();
    PhysicalRecoveryOpenRequest::declare(
        root.to_path_buf(),
        configuration,
        profile,
        limits,
        authority,
    )
}

pub(crate) fn configuration() -> (
    AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    worth_store::physical_runtime::AdmittedRecordAccessPolicy,
) {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(format)
        .unwrap();
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    (format, placement, access)
}

pub(crate) fn media(root: &Path) -> MediaOwnedPhysicalRuntime {
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("physical media must admit");
    };
    media
}

pub(crate) fn durability(
    media: &MediaOwnedPhysicalRuntime,
) -> worth_store::physical_runtime::AdmittedPhysicalDurabilityPolicy {
    let TransitionOutcome::Success(policy) = PhysicalDurabilityDeclaration::builder()
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
        panic!("durability policy must admit");
    };
    policy
}

pub(crate) fn initialize(root: &Path) -> ServingPhysicalRuntime {
    let (format, placement, access) = configuration();
    let media = media(root);
    let durability = durability(&media);
    let TransitionOutcome::Success(serving) = media
        .initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, durability,
        ))
        .into_raw()
    else {
        panic!("store initialization must admit");
    };
    serving
}

pub(crate) fn open(root: &Path) -> ServingPhysicalRuntime {
    let (format, _, access) = configuration();
    let media = media(root);
    let durability = durability(&media);
    let TransitionOutcome::Success(serving) = media
        .open_record_store(PhysicalRecordOpen::new(format, access, durability))
        .into_raw()
    else {
        panic!("store open must admit");
    };
    serving
}

pub(crate) fn append(
    serving: &ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: [u8; 32],
    payload: &[u8],
) -> worth_store::physical_runtime::CompletedPhysicalMutation {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                RecordAppendBatch::try_from_iter([payload]).unwrap(),
                placement,
                PhysicalMutationRequest::platform_durable(
                    key,
                    PhysicalMutationDeadline::after_milliseconds(1_000).unwrap(),
                ),
            )
            .into_raw()
    else {
        panic!("append must prepare");
    };
    let PhysicalMutationOutcome::Completed(completed) = prepared.execute() else {
        panic!("append must complete");
    };
    completed
}

pub(crate) fn scan(
    serving: &ServingPhysicalRuntime,
) -> Vec<(worth_store::physical_runtime::PhysicalRecordId, Vec<u8>)> {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(RecordScanRequest::from_start().with_batch_limit(RecordCountLimit::new(2).unwrap()))
        .unwrap();
    let mut scratch = vec![0; 128_000];
    let mut records = Vec::new();
    while let RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
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

pub(crate) fn read_record(
    serving: &ServingPhysicalRuntime,
    record: worth_store::physical_runtime::PhysicalRecordId,
) -> Vec<u8> {
    let reader = serving.records().unwrap();
    let mut session = reader
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(EXTENT_BYTES as u32).unwrap()),
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
