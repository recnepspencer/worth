pub(crate) use {std::process::Command, std::time::Duration, worth_store_testkit_recovery_runtime::support::EXTENT_BYTES, std::time::Instant, std::path::Path, std::path::PathBuf, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalExtentCopyPhase, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationRequest, std::process::Stdio, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::support::append, crate::c11_arena_copy_crash::checkpoint_source, worth_store_testkit_recovery_runtime::support::configuration, worth_store_testkit_recovery_runtime::support::generation, worth_store_testkit_recovery_runtime::support::initialize, crate::c11_arena_copy_crash::observation::observe_copy_media, worth_store_testkit_recovery_runtime::support::open, crate::c11_arena_copy_crash::recover, worth_store_testkit_recovery_runtime::support::scan, crate::c11_arena_copy_crash::selected_extent, std::thread};
pub(crate) use {worth_store::physical_runtime::PhysicalExtentCopyResolutionProgress, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationDenial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordAppendDenial};


pub(crate) const CHILD: &str = "pre_final::ready_copy_before_final_child";
pub(crate) const MARKER: &str = "C11_READY_COPY_CRASH_MARKER";
pub(crate) const SEAM: &str = "C11_COPY_CRASH_SEAM";

pub(crate) fn verify_killed_copy(seam: &str) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    kill_copy(directory.path(), seam);
    let baseline = std::fs::read(directory.path().join("baseline-catalog")).unwrap();
    assert_eq!(
        std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap(),
        baseline
    );
    let intents = crate::pre_final::independent_copy_intent::produced_copy_intents(&root);
    assert_eq!(intents.len(), 1);
    let destination = intents[0].destination;
    assert_eq!(destination[1], 0);
    assert!(root
        .join(format!(
            "families/records/arenas/arena-{:016x}.data",
            destination[0]
        ))
        .exists());
    observe_copy_media(&root, directory.path(), "killed");
    let recovered = recover(&root);
    assert_eq!(
        recovered
            .selected_sources()
            .root()
            .selected()
            .selector()
            .root_generation(),
        generation(&baseline),
        "an unfinished copy cannot publish its destination"
    );
    drop(recovered);
    let serving = open(&root);
    assert!(serving.certification_holds_recovered_copy_destination(
        destination[0],
        destination[1],
        destination[2]
    ));
    let (_, placement, _) = configuration();
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([214; 32]))
        .unwrap();
    let ordinary = vec![52; EXTENT_BYTES];
    let attempt = submission.prepare_durable_append(
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([ordinary.as_slice()]).unwrap(),
        placement,
        worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
            key,
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        ),
    );
    let worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        attempt.into_raw()
    else {
        panic!("ordinary append must prepare outside the WAL-held copy destination")
    };
    let worth_store::physical_runtime::PhysicalMutationOutcome::Completed(completed) = prepared.execute() else {
        panic!("ordinary append preparation did not complete")
    };
    let ordinary_record = completed.persisted_records()[0];
    assert!(serving.certification_holds_recovered_copy_destination(
        destination[0],
        destination[1],
        destination[2]
    ));
    assert!(matches!(
        submission.cancel_extent_copy().unwrap(),
        worth_store::physical_runtime::PhysicalExtentCopyResolutionProgress::AwaitingCheckpoint { .. }
    ));
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([215; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(checkpoint) = serving.checkpoints().start(request).into_raw()
    else {
        panic!("copy cancellation checkpoint did not admit")
    };
    let worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(checkpoint) = checkpoint.wait() else {
        panic!("copy cancellation checkpoint did not complete")
    };
    assert_eq!(
        submission.finalize_extent_copy(&checkpoint).unwrap(),
        worth_store::physical_runtime::PhysicalExtentCopyResolutionProgress::Resolved
    );
    assert!(!serving.certification_holds_recovered_copy_destination(
        destination[0],
        destination[1],
        destination[2]
    ));
    append(&serving, placement, [216; 32], &ordinary);
    let rows = scan(&serving);
    assert_eq!(
        rows.iter()
            .filter(|(_, bytes)| bytes.len() == EXTENT_BYTES && bytes.iter().all(|&byte| byte == 71))
            .count(),
        1,
        "cancellation preserves the original source payload exactly once"
    );
    assert_eq!(
        rows.iter()
            .filter(|(_, bytes)| bytes.len() == EXTENT_BYTES && bytes.iter().all(|&byte| byte == 52))
            .count(),
        2,
        "both ordinary appends become readable exactly once"
    );
    serving.close();
    let after = recover(&root);
    let placed = selected_extent(&after, ordinary_record).arena_range();
    assert!(
        placed.arena().get() != destination[0]
            || placed.end() <= destination[1]
            || placed.offset() >= destination[1] + destination[2],
        "ordinary append cannot take the WAL-held destination range"
    );
}

pub(crate) fn kill_copy(directory: &Path, seam: &str) {
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD, "--ignored", "--nocapture"])
        .env(MARKER, directory)
        .env(SEAM, seam)
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
                "ready-copy seam not reached: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
}