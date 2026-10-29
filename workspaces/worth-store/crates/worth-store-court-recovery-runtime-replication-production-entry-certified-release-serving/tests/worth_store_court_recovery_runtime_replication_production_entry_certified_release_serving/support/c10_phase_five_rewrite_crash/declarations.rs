




pub(crate) use {crate::c10_phase_five_rewrite_crash::c10_crash_evidence::directory_snapshot, crate::c10_phase_five_rewrite_crash::c10_crash_evidence::ordinary_limits, crate::c10_phase_five_rewrite_crash::c10_crash_evidence::prepare_rewrite, crate::c10_phase_five_rewrite_crash::c10_crash_evidence::segment_snapshot, crate::c10_phase_five_rewrite_crash::c10_crash_evidence::wal_contains_rewrite};
pub(crate) use {std::path::Path, std::path::PathBuf};
pub(crate) use {std::process::Command, std::process::Stdio};
pub(crate) use {std::thread};
pub(crate) use {std::time::Duration, std::time::Instant};

pub(crate) use {worth_store_testkit_recovery_runtime::phase_three_support::recovery_request_with_limits};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint};
pub(crate) use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};
pub(crate) use {worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};

pub(crate) const CHILD_MARKER: &str = "C10_PHASE5_REWRITE_MARKER";
pub(crate) const CHILD_CHECKPOINT: &str = "C10_PHASE5_REWRITE_CHECKPOINT";

pub(crate) fn reseal_wrong_page(path: &Path) {
    use {worth_store_physical_format::decode_data_frame_page_lsn, worth_store_physical_format::DurableFrameKind, worth_store_physical_format::DURABLE_FRAME_HEADER_BYTES};
    let mut page = std::fs::read(path).unwrap();
    let lsn = decode_data_frame_page_lsn(&page, DurableFrameKind::InlinePage).unwrap();
    page[400] ^= 0xff;
    let mut covered = Vec::with_capacity(page.len() - 4);
    covered.extend_from_slice(&page[..DURABLE_FRAME_HEADER_BYTES - 4]);
    covered.extend_from_slice(&page[DURABLE_FRAME_HEADER_BYTES..]);
    let checksum = worth_store_physical_format::durable_artifact_checksum(&covered);
    page[DURABLE_FRAME_HEADER_BYTES - 4..DURABLE_FRAME_HEADER_BYTES]
        .copy_from_slice(&checksum.to_le_bytes());
    assert_eq!(
        decode_data_frame_page_lsn(&page, DurableFrameKind::InlinePage).unwrap(),
        lsn
    );
    std::fs::write(path, page).unwrap();
}

pub(crate) fn rewrite_checkpoint(name: &str) -> CertificationPhysicalMutationCheckpoint {
    match name {
        "before-wal" => worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeWalAppend,
        "after-wal" => worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterWalDurability,
        "after-data" => worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterDataSettlement,
        "during-root" => worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::DuringRootPublication,
        "after-replace" => worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterRootReplacement,
        "before-ack" => worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeTerminalFinalization,
        other => panic!("unknown rewrite checkpoint {other}"),
    }
}

pub(crate) fn checkpoint_source(world: &PhysicalResidencyStoreWorld) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x83; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let handle = match world.serving().checkpoints().start(request).into_raw() {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        _ => panic!("source checkpoint was not admitted"),
    };
    match handle.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_) => {}
        _ => panic!("source checkpoint must complete"),
    }
}

pub(crate) fn kill_child(checkpoint: &str) -> (tempfile::TempDir, PathBuf) {
    let parent = tempfile::tempdir().unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "c10_rewrite_child_parks_at_checkpoint",
            "--ignored",
            "--nocapture",
        ])
        .env(CHILD_MARKER, parent.path())
        .env(CHILD_CHECKPOINT, checkpoint)
        .env("TMP", parent.path())
        .env("TEMP", parent.path())
        .env("TMPDIR", parent.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("launch phase 5 rewrite child");
    let reached = parent.path().join("reached");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while !reached.exists() {
        if std::time::Instant::now() > deadline {
            let _ = child.kill();
            panic!("rewrite child did not reach {checkpoint}");
        }
        if let Some(status) = child.try_wait().unwrap() {
            let output = child.wait_with_output().unwrap();
            panic!(
                "rewrite child exited early: {status}\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    child.kill().unwrap();
    let _ = child.wait();
    let root = std::path::PathBuf::from(std::fs::read_to_string(parent.path().join("root-path.txt")).unwrap());
    (parent, root)
}

pub(crate) fn release_after_read(marker: &Path, root: &Path) -> (Vec<(u64, u64, u64, u64)>, Vec<u8>, u64) {
    let handoff = recover(root);
    let facts = (
        retirements(&handoff),
        source_payload(marker, root, &handoff),
        worth_store_testkit_recovery_runtime::c10_phase_five_read::root_generation(&handoff),
    );
    drop(handoff);
    facts
}

pub(crate) fn recover(root: &Path) -> worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff {
    let outcome =
        worth_store_recovery_runtime::WorthStoreRecovery::recover(recovery_request_with_limits(root, ordinary_limits()));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("killed rewrite must recover");
    };
    *handoff
}

pub(crate) fn retirements(
    handoff: &worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff,
) -> Vec<(u64, u64, u64, u64)> {
    handoff
        .freshness_sample()
        .retirements()
        .iter()
        .map(|retirement| {
            (
                retirement.source_root(),
                retirement
                    .artifact()
                    .segment()
                    .expect("an inline rewrite retires a segment generation"),
                retirement.artifact().generation(),
                retirement.bytes(),
            )
        })
        .collect()
}

pub(crate) fn source_payload(
    marker: &Path,
    root: &Path,
    handoff: &worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff,
) -> Vec<u8> {
    worth_store_testkit_recovery_runtime::c10_phase_five_read::selected_payload(root, handoff, worth_store_testkit_recovery_runtime::c10_phase_five_read::load_identity(marker))
}