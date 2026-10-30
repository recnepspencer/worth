#[allow(unused_imports)]
use crate::c10_phase_five_rewrite_crash::*;
use {
    crate::c10_phase_five_rewrite_crash::c10_crash_evidence::directory_snapshot,
    crate::c10_phase_five_rewrite_crash::c10_crash_evidence::ordinary_limits,
    crate::c10_phase_five_rewrite_crash::c10_crash_evidence::prepare_rewrite,
    crate::c10_phase_five_rewrite_crash::c10_crash_evidence::segment_snapshot,
    crate::c10_phase_five_rewrite_crash::c10_crash_evidence::wal_contains_rewrite,
};

use worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment;
use worth_store_testkit_recovery_runtime::phase_three_support::recovery_request_with_limits;

#[test]
fn killed_rewrite_before_wal_leaves_the_checkpointed_source() {
    let (parent, root) = kill_child("before-wal");
    let killed = segment_snapshot(&root);
    assert!(!wal_contains_rewrite(&root));
    let (first_retirements, first_payload, first_generation) =
        release_after_read(parent.path(), &root);
    let published = segment_snapshot(&root);
    let (second_retirements, second_payload, second_generation) =
        release_after_read(parent.path(), &root);
    assert_eq!(first_retirements, second_retirements);
    assert!(first_retirements.is_empty());
    assert_eq!(first_payload, b"phase5-rewrite-source");
    assert_eq!(second_payload, b"phase5-rewrite-source");
    assert_eq!(first_generation, second_generation);
    assert_eq!(killed, published);
    assert_eq!(published, segment_snapshot(&root));
    assert!(!wal_contains_rewrite(&root));
    drop(parent);
}

#[test]
fn killed_wal_durable_rewrite_publishes_one_generation() {
    let (parent, root) = kill_child("after-wal");
    assert!(wal_contains_rewrite(&root));
    let killed = segment_snapshot(&root);
    let (first_retirements, first_payload, first_generation) =
        release_after_read(parent.path(), &root);
    let published = segment_snapshot(&root);
    let (second_retirements, second_payload, second_generation) =
        release_after_read(parent.path(), &root);
    assert_eq!(first_retirements, second_retirements);
    assert_eq!(first_payload, b"phase5-rewrite-source");
    assert_eq!(second_payload, b"phase5-rewrite-source");
    assert_eq!(second_generation, first_generation + 1);
    assert_eq!(published, segment_snapshot(&root));
    assert_ne!(killed, published, "recovery must apply the sealed rewrite");
    assert!(
        published.len() <= killed.len() + 1,
        "recovery published more than one rewritten generation"
    );
    drop(parent);
}

#[test]
fn killed_rewrite_after_candidate_data_does_not_publish_a_second_generation() {
    for seam in ["after-data", "during-root", "after-replace"] {
        let (parent, root) = kill_child(seam);
        assert!(wal_contains_rewrite(&root), "{seam}");
        let killed = segment_snapshot(&root);
        let (first_retirement, _, _) = release_after_read(parent.path(), &root);
        let first = segment_snapshot(&root);
        let (second_retirement, second_payload, _) = release_after_read(parent.path(), &root);
        assert_eq!(first_retirement, second_retirement, "{seam}");
        assert_eq!(second_payload, b"phase5-rewrite-source", "{seam}");
        assert_eq!(first, segment_snapshot(&root), "{seam}");
        assert!(
            first.len() <= killed.len() + 1,
            "{seam} published more than the candidate generation"
        );
        drop(parent);
    }
}

#[test]
fn killed_rewrite_after_namespace_durability_is_not_repeated() {
    let (parent, root) = kill_child("before-ack");
    assert!(wal_contains_rewrite(&root));
    let killed = segment_snapshot(&root);
    let roots = directory_snapshot(&root, "families/records/roots");
    let (first_retirements, _, first_generation) = release_after_read(parent.path(), &root);
    let published = segment_snapshot(&root);
    let published_roots = directory_snapshot(&root, "families/records/roots");
    let (second_retirements, second_payload, second_generation) =
        release_after_read(parent.path(), &root);
    assert_eq!(first_retirements, second_retirements);
    assert_eq!(first_generation, second_generation);
    assert_eq!(second_payload, b"phase5-rewrite-source");
    assert_eq!(killed, published);
    assert_eq!(roots, published_roots);
    assert_eq!(published, segment_snapshot(&root));
    assert_eq!(
        published_roots,
        directory_snapshot(&root, "families/records/roots")
    );
    drop(parent);
}

#[test]
fn inline_rewrite_result_remains_exact_after_its_root_becomes_historical() {
    let (parent, root) = kill_child("after-wal");
    let (_, expected_payload, _) = release_after_read(parent.path(), &root);
    assert_eq!(expected_payload, b"phase5-rewrite-source");
    let result_root = worth_store_testkit_recovery_runtime::historical_rewrite_support::selected_catalog_generation(&root);
    worth_store_testkit_recovery_runtime::historical_rewrite_support::advance_with_ordinary_appends(
        &root, false,
    );
    let selected_before = worth_store_testkit_recovery_runtime::historical_rewrite_support::selected_catalog_generation(&root);
    assert!(
        selected_before >= result_root + 2,
        "the rewrite result root must be neither selected nor retained-previous"
    );
    let segments_before = segment_snapshot(&root);
    let (_, payload, selected_after) = release_after_read(parent.path(), &root);
    assert_eq!(payload, expected_payload);
    assert_eq!(selected_after, selected_before);
    assert_eq!(
        worth_store_testkit_recovery_runtime::historical_rewrite_support::selected_catalog_generation(&root),
        selected_before,
        "fresh recovery must not republish an already historical rewrite"
    );
    assert_eq!(segment_snapshot(&root), segments_before);
}

#[test]
fn published_rewrite_rejects_a_wrong_page_and_a_missing_page() {
    for remove in [false, true] {
        let (parent, root) = kill_child("after-wal");
        let killed = segment_snapshot(&root);
        let (_, payload, _) = release_after_read(parent.path(), &root);
        assert_eq!(payload, b"phase5-rewrite-source");
        let created = segment_snapshot(&root)
            .into_iter()
            .find(|(name, _)| killed.iter().all(|(old, _)| old != name))
            .expect("recovery did not publish a destination segment");
        let path = root.join("families/records/segments").join(created.0);
        if remove {
            std::fs::remove_file(&path).unwrap();
        } else {
            reseal_wrong_page(&path);
        }
        let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
            recovery_request_with_limits(&root, ordinary_limits()),
        );
        assert!(matches!(
            outcome,
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(_)
        ));
        drop(parent);
    }
}

#[test]
#[ignore = "launched by the phase 5 rewrite parent"]
fn rewrite_child_parks_at_checkpoint() {
    let marker =
        std::path::PathBuf::from(std::env::var_os(CHILD_MARKER).expect("phase 5 rewrite marker"));
    let checkpoint = std::env::var(CHILD_CHECKPOINT).expect("phase 5 rewrite checkpoint");
    let world =
        worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("c10-phase5-rewrite-kill").unwrap();
    std::fs::write(
        marker.join("root-path.txt"),
        world.root().to_string_lossy().as_bytes(),
    )
    .unwrap();
    let record =
        canonical_physical_mutation_acknowledgment(&world, [0x81; 32], b"phase5-rewrite-source")
            .persisted_records()[0];
    worth_store_testkit_recovery_runtime::c10_phase_five_read::store_identity(&marker, record);
    checkpoint_source(&world);
    let gate = world
        .serving()
        .certification_pause_physical_mutation_at(rewrite_checkpoint(&checkpoint));
    let rewrite = prepare_rewrite(&world, [0x82; 32]).start();
    assert!(gate.await_arrival(), "rewrite did not reach {checkpoint}");
    std::mem::forget(rewrite);
    std::mem::forget(gate);
    std::fs::write(marker.join("reached"), b"1").unwrap();
    loop {
        std::thread::park();
    }
}
