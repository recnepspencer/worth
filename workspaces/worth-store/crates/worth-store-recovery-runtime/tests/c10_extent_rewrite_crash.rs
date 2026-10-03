#[path = "c10_extent_rewrite_crash/arena_io.rs"]
mod arena_io;
#[path = "c10_extent_rewrite_crash/arena_route.rs"]
mod arena_route;
use arena_io::{arena_path, extent_payload, flip_arena_byte, payload};
#[allow(dead_code)]
mod c10_crash_evidence;
#[allow(dead_code)]
mod c10_phase_five_read;
#[allow(dead_code)]
mod phase_three_support;

use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use c10_crash_evidence::{directory_snapshot, ordinary_limits, wal_contains_rewrite};
use phase_three_support::recovery_request_with_limits;
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest,
};
use worth_store_physical_format::{
    decode_data_frame_page_lsn, decode_extent_chunk, encode_data_frame_page_lsn,
    prepare_extent_chunk, CurrentPhysicalRecordPlacement, DurableExtentManifest,
    DurableExtentRecordPlacement, DurableFrameKind, ExtentArenaFrameLayout, ExtentArenaRange,
    ExtentChunkCoordinate, PersistedRecordIdentity, DURABLE_EXTENT_FRAME_HEADER_BYTES,
    EXTENT_ARENA_MANIFEST_FRAME_BYTES, EXTENT_CHUNK_METADATA_BYTES,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryOutcome, RecoveredPhysicalRuntimeHandoff, WorthStoreRecovery,
};
use worth_store_test_support::harness::physical_residency::{
    canonical_physical_mutation_acknowledgment, PhysicalResidencyStoreWorld,
};

const CHILD_MARKER: &str = "C10_EXTENT_REWRITE_MARKER";
const CHILD_CHECKPOINT: &str = "C10_EXTENT_REWRITE_CHECKPOINT";
const PAGE_BYTES: usize = 16 * 1024;
const PAYLOAD_BYTES: usize = 40_000;
const ARENAS: &str = "families/records/arenas";

#[test]
fn killed_extent_rewrite_before_wal_keeps_the_source_generation() {
    let (parent, root) = kill_child("before-wal");
    assert!(!wal_contains_rewrite(&root));
    let (first, settled) = recover_until_settled(parent.path(), &root);
    assert_eq!(first, (payload(), 1));
    assert_eq!(settled, first);
    assert_eq!(
        selected_route(&root, c10_phase_five_read::load_identity(parent.path())).generation,
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
    let route = selected_route(&root, c10_phase_five_read::load_identity(parent.path()));
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
            selected_route(&root, c10_phase_five_read::load_identity(parent.path())).generation,
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
    let route = selected_route(&root, c10_phase_five_read::load_identity(parent.path()));
    flip_arena_byte(&root, route.range);
    let outcome =
        WorthStoreRecovery::recover(recovery_request_with_limits(&root, ordinary_limits()));
    assert!(matches!(outcome, PhysicalRecoveryOutcome::Blocked(_)));
    drop(parent);
}

#[test]
fn a_damaged_source_generation_blocks_recovery_without_a_successor() {
    let (parent, root) = kill_child("after-wal");
    let route = selected_route(&root, c10_phase_five_read::load_identity(parent.path()));
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

fn assert_blocked_without_successor(marker: &Path, root: &Path) {
    let outcome =
        WorthStoreRecovery::recover(recovery_request_with_limits(root, ordinary_limits()));
    assert!(matches!(outcome, PhysicalRecoveryOutcome::Blocked(_)));
    assert_eq!(
        selected_route(root, c10_phase_five_read::load_identity(marker)).generation,
        1
    );
}

fn replace_source_payload(root: &Path) {
    let record = arena_route::selected_routes(root)
        .into_iter()
        .find(|route| route.generation == 1)
        .expect("the killed root retains its source extent");
    let path = arena_path(root, record.range);
    let original = std::fs::read(&path).unwrap();
    let range_start = record.range.offset() as usize;
    let (manifest, format) = DurableExtentManifest::decode(
        &original[range_start..range_start + EXTENT_ARENA_MANIFEST_FRAME_BYTES],
    )
    .unwrap();
    assert_eq!(manifest.record(), record.record);
    assert_eq!(manifest.extent().get(), record.extent);
    assert_eq!(manifest.extent_cell().generation().get(), record.generation);
    assert_eq!(manifest.logical_bytes(), record.payload_bytes);
    let layout = ExtentArenaFrameLayout::new(format, manifest.alignment()).unwrap();
    assert!(layout.admits(record.range, manifest.chunk_count()));
    let mut forged = payload();
    forged[PAYLOAD_BYTES / 2] ^= 0xff;
    let capacity = PAGE_BYTES - DURABLE_EXTENT_FRAME_HEADER_BYTES - EXTENT_CHUNK_METADATA_BYTES;
    let mut file = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
    for (ordinal, start) in (1_u32..).zip((0..forged.len()).step_by(capacity)) {
        let offset = range_start + layout.chunk_offset(ordinal).unwrap() as usize;
        let length = (forged.len() - start).min(capacity);
        let coordinate = ExtentChunkCoordinate::new(
            manifest.record(),
            manifest.extent_cell(),
            manifest.logical_bytes(),
            start as u64,
            ordinal,
        )
        .unwrap();
        let mut chunk = prepare_extent_chunk(format, coordinate, length).unwrap();
        chunk
            .payload_mut()
            .copy_from_slice(&forged[start..start + length]);
        let mut frame = chunk.seal();
        let page_lsn = decode_data_frame_page_lsn(
            &original[offset..offset + frame.len()],
            DurableFrameKind::Extent,
        )
        .unwrap();
        encode_data_frame_page_lsn(&mut frame, DurableFrameKind::Extent, page_lsn).unwrap();
        file.seek(SeekFrom::Start(offset as u64)).unwrap();
        file.write_all(&frame).unwrap();
    }
    file.sync_all().unwrap();
}

#[test]
#[ignore = "launched by the extent rewrite parent"]
fn c10_extent_rewrite_child_parks_at_checkpoint() {
    let marker = PathBuf::from(std::env::var_os(CHILD_MARKER).expect("extent rewrite marker"));
    let checkpoint = std::env::var(CHILD_CHECKPOINT).expect("extent rewrite checkpoint");
    let world =
        PhysicalResidencyStoreWorld::initialize_for_recovery("c10-extent-rewrite-kill").unwrap();
    std::fs::write(
        marker.join("root-path.txt"),
        world.root().to_string_lossy().as_bytes(),
    )
    .unwrap();
    let acknowledgment = canonical_physical_mutation_acknowledgment(&world, [0x91; 32], &payload());
    c10_phase_five_read::store_identity(&marker, acknowledgment.persisted_records()[0]);
    let record = acknowledgment.record_ids().next().unwrap();
    checkpoint_source(&world);
    let gate = world
        .serving()
        .certification_pause_physical_mutation_at(rewrite_checkpoint(&checkpoint));
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0x92; 32]))
        .unwrap();
    let prepared = match submission
        .rewrite_selected_extent_record(
            world.placement(),
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(1_000).unwrap(),
            ),
            record,
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
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
        thread::park();
    }
}

fn rewrite_checkpoint(name: &str) -> CertificationPhysicalMutationCheckpoint {
    match name {
        "before-wal" => CertificationPhysicalMutationCheckpoint::BeforeWalAppend,
        "after-wal" => CertificationPhysicalMutationCheckpoint::AfterWalDurability,
        "after-data" => CertificationPhysicalMutationCheckpoint::AfterDataSettlement,
        "during-root" => CertificationPhysicalMutationCheckpoint::DuringRootPublication,
        "after-replace" => CertificationPhysicalMutationCheckpoint::AfterRootReplacement,
        "before-ack" => CertificationPhysicalMutationCheckpoint::BeforeTerminalFinalization,
        other => panic!("unknown extent rewrite checkpoint {other}"),
    }
}

fn checkpoint_source(world: &PhysicalResidencyStoreWorld) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x93; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let handle = match world.serving().checkpoints().start(request).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        _ => panic!("source checkpoint was not admitted"),
    };
    match handle.wait() {
        PhysicalCheckpointOutcome::Completed(_) => {}
        _ => panic!("source checkpoint must complete"),
    }
}

fn kill_child(checkpoint: &str) -> (tempfile::TempDir, PathBuf) {
    let parent = tempfile::tempdir().unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "c10_extent_rewrite_child_parks_at_checkpoint",
            "--ignored",
            "--nocapture",
        ])
        .env(CHILD_MARKER, parent.path())
        .env(CHILD_CHECKPOINT, checkpoint)
        .env("TMP", parent.path())
        .env("TEMP", parent.path())
        .env("TMPDIR", parent.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("launch extent rewrite child");
    let reached = parent.path().join("reached");
    let deadline = Instant::now() + Duration::from_secs(60);
    while !reached.exists() {
        if Instant::now() > deadline {
            let _ = child.kill();
            panic!("extent rewrite child did not reach {checkpoint}");
        }
        if let Some(status) = child.try_wait().unwrap() {
            let output = child.wait_with_output().unwrap();
            panic!(
                "extent rewrite child exited early: {status}\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(20));
    }
    child.kill().unwrap();
    let _ = child.wait();
    let root = PathBuf::from(std::fs::read_to_string(parent.path().join("root-path.txt")).unwrap());
    (parent, root)
}

/// Recovers three times and returns the payload and extent generation the
/// first and the settled passes select. A handoff reports the root recovery
/// selected before publishing, so the second pass shows what the first
/// published; no extent data or manifest may change after the first pass.
fn recover_until_settled(marker: &Path, root: &Path) -> ((Vec<u8>, u64), (Vec<u8>, u64)) {
    let record = c10_phase_five_read::load_identity(marker);
    let first = selected(root, &recover(root), record);
    let arenas = directory_snapshot(root, ARENAS);
    let roots = directory_snapshot(root, "families/records/roots");
    let second = selected(root, &recover(root), record);
    let third = selected(root, &recover(root), record);
    assert_eq!(second, third, "recovery must be idempotent once settled");
    assert_eq!(arenas, directory_snapshot(root, ARENAS));
    assert_eq!(roots, directory_snapshot(root, "families/records/roots"));
    (first, second)
}

fn recover(root: &Path) -> RecoveredPhysicalRuntimeHandoff {
    match WorthStoreRecovery::recover(recovery_request_with_limits(root, ordinary_limits())) {
        PhysicalRecoveryOutcome::Recovered(handoff) => handoff,
        _ => panic!("killed extent rewrite must recover"),
    }
}

fn selected(
    root: &Path,
    handoff: &RecoveredPhysicalRuntimeHandoff,
    record: PersistedRecordIdentity,
) -> (Vec<u8>, u64) {
    let placement = handoff
        .selected_sources()
        .page_facts()
        .placements()
        .iter()
        .find_map(|placement| match placement {
            CurrentPhysicalRecordPlacement::Extent(extent) if extent.record() == record => {
                Some(*extent)
            }
            _ => None,
        })
        .expect("selected root does not place the record in an extent");
    (
        extent_payload(root, placement),
        placement.extent_generation(),
    )
}

/// The one addressed extent the selected root routes for `record`.
fn selected_route(
    root: &std::path::Path,
    record: worth_store_physical_format::PersistedRecordIdentity,
) -> arena_route::RoutedExtent {
    let matching = arena_route::selected_routes(root)
        .into_iter()
        .filter(|route| route.record == record)
        .collect::<Vec<_>>();
    assert_eq!(
        matching.len(),
        1,
        "selected root has one addressed extent for the record"
    );
    matching[0]
}
