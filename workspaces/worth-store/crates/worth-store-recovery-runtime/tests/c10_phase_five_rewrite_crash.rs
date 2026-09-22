#[allow(dead_code)]
mod c10_phase_five_read;
#[allow(dead_code)]
mod phase_three_support;

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use phase_three_support::{limit_declaration, recovery_request_with_limits};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimits, PhysicalRecoveryOutcome, WorthStoreRecovery,
};
use worth_store_test_support::harness::physical_residency::{
    canonical_physical_mutation_acknowledgment, PhysicalResidencyStoreWorld,
};

const CHILD_MARKER: &str = "C10_PHASE5_REWRITE_MARKER";
const CHILD_CHECKPOINT: &str = "C10_PHASE5_REWRITE_CHECKPOINT";

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
    let (first_retirements, first_payload, first_generation) = release_after_read(parent.path(), &root);
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
    assert_eq!(published_roots, directory_snapshot(&root, "families/records/roots"));
    drop(parent);
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
        let outcome =
            WorthStoreRecovery::recover(recovery_request_with_limits(&root, ordinary_limits()));
        assert!(matches!(outcome, PhysicalRecoveryOutcome::Blocked(_)));
        drop(parent);
    }
}

fn reseal_wrong_page(path: &Path) {
    use worth_store_physical_format::{
        decode_data_frame_page_lsn, DurableFrameKind, DURABLE_FRAME_HEADER_BYTES,
    };
    let mut page = std::fs::read(path).unwrap();
    let lsn = decode_data_frame_page_lsn(&page, DurableFrameKind::InlinePage).unwrap();
    page[400] ^= 0xff;
    let mut covered = Vec::with_capacity(page.len() - 4);
    covered.extend_from_slice(&page[..DURABLE_FRAME_HEADER_BYTES - 4]);
    covered.extend_from_slice(&page[DURABLE_FRAME_HEADER_BYTES..]);
    let checksum = worth_store_physical_format::durable_artifact_checksum(&covered);
    page[DURABLE_FRAME_HEADER_BYTES - 4..DURABLE_FRAME_HEADER_BYTES]
        .copy_from_slice(&checksum.to_le_bytes());
    assert_eq!(
        decode_data_frame_page_lsn(&page, DurableFrameKind::InlinePage).unwrap(),
        lsn
    );
    std::fs::write(path, page).unwrap();
}

#[test]
#[ignore = "launched by the phase 5 rewrite parent"]
fn c10_rewrite_child_parks_at_checkpoint() {
    let marker = PathBuf::from(std::env::var_os(CHILD_MARKER).expect("phase 5 rewrite marker"));
    let checkpoint = std::env::var(CHILD_CHECKPOINT).expect("phase 5 rewrite checkpoint");
    let world =
        PhysicalResidencyStoreWorld::initialize_for_recovery("c10-phase5-rewrite-kill").unwrap();
    std::fs::write(
        marker.join("root-path.txt"),
        world.root().to_string_lossy().as_bytes(),
    )
    .unwrap();
    let record = canonical_physical_mutation_acknowledgment(
        &world,
        [0x81; 32],
        b"phase5-rewrite-source",
    )
    .persisted_records()[0];
    c10_phase_five_read::store_identity(&marker, record);
    checkpoint_source(&world);
    let gate = world
        .serving()
        .certification_pause_physical_mutation_at(rewrite_checkpoint(&checkpoint));
    let rewrite = prepare_rewrite(&world, [0x82; 32]).start();
    assert!(
        gate.await_arrival(),
        "rewrite did not reach {checkpoint}"
    );
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
        other => panic!("unknown rewrite checkpoint {other}"),
    }
}

fn checkpoint_source(world: &PhysicalResidencyStoreWorld) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x83; 32]),
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

fn prepare_rewrite(
    world: &PhysicalResidencyStoreWorld,
    material: [u8; 32],
) -> worth_store::physical_runtime::PreparedPhysicalMutation {
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    match submission
        .rewrite_selected_inline_segment(
            world.placement(),
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(1_000).unwrap(),
            ),
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("rewrite preparation must succeed"),
    }
}

fn kill_child(checkpoint: &str) -> (tempfile::TempDir, PathBuf) {
    let parent = tempfile::tempdir().unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "c10_rewrite_child_parks_at_checkpoint",
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
        .expect("launch phase 5 rewrite child");
    let reached = parent.path().join("reached");
    let deadline = Instant::now() + Duration::from_secs(60);
    while !reached.exists() {
        if Instant::now() > deadline {
            let _ = child.kill();
            panic!("rewrite child did not reach {checkpoint}");
        }
        if let Some(status) = child.try_wait().unwrap() {
            let output = child.wait_with_output().unwrap();
            panic!(
                "rewrite child exited early: {status}\nstdout:\n{}\nstderr:\n{}",
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

fn release_after_read(
    marker: &Path,
    root: &Path,
) -> (Vec<(u64, u64, u64, u64)>, Vec<u8>, u64) {
    let handoff = recover(root);
    let facts = (
        retirements(&handoff),
        source_payload(marker, root, &handoff),
        c10_phase_five_read::root_generation(&handoff),
    );
    drop(handoff);
    facts
}

fn recover(root: &Path) -> worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff {
    let outcome = WorthStoreRecovery::recover(recovery_request_with_limits(root, ordinary_limits()));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("killed rewrite must recover");
    };
    handoff
}

fn retirements(
    handoff: &worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff,
) -> Vec<(u64, u64, u64, u64)> {
    handoff
        .freshness_sample()
        .retirements()
        .iter()
        .map(|retirement| {
            (
                retirement.source_root(),
                retirement.segment_id(),
                retirement.generation(),
                retirement.bytes(),
            )
        })
        .collect()
}

fn source_payload(
    marker: &Path,
    root: &Path,
    handoff: &worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff,
) -> Vec<u8> {
    c10_phase_five_read::selected_payload(
        root,
        handoff,
        c10_phase_five_read::load_identity(marker),
    )
}

fn ordinary_limits() -> PhysicalRecoveryLimits {
    let mut declaration = limit_declaration(2, 8, 2 * 1024 * 1024);
    declaration.manifest_entries = 4_096;
    declaration.wal_bytes = 2 * 1024 * 1024;
    declaration.redo_targets = 4_096;
    declaration.redo_bytes = 4 * 1024 * 1024;
    declaration.distinct_pages_and_extents = 4_096;
    declaration.operation_bindings = 4_096;
    declaration.staging_bytes = 32 * 1024 * 1024;
    declaration.recovery_memory_bytes = 32 * 1024 * 1024;
    declaration.dirty_frames = 4_096;
    declaration.publication_effects = 64;
    declaration.observation_bytes = 32 * 1024 * 1024;
    PhysicalRecoveryLimits::admit(declaration).unwrap()
}

fn segment_snapshot(root: &Path) -> Vec<(String, u64)> {
    directory_snapshot(root, "families/records/segments")
}

fn directory_snapshot(root: &Path, relative: &str) -> Vec<(String, u64)> {
    let directory = root.join(relative);
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(directory) {
        for entry in entries.flatten() {
            let bytes = std::fs::read(entry.path()).unwrap_or_default();
            let mut checksum = 0u64;
            for byte in bytes {
                checksum = checksum.wrapping_mul(16777619).wrapping_add(u64::from(byte));
            }
            files.push((entry.file_name().to_string_lossy().into_owned(), checksum));
        }
    }
    files.sort();
    files
}

fn wal_contains_rewrite(root: &Path) -> bool {
    let domain = worth_store_physical_format::REWRITE_REDO_DOMAIN;
    let mut stack = vec![root.join("families").join("wal")];
    while let Some(directory) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if let Ok(bytes) = std::fs::read(&path) {
                if bytes.windows(domain.len()).any(|window| window == domain) {
                    return true;
                }
            }
        }
    }
    false
}
