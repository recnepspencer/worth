#[allow(unused_imports)]
use crate::c10_phase_six_span_crash::*;
use {
    crate::c10_phase_six_span_crash::c10_crash_evidence::segment_snapshot,
    crate::c10_phase_six_span_crash::c10_crash_evidence::wal_contains_rewrite,
};

use worth_store_test_support::harness::physical_residency::canonical_physical_batch_acknowledgment;

#[test]
fn sealed_spans_recover_one_generation() {
    for (published, pages, checkpoint, new_generation) in [
        (4_u32, 4_u32, "after-wal", true),
        (16, 16, "after-wal", true),
        (8, 4, "after-wal", true),
        (4, 4, "after-data", false),
    ] {
        let (parent, root) = kill_span(published, pages, checkpoint);
        assert!(wal_contains_rewrite(&root), "{checkpoint} {pages}");
        let killed = segment_snapshot(&root);
        let (first_payload, first_generation) =
            release(parent.path(), &root, published, checkpoint, pages, "first");
        let published_files = segment_snapshot(&root);
        let (second_payload, second_generation) =
            release(parent.path(), &root, published, checkpoint, pages, "second");
        assert_eq!(first_payload, second_payload, "{checkpoint} {pages}");
        assert_eq!(
            second_generation,
            first_generation + 1,
            "{checkpoint} {pages}"
        );
        assert_eq!(
            published_files,
            segment_snapshot(&root),
            "{checkpoint} {pages}"
        );
        if new_generation {
            assert_ne!(killed, published_files, "{checkpoint} {pages}");
            assert!(
                published_files.len() <= killed.len() + 1,
                "{checkpoint} {pages} published more than one generation"
            );
        }
        for (index, payload) in first_payload.iter().enumerate() {
            assert_eq!(payload.len(), 7_500, "{checkpoint} {pages} record {index}");
            assert_eq!(
                payload[0], index as u8,
                "{checkpoint} {pages} record {index}"
            );
        }
        drop(parent);
    }
}

#[test]
fn partial_span_rewrite_result_remains_exact_after_later_roots() {
    let (parent, root) = kill_span(8, 4, "after-wal");
    let (expected_payload, _) = release(parent.path(), &root, 8, "after-wal", 4, "result");
    assert_eq!(expected_payload.len(), 8);
    for (index, bytes) in expected_payload.iter().enumerate() {
        assert_eq!(bytes.len(), 7_500);
        assert_eq!(bytes[0], index as u8);
    }
    let result_root = worth_store_testkit_recovery_runtime::historical_rewrite_support::selected_catalog_generation(&root);
    worth_store_testkit_recovery_runtime::historical_rewrite_support::advance_with_ordinary_appends(
        &root, true,
    );
    let selected_before = worth_store_testkit_recovery_runtime::historical_rewrite_support::selected_catalog_generation(&root);
    assert!(
        selected_before >= result_root + 2,
        "the partial-span result root must be neither selected nor retained-previous"
    );
    let segments_before = segment_snapshot(&root);
    let (payload, selected_after) = release(parent.path(), &root, 8, "after-wal", 4, "historical");
    assert_eq!(payload, expected_payload);
    assert_eq!(selected_after, selected_before);
    assert_eq!(
        worth_store_testkit_recovery_runtime::historical_rewrite_support::selected_catalog_generation(&root),
        selected_before,
        "fresh recovery must not republish an already historical span"
    );
    assert_eq!(segment_snapshot(&root), segments_before);
}

#[test]
#[ignore = "launched by the phase 6 span parent"]
fn span_child_parks() {
    let marker = std::path::PathBuf::from(std::env::var_os(MARKER).expect("span marker"));
    let checkpoint = std::env::var(CHECKPOINT).expect("span checkpoint");
    let published: u32 = std::env::var(PUBLISHED).unwrap().parse().unwrap();
    let pages: u32 = std::env::var(PAGES).unwrap().parse().unwrap();
    let world =
        worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_span_rewrite("c10-phase6-span-kill", 16)
            .unwrap();
    std::fs::write(
        marker.join("root-path.txt"),
        world.root().to_string_lossy().as_bytes(),
    )
    .unwrap();
    let mut records = Vec::with_capacity(published as usize);
    for index in 0..published {
        let mut bytes = vec![0_u8; 7_500];
        bytes[0] = index as u8;
        records.push(bytes);
    }
    let borrowed: Vec<&[u8]> = records.iter().map(Vec::as_slice).collect();
    let acknowledgment = canonical_physical_batch_acknowledgment(&world, [0x91; 32], borrowed);
    store_identities(&marker, acknowledgment.persisted_records());
    checkpoint_source(&world);
    let gate = world
        .serving()
        .certification_pause_physical_mutation_at(rewrite_checkpoint(&checkpoint));
    let rewrite = prepare_span(&world, pages).start();
    assert!(
        gate.await_arrival(),
        "span rewrite did not reach {checkpoint}"
    );
    std::mem::forget(rewrite);
    std::mem::forget(gate);
    std::fs::write(marker.join("reached"), b"1").unwrap();
    loop {
        std::thread::park();
    }
}
