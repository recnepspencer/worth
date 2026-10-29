


pub(crate) use {crate::c10_phase_five_retirement_crash::c10_crash_evidence::ordinary_limits, crate::c10_phase_five_retirement_crash::c10_crash_evidence::prepare_rewrite, crate::c10_phase_five_retirement_crash::c10_crash_evidence::segment_snapshot, crate::c10_phase_five_retirement_crash::c10_crash_evidence::wal_contains_rewrite};
pub(crate) use {std::path::Path, std::path::PathBuf};
pub(crate) use {std::process::Command, std::process::Stdio};
pub(crate) use {std::thread};
pub(crate) use {std::time::Duration, std::time::Instant};

pub(crate) use {worth_store_testkit_recovery_runtime::phase_three_support::recovery_request_with_limits};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint};
pub(crate) use {worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause, worth_store::physical_runtime::PhysicalMutationRequest};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};
pub(crate) use {worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};

pub(crate) const CHILD_MARKER: &str = "C10_PHASE5_RETIREMENT_MARKER";
pub(crate) const CHILD_SEAM: &str = "C10_PHASE5_RETIREMENT_SEAM";

pub(crate) fn park_retirement_until_killed(world: &PhysicalResidencyStoreWorld, marker: &Path, seam: u8) {
    let arrived = world.serving().certification_arm_retirement_kill(seam);
    let marker = marker.to_path_buf();
    std::thread::spawn(move || {
        while !arrived.load(std::sync::atomic::Ordering::Acquire) {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        std::fs::write(marker.join("reached"), b"1").unwrap();
        loop {
            std::thread::park();
        }
    });
    let returned = world.serving().retire_displaced_segment();
    panic!("armed retirement seam {seam} returned before the kill: {returned:?}");
}

pub(crate) fn segment_name(segment: u64, generation: u64) -> String {
    format!("segment-{segment:016x}-{generation:016x}.pages")
}

pub(crate) fn segment_names(root: &Path) -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root.join("families/records/segments")) {
        for entry in entries.flatten() {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names
}

pub(crate) fn park_behind_durable_append(world: &PhysicalResidencyStoreWorld) {
    checkpoint_source(world);
    let gate = world.serving().certification_pause_physical_mutation_at(
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterWalDurability,
    );
    let append = prepare_append(world, [0x73; 32], b"wal-durable-append").start();
    assert!(gate.await_arrival(), "append did not reach WAL durability");
    match prepare_rewrite(world, [0x74; 32]).execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            assert_eq!(
                fate.cause(),
                worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause::ScopeConflict
            );
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) => panic!("rewrite passed a pending append"),
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("rewrite reached an effect: {:?}", fate.stage())
        }
    }
    std::mem::forget(append);
    std::mem::forget(gate);
}

pub(crate) fn checkpoint_source(world: &PhysicalResidencyStoreWorld) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x75; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
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

pub(crate) fn prepare_append(
    world: &PhysicalResidencyStoreWorld,
    material: [u8; 32],
    payload: &'static [u8],
) -> worth_store::physical_runtime::PreparedPhysicalMutation {
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([payload]).unwrap(),
            world.placement(),
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(1_000).unwrap(),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("append preparation must succeed"),
    }
}

pub(crate) fn kill_and_recover(seam: &str) -> Vec<(u64, u64, u64, u64)> {
    let (parent, root) = kill_child(seam);
    let first = recovered_retirements(&root);
    let second = recovered_retirements(&root);
    assert_eq!(first, second, "{seam} second reopen changed the obligation");
    drop(parent);
    first
}

pub(crate) fn kill_child(seam: &str) -> (tempfile::TempDir, PathBuf) {
    let parent = tempfile::tempdir().unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "c10_retirement_child_parks_at_seam",
            "--ignored",
            "--nocapture",
        ])
        .env(CHILD_MARKER, parent.path())
        .env(CHILD_SEAM, seam)
        .env("TMP", parent.path())
        .env("TEMP", parent.path())
        .env("TMPDIR", parent.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("launch phase 5 retirement child");
    let reached = parent.path().join("reached");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while !reached.exists() {
        if std::time::Instant::now() > deadline {
            let _ = child.kill();
            panic!("child did not reach the {seam} seam");
        }
        if let Some(status) = child.try_wait().unwrap() {
            let output = child.wait_with_output().unwrap();
            panic!(
                "child exited before {seam}: {status}\nstdout:\n{}\nstderr:\n{}",
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

pub(crate) fn publish_rewrite(world: &PhysicalResidencyStoreWorld) {
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0x72; 32]))
        .unwrap();
    let prepared = match submission
        .rewrite_selected_inline_segment(
            world.placement(),
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(1_000).unwrap(),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        worth_proof::TransitionOutcome::Success(_) => panic!("rewrite preparation must return a mutation"),
        worth_proof::TransitionOutcome::Denied(_) => panic!("rewrite preparation was denied"),
        worth_proof::TransitionOutcome::Deferred(_) => panic!("rewrite preparation was deferred"),
        worth_proof::TransitionOutcome::Stale(_) => panic!("rewrite preparation was stale"),
        worth_proof::TransitionOutcome::RebindRequired(_) => panic!("rewrite preparation required rebind"),
        worth_proof::TransitionOutcome::Failed(_) => panic!("rewrite preparation failed"),
    };
    match prepared.execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) => {}
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("rewrite must publish: {:?}", fate.cause())
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("rewrite must settle: {:?}", fate.stage())
        }
    }
}

pub(crate) fn recovered_retirements(root: &Path) -> Vec<(u64, u64, u64, u64)> {
    let outcome =
        worth_store_recovery_runtime::WorthStoreRecovery::recover(recovery_request_with_limits(root, ordinary_limits()));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("killed retirement must recover: {outcome:?}")
    };
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