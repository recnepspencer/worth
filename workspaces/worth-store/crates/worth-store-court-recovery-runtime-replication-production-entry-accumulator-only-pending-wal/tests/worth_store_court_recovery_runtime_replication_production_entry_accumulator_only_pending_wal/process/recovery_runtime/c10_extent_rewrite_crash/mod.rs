

#[allow(unused_imports)] use crate::c10_extent_rewrite_crash::*;
use {crate::c10_extent_rewrite_crash::arena_io::arena_path, crate::c10_extent_rewrite_crash::arena_io::extent_payload, crate::c10_extent_rewrite_crash::arena_io::flip_arena_byte, crate::c10_extent_rewrite_crash::arena_io::payload};





use {std::io::Seek, std::io::SeekFrom, std::io::Write};
use {std::path::Path, std::path::PathBuf};
use {std::process::Command, std::process::Stdio};
use {std::thread};
use {std::time::Duration, std::time::Instant};

use {crate::c10_extent_rewrite_crash::c10_crash_evidence::directory_snapshot, crate::c10_extent_rewrite_crash::c10_crash_evidence::ordinary_limits, crate::c10_extent_rewrite_crash::c10_crash_evidence::wal_contains_rewrite};
use {worth_store_testkit_recovery_runtime::phase_three_support::recovery_request_with_limits};
use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint};
use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest};
use {worth_store_physical_format::decode_data_frame_page_lsn, worth_store_physical_format::decode_extent_chunk, worth_store_physical_format::encode_data_frame_page_lsn, worth_store_physical_format::prepare_extent_chunk, worth_store_physical_format::CurrentPhysicalRecordPlacement, worth_store_physical_format::DurableExtentManifest, worth_store_physical_format::DurableExtentRecordPlacement, worth_store_physical_format::DurableFrameKind, worth_store_physical_format::ExtentArenaFrameLayout, worth_store_physical_format::ExtentArenaRange, worth_store_physical_format::ExtentChunkCoordinate, worth_store_physical_format::PersistedRecordIdentity, worth_store_physical_format::DURABLE_EXTENT_FRAME_HEADER_BYTES, worth_store_physical_format::EXTENT_ARENA_MANIFEST_FRAME_BYTES, worth_store_physical_format::EXTENT_CHUNK_METADATA_BYTES};
use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff, worth_store_recovery_runtime::WorthStoreRecovery};
use {worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};

#[test]
fn killed_extent_rewrite_before_wal_keeps_the_source_generation() {
    let (parent, root) = kill_child("before-wal");
    assert!(!wal_contains_rewrite(&root));
    let (first, settled) = recover_until_settled(parent.path(), &root);
    assert_eq!(first, (payload(), 1));
    assert_eq!(settled, first);
    assert_eq!(
        selected_route(&root, worth_store_testkit_recovery_runtime::c10_phase_five_read::load_identity(parent.path())).generation,
        1
    );
    drop(parent);
}

#[test]
fn killed_wal_durable_extent_rewrite_publishes_the_next_generation_once() {
    let (parent, root) = kill_child("after-wal");
    assert!(wal_contains_rewrite(&root));
    let killed = directory_snapshot(&root, ARENAS);
    let (first, settled) = recover_until_settled(parent.path(), &root);
    assert_eq!(
        first,
        (payload(), 1),
        "the killed root still selects the source"
    );
    assert_eq!(settled, (payload(), 2));
    assert_ne!(killed, directory_snapshot(&root, ARENAS));
    let route = selected_route(&root, worth_store_testkit_recovery_runtime::c10_phase_five_read::load_identity(parent.path()));
    assert_eq!(route.generation, 2);
    assert!(arena_path(&root, route.range).exists());
    drop(parent);
}

#[test]
fn killed_extent_rewrite_after_candidate_data_settles_one_generation() {
    // The first pass reports the root it selected: the source until the killed
    // process replaced the root, the successor afterwards.
    for (seam, selected) in [
        ("after-data", 1),
        ("during-root", 1),
        ("after-replace", 2),
        ("before-ack", 2),
    ] {
        let (parent, root) = kill_child(seam);
        assert!(wal_contains_rewrite(&root), "{seam}");
        let (first, settled) = recover_until_settled(parent.path(), &root);
        assert_eq!(first, (payload(), selected), "{seam}");
        assert_eq!(settled, (payload(), 2), "{seam}");
        assert_eq!(
            selected_route(&root, worth_store_testkit_recovery_runtime::c10_phase_five_read::load_identity(parent.path())).generation,
            2
        );
        drop(parent);
    }
}

#[test]
fn a_damaged_recovered_extent_generation_blocks_recovery() {
    let (parent, root) = kill_child("after-wal");
    let (_, settled) = recover_until_settled(parent.path(), &root);
    assert_eq!(settled.1, 2);
    let route = selected_route(&root, worth_store_testkit_recovery_runtime::c10_phase_five_read::load_identity(parent.path()));
    flip_arena_byte(&root, route.range);
    let outcome =
        worth_store_recovery_runtime::WorthStoreRecovery::recover(recovery_request_with_limits(&root, ordinary_limits()));
    assert!(matches!(outcome, worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(_)));
    drop(parent);
}

#[test]
fn a_damaged_source_generation_blocks_recovery_without_a_successor() {
    let (parent, root) = kill_child("after-wal");
    let route = selected_route(&root, worth_store_testkit_recovery_runtime::c10_phase_five_read::load_identity(parent.path()));
    flip_arena_byte(&root, route.range);
    assert_blocked_without_successor(parent.path(), &root);
    drop(parent);
}

/// Every frame and the manifest stay valid; only the payload differs from the
/// bytes the rewrite redo digested, so only that digest can refuse it.
#[test]
fn a_well_formed_source_that_differs_from_the_redo_digest_blocks_recovery() {
    let (parent, root) = kill_child("after-wal");
    replace_source_payload(&root);
    assert_blocked_without_successor(parent.path(), &root);
    drop(parent);
}

#[test]
#[ignore = "launched by the extent rewrite parent"]
fn c10_extent_rewrite_child_parks_at_checkpoint() {
    let marker = std::path::PathBuf::from(std::env::var_os(CHILD_MARKER).expect("extent rewrite marker"));
    let checkpoint = std::env::var(CHILD_CHECKPOINT).expect("extent rewrite checkpoint");
    let world =
        worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("c10-extent-rewrite-kill").unwrap();
    std::fs::write(
        marker.join("root-path.txt"),
        world.root().to_string_lossy().as_bytes(),
    )
    .unwrap();
    let acknowledgment = canonical_physical_mutation_acknowledgment(&world, [0x91; 32], &payload());
    worth_store_testkit_recovery_runtime::c10_phase_five_read::store_identity(&marker, acknowledgment.persisted_records()[0]);
    let record = acknowledgment.record_ids().next().unwrap();
    checkpoint_source(&world);
    let gate = world
        .serving()
        .certification_pause_physical_mutation_at(rewrite_checkpoint(&checkpoint));
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0x92; 32]))
        .unwrap();
    let prepared = match submission
        .rewrite_selected_extent_record(
            world.placement(),
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(1_000).unwrap(),
            ),
            record,
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("extent rewrite preparation must succeed"),
    };
    let rewrite = prepared.start();
    assert!(gate.await_arrival(), "rewrite did not reach {checkpoint}");
    std::mem::forget(rewrite);
    std::mem::forget(gate);
    std::fs::write(marker.join("reached"), b"1").unwrap();
    loop {
        std::thread::park();
    }
}
#[path = "observation_limit/mod.rs"]
pub(crate) mod observation_limit;
