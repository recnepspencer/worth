#[allow(unused_imports)] use crate::c11_arena_copy_crash::pre_final::*;
use {std::process::Command, std::time::Duration, worth_store_testkit_recovery_runtime::support::EXTENT_BYTES, std::time::Instant, std::path::Path, std::path::PathBuf, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalExtentCopyPhase, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationRequest, std::process::Stdio, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::support::append, crate::c11_arena_copy_crash::checkpoint_source, worth_store_testkit_recovery_runtime::support::configuration, worth_store_testkit_recovery_runtime::support::generation, worth_store_testkit_recovery_runtime::support::initialize, crate::c11_arena_copy_crash::observation::observe_copy_media, worth_store_testkit_recovery_runtime::support::open, crate::c11_arena_copy_crash::recover, worth_store_testkit_recovery_runtime::support::scan, crate::c11_arena_copy_crash::selected_extent, std::thread};
use {worth_store::physical_runtime::PhysicalExtentCopyResolutionProgress, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationDenial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordAppendDenial};

#[test]
fn killed_ready_copy_retains_exact_private_destination_after_recovery() {
    verify_killed_copy("ready");
}

#[test]
fn killed_partial_copy_retains_exact_private_destination_after_recovery() {
    verify_killed_copy("partial");
}

#[test]
fn oversized_unpublished_copy_arena_cannot_become_a_private_claim() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    kill_copy(directory.path(), "ready");
    let intents = crate::pre_final::independent_copy_intent::produced_copy_intents(&root);
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
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([217; 32]))
        .unwrap();
    let outcome = submission.prepare_durable_append(
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"must-deny".as_slice()]).unwrap(),
        placement,
        worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
            key,
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        ),
    );
    assert!(matches!(
        outcome.into_raw(),
        worth_proof::TransitionOutcome::Denied(worth_store::physical_runtime::PhysicalMutationPreparationDenial::RecordAppend(
            worth_store::physical_runtime::RecordAppendDenial::ServingRequiresInspection
        ))
    ));
    serving.close();
}

#[test]
#[ignore = "parent kills a durable partial or ready copy before final WAL publication"]
fn ready_copy_before_final_child() {
    let directory = std::path::PathBuf::from(std::env::var_os(MARKER).unwrap());
    let seam = std::env::var(SEAM).unwrap();
    let root = directory.join("store");
    let serving = initialize(&root);
    let (_, placement, _) = configuration();
    let completed = append(&serving, placement, [211; 32], &vec![71; EXTENT_BYTES]);
    let persisted = completed.persisted_records()[0];
    checkpoint_source(&serving);
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([212; 32]))
        .unwrap();
    let request = worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
        key,
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    );
    let Ok(mut progress) = submission
        .certification_begin_selected_extent_copy(placement, request, persisted)
        .unwrap()
    else {
        panic!("selected extent copy did not admit")
    };
    for _ in 0..128 {
        if (seam == "ready" && progress.phase == worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption)
            || (seam == "partial"
                && progress.phase == worth_store::physical_runtime::PhysicalExtentCopyPhase::Copying
                && progress.completed_payload_bytes > 0)
        {
            break;
        }
        progress = submission.advance_extent_copy().unwrap();
    }
    assert!(
        (seam == "ready" && progress.phase == worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption)
            || (seam == "partial"
                && progress.phase == worth_store::physical_runtime::PhysicalExtentCopyPhase::Copying
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
        std::thread::park();
    }
}
