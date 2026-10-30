pub(crate) use std::thread;
pub(crate) use {
    crate::c10_phase_five_mixed_tail::c10_crash_evidence::ordinary_limits,
    crate::c10_phase_five_mixed_tail::c10_crash_evidence::prepare_rewrite,
    crate::c10_phase_five_mixed_tail::c10_crash_evidence::segment_snapshot,
    crate::c10_phase_five_mixed_tail::c10_crash_evidence::wal_contains_rewrite,
};
pub(crate) use {std::path::Path, std::path::PathBuf};
pub(crate) use {std::process::Command, std::process::Stdio};
pub(crate) use {std::time::Duration, std::time::Instant};

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint;
pub(crate) use worth_store_testkit_recovery_runtime::phase_three_support::recovery_request_with_limits;
pub(crate) use {
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause,
    worth_store::physical_runtime::PhysicalMutationRequest,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};
pub(crate) use {
    worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};

pub(crate) const CHILD_MARKER: &str = "C10_PHASE5_MIXED_MARKER";
pub(crate) const CHILD_SEAM: &str = "C10_PHASE5_MIXED_SEAM";

pub(crate) fn finish_append_ahead_of_stale_rewrite(
    world: &PhysicalResidencyStoreWorld,
    marker: &Path,
) {
    let gate = world.serving().certification_pause_physical_mutation_at(
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterWalDurability,
    );
    let append = prepare_append(world, [0x93; 32], b"phase5-member-a").start();
    assert!(gate.await_arrival(), "append did not reach WAL durability");
    match prepare_rewrite(world, [0x95; 32]).execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            assert_eq!(
                fate.cause(),
                worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause::ScopeConflict
            );
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) => {
            panic!("stale rewrite must not pass the pending append")
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("stale rewrite reached an effect")
        }
    }
    gate.release();
    match append.wait() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(done) => {
            worth_store_testkit_recovery_runtime::c10_phase_five_read::store_identity(
                marker,
                done.persisted_records()[0],
            );
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(_)
        | worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("released append must publish")
        }
    }
}

pub(crate) fn rewrite_checkpoint(seam: &str) -> CertificationPhysicalMutationCheckpoint {
    match seam {
        "mixed-before-wal" => worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::BeforeWalAppend,
        "mixed-after-wal" | "competing-then-after-wal" => {
            worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterWalDurability
        }
        "competing-then-after-data" => worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterDataSettlement,
        "competing-then-during-root" => {
            worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::DuringRootPublication
        }
        other => panic!("unknown mixed seam {other}"),
    }
}

pub(crate) fn publish_append(
    world: &PhysicalResidencyStoreWorld,
    marker: &Path,
    material: [u8; 32],
    payload: &'static [u8],
) {
    match prepare_append(world, material, payload).execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(done) => {
            worth_store_testkit_recovery_runtime::c10_phase_five_read::store_identity(
                marker,
                done.persisted_records()[0],
            );
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(_)
        | worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("mixed append must publish")
        }
    }
}

pub(crate) fn checkpoint_source(world: &PhysicalResidencyStoreWorld) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x96; 32]),
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
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(material),
        )
        .unwrap();
    match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([payload]).unwrap(),
            world.placement(),
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(1_000)
                    .unwrap(),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        _ => panic!("append preparation must succeed"),
    }
}

pub(crate) fn kill_child(seam: &str) -> (tempfile::TempDir, PathBuf) {
    let parent = tempfile::tempdir().unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "process::recovery_runtime::c10_phase_five_mixed_tail::mixed_child_parks_at_seam",
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
        .expect("launch phase 5 mixed child");
    let reached = parent.path().join("reached");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while !reached.exists() {
        if std::time::Instant::now() > deadline {
            let _ = child.kill();
            panic!("mixed child did not reach {seam}");
        }
        if let Some(status) = child.try_wait().unwrap() {
            let output = child.wait_with_output().unwrap();
            panic!(
                "mixed child exited early: {status}\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    child.kill().unwrap();
    let _ = child.wait();
    let root = std::path::PathBuf::from(
        std::fs::read_to_string(parent.path().join("root-path.txt")).unwrap(),
    );
    (parent, root)
}

pub(crate) fn recover(root: &Path, marker: &Path) -> (Vec<(u64, u64, u64, u64)>, Vec<u8>) {
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        recovery_request_with_limits(root, ordinary_limits()),
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        if let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) = &outcome {
            panic!(
                "mixed tail blocked {:?} artifact {:?} denial {:?} lsn {:?}",
                blocked.cause(),
                blocked.evidence().artifact,
                blocked.evidence().planning_denial,
                blocked.evidence().lsn
            );
        }
        panic!("mixed tail must recover");
    };
    let payload = worth_store_testkit_recovery_runtime::c10_phase_five_read::selected_payload(
        root,
        &handoff,
        worth_store_testkit_recovery_runtime::c10_phase_five_read::load_identity(marker),
    );
    let retirements = handoff
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
        .collect();
    drop(handoff);
    (retirements, payload)
}
