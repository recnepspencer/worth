

pub(crate) use {std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio, std::thread, std::time::Duration, std::time::Instant};
pub(crate) use {worth_store_testkit_recovery_runtime::support::EXTENT_BYTES, worth_store::physical_runtime::ServingPhysicalRuntime, worth_store_testkit_recovery_runtime::support::append, worth_store_testkit_recovery_runtime::support::configuration, worth_store_testkit_recovery_runtime::support::generation, worth_store_testkit_recovery_runtime::support::initialize, worth_store_testkit_recovery_runtime::support::open, worth_store_testkit_recovery_runtime::support::recovery_request, worth_store_testkit_recovery_runtime::support::scan};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalExtentCopyPhase, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationRequest};
pub(crate) use {worth_store_physical_format::CurrentPhysicalRecordPlacement, worth_store_physical_format::DurableExtentRecordPlacement, worth_store_physical_format::PersistedRecordIdentity};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff, worth_store_recovery_runtime::WorthStoreRecovery};

pub(crate) use {crate::c11_arena_copy_crash::observation::observe_copy_media};





pub(crate) const CHILD: &str = "final_copy_wal_child";
pub(crate) const MARKER: &str = "C11_FINAL_COPY_CRASH_MARKER";

pub(crate) fn checkpoint_source(serving: &worth_store::physical_runtime::ServingPhysicalRuntime) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([204; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("source checkpoint did not admit")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
}

pub(crate) fn kill_after_final_wal(directory: &Path) {
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD, "--ignored", "--nocapture"])
        .env(MARKER, directory)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while !directory.join("reached").exists() {
        if child.try_wait().unwrap().is_some() || std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "final WAL seam not reached: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
}

pub(crate) fn recover(root: &Path) -> RecoveredPhysicalRuntimeHandoff {
    match worth_store_recovery_runtime::WorthStoreRecovery::recover(recovery_request(root)) {
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) => *handoff,
        outcome => panic!("final copy recovery blocked: {outcome:?}"),
    }
}

pub(crate) fn selected_extent(
    handoff: &RecoveredPhysicalRuntimeHandoff,
    record: PersistedRecordIdentity,
) -> DurableExtentRecordPlacement {
    handoff
        .selected_sources()
        .page_facts()
        .placements()
        .iter()
        .find_map(|placement| match placement {
            worth_store_physical_format::CurrentPhysicalRecordPlacement::Extent(extent) if extent.record() == record => {
                Some(*extent)
            }
            _ => None,
        })
        .expect("selected root must route the copied extent")
}

pub(crate) fn save_identity(directory: &Path, record: PersistedRecordIdentity) {
    let mut bytes = record.allocation_epoch().to_vec();
    bytes.extend_from_slice(&record.ordinal().to_le_bytes());
    std::fs::write(directory.join("record-id.bin"), bytes).unwrap();
}

pub(crate) fn load_identity(directory: &Path) -> PersistedRecordIdentity {
    let bytes = std::fs::read(directory.join("record-id.bin")).unwrap();
    let epoch = bytes[..16].try_into().unwrap();
    let ordinal = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
    worth_store_physical_format::PersistedRecordIdentity::new(epoch, ordinal).unwrap()
}