#[allow(unused_imports)]
use crate::store_recovery_runtime_context::process_context::*;

use worth_store_testkit_recovery_runtime::phase_three_support::recovery_request_with_limits;
use {
    worth_store_test_support::harness::physical_residency::canonical_durable_wal_attempt_without_execution,
    worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment,
};

use crate::assertions::process_context::assert_expected_posture;

#[test]
#[ignore = "launched by the Phase 7 process-boundary parent"]
fn writer_process() {
    let marker = required_path(CHILD_ROOT);
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery_with_wal_segment_bytes(
        "c8-phase7-writer",
        std::num::NonZeroU64::new(24 * 1024).unwrap(),
    )
    .unwrap();
    let retained_root = world.retained_root();
    let mode = required_text(WRITER_MODE);
    let mut preserved_wal = std::collections::BTreeMap::new();
    canonical_physical_mutation_acknowledgment(&world, [0x71; 32], b"before-checkpoint");
    if mode == "multiple-settled" {
        preserve_wal(&world, &mut preserved_wal);
        for ordinal in 0..10_u8 {
            let mut key = [0x70; 32];
            key[0] = ordinal;
            let payload = vec![ordinal; 4 * 1024];
            canonical_physical_mutation_acknowledgment(&world, key, &payload);
            preserve_wal(&world, &mut preserved_wal);
        }
    }
    publish_checkpoint(&world);
    match mode.as_str() {
        "settled" | "multiple-settled" => {
            canonical_physical_mutation_acknowledgment(
                &world,
                [0x72; 32],
                b"after-checkpoint-settled",
            );
        }
        "unresolved" => {
            canonical_durable_wal_attempt_without_execution(
                &world,
                [0x73; 32],
                b"after-checkpoint-unresolved",
            );
        }
        mode => panic!("unsupported writer mode: {mode}"),
    };
    drop(world);
    let root = retained_root.persist();
    restore_preserved_wal(&root, preserved_wal);
    assert!(
        wal_files(&root).len() >= 2,
        "ordinary writer must rotate WAL"
    );
    std::fs::write(marker, root.to_string_lossy().as_bytes()).unwrap();
}

#[test]
#[ignore = "launched by the Phase 7 process-boundary parent"]
fn cleanup_process() {
    let root = required_path(CHILD_ROOT);
    let mut artifacts = crate::artifact_snapshot::process_context::ArtifactSnapshot::capture(&root);
    let expected = required_text(EXPECTED_WAL);
    let cleanup_bytes = required_text(CLEANUP_BYTES).parse::<u64>().unwrap();
    let cleanup_candidates = required_text(CLEANUP_CANDIDATES).parse::<u64>().unwrap();
    let expected_deferred = required_text(EXPECTED_DEFERRED).parse::<u64>().unwrap();
    let limits = cleanup_limits(cleanup_bytes, cleanup_candidates);
    let request = recovery_request_with_limits(&root, limits);
    let reopened = request
        .admit()
        .unwrap()
        .discover()
        .unwrap()
        .select()
        .unwrap()
        .plan()
        .unwrap()
        .stage()
        .unwrap()
        .publish()
        .unwrap()
        .reopen()
        .unwrap();
    artifacts.include_recovery_created(
        &root,
        reopened.publication_expectation().created_artifacts(),
    );
    let outcome = match std::env::var(CANCEL_AFTER).ok() {
        Some(cancel_after) => {
            let cancellation = if cancel_after == "0" {
                reopened.cleanup_cancellation_before_first().unwrap()
            } else {
                reopened
                    .cleanup_cancellation_after_removal(cancel_after.parse::<u64>().unwrap() - 1)
                    .unwrap()
            };
            reopened.finish_with_cleanup_cancellation(cancellation)
        }
        None => reopened.finish(),
    };
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("cleanup debt must not invalidate recovered success")
    };
    let posture = handoff.cleanup_posture();
    let evidence = posture.evidence();
    let expected_identity =
        worth_store::physical_runtime::recovery_wal::WalSegmentArtifactIdentity::parse(&expected)
            .unwrap();
    let disposition = evidence
        .dispositions()
        .iter()
        .find(|disposition| {
            disposition.target()
                == &worth_store_recovery_runtime::RecoveryCleanupTarget::Wal(expected_identity)
        })
        .expect("checkpoint-covered artifact has one disposition");
    artifacts.assert_reconciled(&root, evidence);
    assert_eq!(evidence.counters().eligible_after_cleanup, 0);
    assert_expected_posture(
        posture,
        disposition,
        expected_identity,
        expected_deferred,
        cleanup_bytes
            + std::fs::metadata(root.join("families").join("checkpoint.current"))
                .unwrap()
                .len(),
    );
}
