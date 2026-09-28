use super::*;
use worth_store::physical_runtime::{
    PhysicalExtentCopyResolutionProgress, PhysicalMutationOutcome,
    PhysicalMutationPreparationDenial, PhysicalMutationPreparationSuccess, RecordAppendBatch,
    RecordAppendDenial,
};
#[path = "../../../worth-store/tests/physical_record_journeys/durability_admission/independent_wal_oracle/copy_intent.rs"]
mod independent_copy_intent;

const CHILD: &str = "pre_final::ready_copy_before_final_child";
const MARKER: &str = "C11_READY_COPY_CRASH_MARKER";
const SEAM: &str = "C11_COPY_CRASH_SEAM";

#[test]
fn killed_ready_copy_retains_exact_private_destination_after_recovery() {
    verify_killed_copy("ready");
}

#[test]
fn killed_partial_copy_retains_exact_private_destination_after_recovery() {
    verify_killed_copy("partial");
}

fn verify_killed_copy(seam: &str) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    kill_copy(directory.path(), seam);
    let baseline = std::fs::read(directory.path().join("baseline-catalog")).unwrap();
    assert_eq!(
        std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap(),
        baseline
    );
    let intents = independent_copy_intent::produced_copy_intents(&root);
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
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([214; 32]))
        .unwrap();
    let ordinary = vec![52; EXTENT_BYTES];
    let attempt = submission.prepare_durable_append(
        RecordAppendBatch::try_from_iter([ordinary.as_slice()]).unwrap(),
        placement,
        PhysicalMutationRequest::platform_durable(
            key,
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        ),
    );
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        attempt.into_raw()
    else {
        panic!("ordinary append must prepare outside the WAL-held copy destination")
    };
    let PhysicalMutationOutcome::Completed(completed) = prepared.execute() else {
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
        PhysicalExtentCopyResolutionProgress::AwaitingCheckpoint { .. }
    ));
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([215; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(checkpoint) = serving.checkpoints().start(request).into_raw()
    else {
        panic!("copy cancellation checkpoint did not admit")
    };
    let PhysicalCheckpointOutcome::Completed(checkpoint) = checkpoint.wait() else {
        panic!("copy cancellation checkpoint did not complete")
    };
    assert_eq!(
        submission.finalize_extent_copy(&checkpoint).unwrap(),
        PhysicalExtentCopyResolutionProgress::Resolved
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

#[test]
fn oversized_unpublished_copy_arena_cannot_become_a_private_claim() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    kill_copy(directory.path(), "ready");
    let intents = independent_copy_intent::produced_copy_intents(&root);
    assert_eq!(intents.len(), 1);
    let destination = intents[0].destination;
    let staged = root.join(format!(
        "families/records/arenas/arena-{:016x}.data",
        destination[0]
    ));
    std::fs::OpenOptions::new()
        .write(true)
        .open(&staged)
        .unwrap()
        .set_len(destination[1] + destination[2] + 1)
        .unwrap();
    let serving = open(&root);
    assert!(!serving.certification_holds_recovered_copy_destination(
        destination[0],
        destination[1],
        destination[2]
    ));
    let (_, placement, _) = configuration();
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([217; 32]))
        .unwrap();
    let outcome = submission.prepare_durable_append(
        RecordAppendBatch::try_from_iter([b"must-deny".as_slice()]).unwrap(),
        placement,
        PhysicalMutationRequest::platform_durable(
            key,
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        ),
    );
    assert!(matches!(
        outcome.into_raw(),
        TransitionOutcome::Denied(PhysicalMutationPreparationDenial::RecordAppend(
            RecordAppendDenial::ServingRequiresInspection
        ))
    ));
    serving.close();
}

#[test]
#[ignore = "parent kills a durable partial or ready copy before final WAL publication"]
fn ready_copy_before_final_child() {
    let directory = PathBuf::from(std::env::var_os(MARKER).unwrap());
    let seam = std::env::var(SEAM).unwrap();
    let root = directory.join("store");
    let serving = initialize(&root);
    let (_, placement, _) = configuration();
    let completed = append(&serving, placement, [211; 32], &vec![71; EXTENT_BYTES]);
    let persisted = completed.persisted_records()[0];
    checkpoint_source(&serving);
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([212; 32]))
        .unwrap();
    let request = PhysicalMutationRequest::platform_durable(
        key,
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    );
    let Ok(mut progress) = submission
        .certification_begin_selected_extent_copy(placement, request, persisted)
        .unwrap()
    else {
        panic!("selected extent copy did not admit")
    };
    for _ in 0..128 {
        if (seam == "ready" && progress.phase == PhysicalExtentCopyPhase::ReadyForAdoption)
            || (seam == "partial"
                && progress.phase == PhysicalExtentCopyPhase::Copying
                && progress.completed_payload_bytes > 0)
        {
            break;
        }
        progress = submission.advance_extent_copy().unwrap();
    }
    assert!(
        (seam == "ready" && progress.phase == PhysicalExtentCopyPhase::ReadyForAdoption)
            || (seam == "partial"
                && progress.phase == PhysicalExtentCopyPhase::Copying
                && progress.completed_payload_bytes > 0
                && progress.completed_payload_bytes < EXTENT_BYTES as u64)
    );
    std::fs::write(
        directory.join("baseline-catalog"),
        std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap(),
    )
    .unwrap();
    std::fs::write(directory.join("reached"), b"copy-unpublished").unwrap();
    loop {
        thread::park();
    }
}

fn kill_copy(directory: &Path, seam: &str) {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD, "--ignored", "--nocapture"])
        .env(MARKER, directory)
        .env(SEAM, seam)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    while !directory.join("reached").exists() {
        if child.try_wait().unwrap().is_some() || Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "ready-copy seam not reached: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    child.kill().unwrap();
    child.wait().unwrap();
}
