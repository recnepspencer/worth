




pub(crate) use {crate::c10_phase_six_span_crash::c10_crash_evidence::ordinary_limits, crate::c10_phase_six_span_crash::c10_crash_evidence::segment_snapshot, crate::c10_phase_six_span_crash::c10_crash_evidence::wal_contains_rewrite};
pub(crate) use {std::path::Path, std::path::PathBuf};
pub(crate) use {std::process::Command, std::process::Stdio};
pub(crate) use {std::thread};
pub(crate) use {std::time::Duration, std::time::Instant};

pub(crate) use {worth_store_testkit_recovery_runtime::phase_three_support::recovery_request_with_limits};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint};
pub(crate) use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest};
pub(crate) use {worth_store_physical_format::PersistedRecordIdentity};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};
pub(crate) use {worth_store_test_support::harness::physical_residency::canonical_physical_batch_acknowledgment, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};

pub(crate) const MARKER: &str = "C10_PHASE6_SPAN_MARKER";
pub(crate) const CHECKPOINT: &str = "C10_PHASE6_SPAN_CHECKPOINT";
pub(crate) const PUBLISHED: &str = "C10_PHASE6_SPAN_PUBLISHED";
pub(crate) const PAGES: &str = "C10_PHASE6_SPAN_PAGES";

pub(crate) fn checkpoint_source(world: &PhysicalResidencyStoreWorld) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x93; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let handle = match world.serving().checkpoints().start(request).into_raw() {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        _ => panic!("span source checkpoint was not admitted"),
    };
    match handle.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_) => {}
        _ => panic!("span source checkpoint must complete"),
    }
}

pub(crate) fn rewrite_checkpoint(name: &str) -> CertificationPhysicalMutationCheckpoint {
    match name {
        "after-wal" => worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterWalDurability,
        "after-data" => worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterDataSettlement,
        "after-replace" => worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterRootReplacement,
        other => panic!("unknown span checkpoint {other}"),
    }
}

pub(crate) fn prepare_span(
    world: &PhysicalResidencyStoreWorld,
    pages: u32,
) -> worth_store::physical_runtime::PreparedPhysicalMutation {
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0x92; 32]))
        .unwrap();
    match submission
        .rewrite_selected_inline_pages(
            world.placement(),
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ),
            pages,
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        worth_proof::TransitionOutcome::Success(_) => panic!("span rewrite did not stay prepared"),
        worth_proof::TransitionOutcome::Denied(_) => panic!("span rewrite denied"),
        worth_proof::TransitionOutcome::Deferred(_) => panic!("span rewrite deferred"),
        worth_proof::TransitionOutcome::Stale(_) => panic!("span rewrite stale"),
        worth_proof::TransitionOutcome::RebindRequired(_) => panic!("span rewrite rebind"),
        worth_proof::TransitionOutcome::Failed(_) => panic!("span rewrite failed"),
    }
}

pub(crate) fn kill_span(published: u32, pages: u32, checkpoint: &str) -> (tempfile::TempDir, PathBuf) {
    let parent = tempfile::tempdir().unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "c10_span_child_parks",
            "--ignored",
            "--nocapture",
        ])
        .env(MARKER, parent.path())
        .env(CHECKPOINT, checkpoint)
        .env(PUBLISHED, published.to_string())
        .env(PAGES, pages.to_string())
        .env("TMP", parent.path())
        .env("TEMP", parent.path())
        .env("TMPDIR", parent.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("launch phase 6 span child");
    let reached = parent.path().join("reached");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while !reached.exists() {
        if std::time::Instant::now() > deadline {
            let _ = child.kill();
            panic!("span child did not reach {checkpoint} pages={pages}");
        }
        if let Some(status) = child.try_wait().unwrap() {
            let output = child.wait_with_output().unwrap();
            panic!(
                "span child exited early: {status}\nstdout:\n{}\nstderr:\n{}",
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

pub(crate) fn release(
    marker: &Path,
    root: &Path,
    published: u32,
    checkpoint: &str,
    pages: u32,
    pass: &str,
) -> (Vec<Vec<u8>>, u64) {
    let handoff = recover(root, checkpoint, pages, pass);
    let identities = load_identities(marker);
    assert_eq!(identities.len(), published as usize);
    let payload = identities
        .into_iter()
        .map(|identity| worth_store_testkit_recovery_runtime::c10_phase_five_read::selected_payload(root, &handoff, identity))
        .collect();
    let generation = worth_store_testkit_recovery_runtime::c10_phase_five_read::root_generation(&handoff);
    drop(handoff);
    (payload, generation)
}

pub(crate) fn recover(
    root: &Path,
    checkpoint: &str,
    pages: u32,
    pass: &str,
) -> worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff {
    let outcome =
        worth_store_recovery_runtime::WorthStoreRecovery::recover(recovery_request_with_limits(root, ordinary_limits()));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("sealed span {checkpoint} pages={pages} {pass} must recover: {outcome:?}");
    };
    *handoff
}

pub(crate) fn store_identities(marker: &Path, records: &[PersistedRecordIdentity]) {
    let mut bytes = Vec::new();
    for record in records {
        bytes.extend_from_slice(&record.allocation_epoch());
        bytes.extend_from_slice(&record.ordinal().to_le_bytes());
    }
    std::fs::write(marker.join("span-ids.bin"), bytes).unwrap();
}

pub(crate) fn load_identities(marker: &Path) -> Vec<PersistedRecordIdentity> {
    let bytes = std::fs::read(marker.join("span-ids.bin")).unwrap();
    bytes
        .chunks(24)
        .map(|chunk| {
            let mut epoch = [0; 16];
            epoch.copy_from_slice(&chunk[..16]);
            let ordinal = u64::from_le_bytes(chunk[16..24].try_into().unwrap());
            worth_store_physical_format::PersistedRecordIdentity::new(epoch, ordinal).unwrap()
        })
        .collect()
}