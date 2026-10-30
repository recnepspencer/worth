#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::arena_copy_crash::*;
use {worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, crate::durability_admission::managed_mutation::extent_record_rewrite::independent_arena_observer, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::request, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};
use {std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio, std::thread, std::time::Duration, std::time::Instant};
use {worth_store::physical_runtime::PhysicalExtentCopyPhase};

#[test]
fn partial_copy_bytes_before_final_kill_preserves_private_destination_claim() {
    verify_killed_copy("partial");
}

#[test]
fn fully_copied_unpublished_destination_reopens_with_private_claim() {
    verify_killed_copy("ready");
}

#[test]
#[ignore = "parent terminates this process after a durable copy intent and copied frame"]
fn copy_bytes_before_final_child() {
    let directory = std::path::PathBuf::from(std::env::var_os(DIRECTORY).unwrap());
    let seam = std::env::var(SEAM).unwrap();
    let root = directory.join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let payload = vec![73; 40_000];
    let append = completed(prepare(&serving, placement, [241; 32], &payload).execute());
    let identity = append.persisted_records()[0];
    std::fs::write(
        directory.join("baseline-catalog"),
        std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap(),
    )
    .unwrap();
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([242; 32]),
        )
        .unwrap();
    let Ok(mut progress) = submission
        .certification_begin_selected_extent_copy(placement, request(key), identity)
        .unwrap()
    else {
        panic!("selected copy rejected")
    };
    for _ in 0..128 {
        progress = submission.advance_extent_copy().unwrap();
        if seam == "partial"
            && progress.phase == worth_store::physical_runtime::PhysicalExtentCopyPhase::Copying
            && progress.completed_payload_bytes > 0
        {
            break;
        }
        if seam == "ready"
            && progress.phase
                == worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption
        {
            break;
        }
    }
    match seam.as_str() {
        "partial" => assert!(
            progress.phase == worth_store::physical_runtime::PhysicalExtentCopyPhase::Copying
                && progress.completed_payload_bytes > 0
                && progress.completed_payload_bytes < payload.len() as u64
        ),
        "ready" => assert_eq!(
            progress.phase,
            worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption
        ),
        _ => panic!("unknown crash seam"),
    }
    std::fs::write(directory.join("reached"), b"durable-intent-and-bytes").unwrap();
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
