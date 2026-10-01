use std::{
    fs,
    num::NonZeroU64,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    production::PhysicalMutationCheckpoint, BlobCheckpointLimit, BlobIngestDeclaration,
    BlobReadLimits, BlobReadOpenFailure, LayoutRebuildLimits, PhysicalCheckpointDeadline,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
    PhysicalLayoutDenial, PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationSuccess, PhysicalMutationRequest,
    PublishedBlobGeneration, RecordAppendBatch,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_contracts::DurableArtifactFamilyId;

use super::{
    blob_ingest_process::observe_closed_store_named,
    fixture::{admitted_blob_scope, placement, serving_from_initialization, serving_from_open},
};

#[path = "blob_crash/recovery.rs"]
mod recovery;
pub(super) use recovery::{recover_closed_store, recover_closed_store_with_profile};

#[path = "blob_crash/chunk_bytes.rs"]
mod chunk_bytes;
use chunk_bytes::{expected_byte, fill_chunk};

const CHUNK_BYTES: usize = 64 * 1024;
const CRASH_OBJECT_CHUNKS: usize = 3;
pub(super) const SCOPE_KEY: &str = "c11.blob.crash.scope";
const CHILD_ROLE: &str = "WORTH_STORE_C11_BLOB_CHILD_ROLE";
const CHILD_ROOT: &str = "WORTH_STORE_C11_BLOB_CHILD_ROOT";
const CHILD_TEST: &str = "c11_blob_child_role";
const MARKER: &str = "blob-crash-ready";

#[test]
fn declaration_root_publication_survives_kill_without_inventing_generation() {
    let world = kill_at("crash-declaration", Duration::from_secs(90));
    assert_reopened(&world, false, 0);
}

#[test]
fn selected_chunk_claim_survives_kill_without_inventing_generation() {
    let world = kill_at("crash-chunk", Duration::from_secs(90));
    assert_reopened(&world, false, 1);
}

#[test]
fn wal_durable_generation_rolls_forward_after_kill_before_root_publication() {
    let world = kill_at("crash-generation-wal", Duration::from_secs(300));
    assert_reopened(&world, true, CRASH_OBJECT_CHUNKS);
}

pub(super) fn child(root: &Path, role: &str) {
    let serving = serving_from_initialization(root);
    establish_recovery_frontier(&serving);
    let scope = admitted_blob_scope(SCOPE_KEY);
    let limits = BlobReadLimits::new(NonZeroU64::new(1024).unwrap());
    let blobs = serving.blobs().unwrap();
    let object = blobs
        .issue_object_id(limits)
        .unwrap_or_else(|_| panic!("Store must issue an unoccupied object identity"));
    let total = if role == "crash-generation-wal" {
        CRASH_OBJECT_CHUNKS * CHUNK_BYTES
    } else {
        2 * CHUNK_BYTES
    };
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        total as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(1024).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK_BYTES as u64, limits)
        .unwrap_or_else(|_| panic!("durable declaration must complete before crash seam"));
    let token_path = resume_token_path(root);
    let pending_token = token_path.with_extension("pending");
    fs::write(&pending_token, ingest.resume_token().encode()).unwrap();
    fs::rename(pending_token, token_path).unwrap();
    let session = ingest.session_id().bytes();
    match role {
        "crash-declaration" => park_at_marker(root, object.bytes(), session),
        "crash-chunk" => {
            let mut frame = vec![0_u8; CHUNK_BYTES];
            fill_chunk(0, &mut frame);
            ingest
                .push(&frame)
                .unwrap_or_else(|error| panic!("first selected chunk claim failed: {error:?}"));
            park_at_marker(root, object.bytes(), session);
        }
        "crash-frontier-partial" => {
            let full = [0x3c_u8; CHUNK_BYTES];
            ingest
                .push(&full)
                .unwrap_or_else(|error| panic!("frontier child full chunk failed: {error:?}"));
            let frontier = ingest
                .checkpoint()
                .unwrap_or_else(|error| panic!("frontier child checkpoint failed: {error:?}"));
            assert_eq!(
                (frontier.bytes(), frontier.next_chunk_ordinal()),
                (CHUNK_BYTES as u64, 1)
            );
            ingest
                .push(&[0xa5; 17])
                .unwrap_or_else(|error| panic!("frontier child partial source failed: {error:?}"));
            assert_eq!(
                ingest.frontier(),
                frontier,
                "partial input cannot advance durable prefix"
            );
            park_at_marker(root, object.bytes(), session);
        }
        "crash-leaf-selected" => {
            let mut frame = vec![0_u8; CHUNK_BYTES];
            for ordinal in 0..2 {
                fill_chunk(ordinal, &mut frame);
                ingest.push(&frame).unwrap_or_else(|error| {
                    panic!("chunk {ordinal} failed before selected leaf: {error:?}")
                });
            }
            super::blob_resume_tree::park_after_leaf_root(
                &serving,
                ingest,
                root,
                object.bytes(),
                session,
            );
        }
        "crash-generation-wal" => {
            let mut frame = vec![0_u8; CHUNK_BYTES];
            for ordinal in 0..CRASH_OBJECT_CHUNKS {
                fill_chunk(ordinal, &mut frame);
                ingest.push(&frame).unwrap_or_else(|error| {
                    panic!("chunk {ordinal} failed before generation seam: {error:?}")
                });
            }
            // A nonempty partial leaf below fanout 4096 is flushed by finish,
            // then finish appends the generation. Replace the first C4 gate
            // while it holds the leaf; the second gate marks publication WAL.
            let leaf_gate =
                serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::AfterWalDurability);
            let marker = marker_path(root);
            thread::scope(|workers| {
                workers.spawn(|| {
                    let deadline = Instant::now() + Duration::from_secs(180);
                    while Instant::now() < deadline {
                        if leaf_gate.await_arrival() {
                            let publication_gate = serving.pause_physical_mutation_at(
                                PhysicalMutationCheckpoint::AfterWalDurability,
                            );
                            leaf_gate.release();
                            while Instant::now() < deadline {
                                if publication_gate.await_arrival() {
                                    write_marker(&marker, object.bytes(), session);
                                    return;
                                }
                            }
                            panic!("generation publication never reached durable WAL seam");
                        }
                    }
                    panic!("tree leaf never reached durable WAL seam");
                });
                let _ = ingest.finish();
                panic!("generation publication escaped WAL-durable pause before process kill");
            });
        }
        _ => panic!("unknown blob crash seam: {role}"),
    }
}

pub(super) fn establish_recovery_frontier(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
) {
    // C8 source selection requires a bounded checkpoint/WAL frontier. The
    // bootstrap root has no checkpoint and its first WAL LSN is 1, so publish
    // one ordinary seed record before checkpointing, as in the C8 crash lane.
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0xc0; 32]))
        .unwrap();
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                RecordAppendBatch::try_from_iter([b"c11-recovery-frontier".as_slice()]).unwrap(),
                placement(),
                PhysicalMutationRequest::platform_durable(
                    key,
                    PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                ),
            )
            .into_raw()
    else {
        panic!("baseline C5 record must prepare before checkpoint");
    };
    let PhysicalMutationOutcome::Completed(seed) = prepared.execute() else {
        panic!("baseline C5 record must commit before checkpoint");
    };
    assert_eq!(seed.persisted_records().len(), 1);
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0xc1; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(checkpoint) = serving.checkpoints().start(request).into_raw()
    else {
        panic!("baseline C8 recovery checkpoint must admit before blob effects");
    };
    assert!(
        matches!(checkpoint.wait(), PhysicalCheckpointOutcome::Completed(_)),
        "baseline C8 recovery checkpoint must complete before blob effects"
    );
}

pub(super) struct KilledBlobWorld {
    _directory: tempfile::TempDir,
    pub(super) root: PathBuf,
    pub(super) object: [u8; 16],
    pub(super) session: [u8; 16],
    seam: &'static str,
}

pub(super) fn kill_at(seam: &'static str, timeout: Duration) -> KilledBlobWorld {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    fs::create_dir(&root).unwrap();
    let (object, session) = kill_child_at(&root, seam, timeout);
    KilledBlobWorld {
        _directory: directory,
        root,
        object,
        session,
        seam,
    }
}

fn kill_child_at(root: &Path, seam: &'static str, timeout: Duration) -> ([u8; 16], [u8; 16]) {
    let marker = marker_path(root);
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
        .env(CHILD_ROLE, seam)
        .env(CHILD_ROOT, &root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + timeout;
    while !marker.is_file() {
        if child.try_wait().unwrap().is_some() || Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "blob {seam} seam not reached: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    let marked = fs::read(marker).unwrap();
    assert_eq!(marked.len(), 32, "marker must bind object and session");
    child.kill().unwrap();
    child.wait().unwrap();
    (
        marked[..16].try_into().unwrap(),
        marked[16..32].try_into().unwrap(),
    )
}

fn assert_reopened(world: &KilledBlobWorld, published: bool, chunks: usize) {
    recover_closed_store(&world.root);
    let serving = serving_from_open(&world.root);
    let scope = admitted_blob_scope(SCOPE_KEY);
    let limits = BlobReadLimits::new(NonZeroU64::new(1024).unwrap());
    let blobs = serving.blobs().unwrap();
    if published {
        let latest = serving
            .certification_selected_latest_blob_publication()
            .unwrap()
            .expect("C8 must restore the generation marker; prebuild the current physical_store_recover helper before this cross-package crash test");
        assert!(
            matches!(
                blobs.resolve_publication(world.object, 1, &scope, limits),
                Err(BlobReadOpenFailure::Layout(
                    PhysicalLayoutDenial::IndexStaleRequiresRebuild { indexed: None, .. }
                ))
            ),
            "C8 publishes authority before the derived catalog exists"
        );
        let rebuilt = serving
            .layouts()
            .unwrap()
            .rebuild(
                DurableArtifactFamilyId::BlobCatalog,
                LayoutRebuildLimits::new(
                    NonZeroU64::new(1024).unwrap(),
                    NonZeroU64::new(1024).unwrap(),
                ),
                placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            )
            .expect("Store rebuild must be reachable despite a stale catalog");
        assert_eq!(rebuilt.source(), latest);
        assert_eq!(rebuilt.validated_chunks(), CRASH_OBJECT_CHUNKS as u64);
    }
    let resolved = blobs.resolve_publication(world.object, 1, &scope, limits);
    if published {
        let generation = resolved.expect("C8 must roll durable generation WAL forward");
        assert_eq!(generation.session().bytes(), world.session);
        assert_eq!(generation.object().bytes(), world.object);
        assert_eq!(generation.generation().sequence(), 1);
        assert_recovered_bytes(&blobs, generation, &scope, limits);
    } else {
        assert!(matches!(
            resolved,
            Err(BlobReadOpenFailure::PublicationNotFound)
        ));
    }
    drop(blobs);
    serving.close();

    let report = observe_closed_store_named(&world.root, "c11-blob-crash", world.seam);
    assert_eq!(report["completeness"], "complete", "{report}");
    let artifacts = report["artifacts"].as_array().unwrap();
    let family = |name: &str| {
        artifacts
            .iter()
            .filter(|artifact| artifact["family"] == name)
            .collect::<Vec<_>>()
    };
    assert_eq!(family("blob_resume_session").len(), 1, "{report}");
    assert_eq!(family("blob_chunk_frame").len(), chunks, "{report}");
    assert_eq!(
        family("blob_generation_publication").len(),
        usize::from(published),
        "{report}"
    );
    for artifact in artifacts.iter().filter(|artifact| {
        artifact["family"]
            .as_str()
            .is_some_and(|name| name.starts_with("blob_"))
    }) {
        assert_eq!(artifact["outcome"]["posture"], "intact", "{artifact}");
    }
}

fn assert_recovered_bytes(
    blobs: &worth_store::physical_runtime::PhysicalBlobFacade<'_>,
    published: PublishedBlobGeneration,
    scope: &worth_store::physical_runtime::AdmittedBlobScope,
    limits: BlobReadLimits,
) {
    let mut read = blobs.read(published, scope, 0, 128, limits).unwrap();
    let mut observed = [0_u8; 128];
    let mut used = 0;
    while used < observed.len() {
        let count = read.read_next(&mut observed[used..]).unwrap();
        assert!(count > 0);
        used += count;
    }
    for (index, byte) in observed.iter().enumerate() {
        assert_eq!(*byte, expected_byte(0, index));
    }
}

fn park_at_marker(root: &Path, object: [u8; 16], session: [u8; 16]) -> ! {
    write_marker(&marker_path(root), object, session);
    loop {
        thread::park();
    }
}

pub(super) fn marker_path(root: &Path) -> PathBuf {
    root.parent().unwrap().join(MARKER)
}

pub(super) fn resume_token_path(root: &Path) -> PathBuf {
    root.parent().unwrap().join("blob-crash-resume-token")
}

pub(super) fn write_marker(path: &Path, object: [u8; 16], session: [u8; 16]) {
    let mut identity = [0_u8; 32];
    identity[..16].copy_from_slice(&object);
    identity[16..].copy_from_slice(&session);
    let pending = path.with_extension("pending");
    fs::write(&pending, identity).unwrap();
    fs::rename(pending, path).unwrap();
}
