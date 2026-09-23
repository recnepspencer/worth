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
use worth_store_physical_format::PersistedRecordIdentity;
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimits, PhysicalRecoveryOutcome, WorthStoreRecovery,
};
use worth_store_test_support::harness::physical_residency::{
    canonical_physical_batch_acknowledgment, PhysicalResidencyStoreWorld,
};

const MARKER: &str = "C10_PHASE6_SPAN_MARKER";
const CHECKPOINT: &str = "C10_PHASE6_SPAN_CHECKPOINT";
const PUBLISHED: &str = "C10_PHASE6_SPAN_PUBLISHED";
const PAGES: &str = "C10_PHASE6_SPAN_PAGES";

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
#[ignore = "launched by the phase 6 span parent"]
fn c10_span_child_parks() {
    let marker = PathBuf::from(std::env::var_os(MARKER).expect("span marker"));
    let checkpoint = std::env::var(CHECKPOINT).expect("span checkpoint");
    let published: u32 = std::env::var(PUBLISHED).unwrap().parse().unwrap();
    let pages: u32 = std::env::var(PAGES).unwrap().parse().unwrap();
    let world =
        PhysicalResidencyStoreWorld::initialize_for_span_rewrite("c10-phase6-span-kill", 16)
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
        thread::park();
    }
}

fn checkpoint_source(world: &PhysicalResidencyStoreWorld) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x93; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let handle = match world.serving().checkpoints().start(request).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        _ => panic!("span source checkpoint was not admitted"),
    };
    match handle.wait() {
        PhysicalCheckpointOutcome::Completed(_) => {}
        _ => panic!("span source checkpoint must complete"),
    }
}

fn rewrite_checkpoint(name: &str) -> CertificationPhysicalMutationCheckpoint {
    match name {
        "after-wal" => CertificationPhysicalMutationCheckpoint::AfterWalDurability,
        "after-data" => CertificationPhysicalMutationCheckpoint::AfterDataSettlement,
        other => panic!("unknown span checkpoint {other}"),
    }
}

fn prepare_span(
    world: &PhysicalResidencyStoreWorld,
    pages: u32,
) -> worth_store::physical_runtime::PreparedPhysicalMutation {
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0x92; 32]))
        .unwrap();
    match submission
        .rewrite_selected_inline_pages(
            world.placement(),
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ),
            pages,
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        TransitionOutcome::Success(_) => panic!("span rewrite did not stay prepared"),
        TransitionOutcome::Denied(_) => panic!("span rewrite denied"),
        TransitionOutcome::Deferred(_) => panic!("span rewrite deferred"),
        TransitionOutcome::Stale(_) => panic!("span rewrite stale"),
        TransitionOutcome::RebindRequired(_) => panic!("span rewrite rebind"),
        TransitionOutcome::Failed(_) => panic!("span rewrite failed"),
    }
}

fn kill_span(published: u32, pages: u32, checkpoint: &str) -> (tempfile::TempDir, PathBuf) {
    let parent = tempfile::tempdir().unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "c10_span_child_parks",
            "--ignored",
            "--nocapture",
        ])
        .env(MARKER, parent.path())
        .env(CHECKPOINT, checkpoint)
        .env(PUBLISHED, published.to_string())
        .env(PAGES, pages.to_string())
        .env("TMP", parent.path())
        .env("TEMP", parent.path())
        .env("TMPDIR", parent.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("launch phase 6 span child");
    let reached = parent.path().join("reached");
    let deadline = Instant::now() + Duration::from_secs(60);
    while !reached.exists() {
        if Instant::now() > deadline {
            let _ = child.kill();
            panic!("span child did not reach {checkpoint} pages={pages}");
        }
        if let Some(status) = child.try_wait().unwrap() {
            let output = child.wait_with_output().unwrap();
            panic!(
                "span child exited early: {status}\nstdout:\n{}\nstderr:\n{}",
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

fn release(
    marker: &Path,
    root: &Path,
    published: u32,
    checkpoint: &str,
    pages: u32,
    pass: &str,
) -> (Vec<Vec<u8>>, u64) {
    let handoff = recover(root, checkpoint, pages, pass);
    let identities = load_identities(marker);
    assert_eq!(identities.len(), published as usize);
    let payload = identities
        .into_iter()
        .map(|identity| c10_phase_five_read::selected_payload(root, &handoff, identity))
        .collect();
    let generation = c10_phase_five_read::root_generation(&handoff);
    drop(handoff);
    (payload, generation)
}

fn recover(
    root: &Path,
    checkpoint: &str,
    pages: u32,
    pass: &str,
) -> worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff {
    let outcome =
        WorthStoreRecovery::recover(recovery_request_with_limits(root, ordinary_limits()));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("sealed span {checkpoint} pages={pages} {pass} must recover: {outcome:?}");
    };
    handoff
}

fn store_identities(marker: &Path, records: &[PersistedRecordIdentity]) {
    let mut bytes = Vec::new();
    for record in records {
        bytes.extend_from_slice(&record.allocation_epoch());
        bytes.extend_from_slice(&record.ordinal().to_le_bytes());
    }
    std::fs::write(marker.join("span-ids.bin"), bytes).unwrap();
}

fn load_identities(marker: &Path) -> Vec<PersistedRecordIdentity> {
    let bytes = std::fs::read(marker.join("span-ids.bin")).unwrap();
    bytes
        .chunks(24)
        .map(|chunk| {
            let mut epoch = [0; 16];
            epoch.copy_from_slice(&chunk[..16]);
            let ordinal = u64::from_le_bytes(chunk[16..24].try_into().unwrap());
            PersistedRecordIdentity::new(epoch, ordinal).unwrap()
        })
        .collect()
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
    let mut files = Vec::new();
    let directory = root.join("families/records/segments");
    if let Ok(entries) = std::fs::read_dir(directory) {
        for entry in entries.flatten() {
            let bytes = std::fs::read(entry.path()).unwrap_or_default();
            let mut checksum = 0_u64;
            for byte in bytes {
                checksum = checksum
                    .wrapping_mul(16_777_619)
                    .wrapping_add(u64::from(byte));
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
