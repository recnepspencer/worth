pub(crate) use super::marker::{marker_path, park_at_marker, resume_token_path, write_marker};
pub(crate) use {
    std::fs, std::num::NonZeroU64, std::path::Path, std::path::PathBuf, std::process::Command,
    std::process::Stdio, std::thread, std::time::Duration, std::time::Instant,
};

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_store_blob_chunks::BlobChunkSize;
pub(crate) use worth_store_contracts::DurableArtifactFamilyId;
pub(crate) use {
    worth_store::physical_runtime::production::PhysicalMutationCheckpoint,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReadOpenFailure,
    worth_store::physical_runtime::LayoutRebuildLimits,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalLayoutDenial,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PublishedBlobGeneration,
    worth_store::physical_runtime::RecordAppendBatch,
};

pub(crate) use {
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
};

pub(crate) use {
    crate::blob_crash::recovery::recover_closed_store,
    crate::blob_crash::recovery::recover_closed_store_with_profile,
};

pub(crate) use {
    crate::blob_crash::chunk_bytes::expected_byte, crate::blob_crash::chunk_bytes::fill_chunk,
};

pub(crate) const CHUNK_BYTES: usize = 64 * 1024;
pub(crate) const CRASH_OBJECT_CHUNKS: usize = 3;
pub(crate) const SCOPE_KEY: &str = "c11.blob.crash.scope";
pub(crate) const CHILD_ROLE: &str = "WORTH_STORE_C11_BLOB_CHILD_ROLE";
pub(crate) const CHILD_ROOT: &str = "WORTH_STORE_C11_BLOB_CHILD_ROOT";
pub(crate) const CHILD_TEST: &str = "process::store::physical_blob_journeys::blob_child_role";

pub(crate) fn child(root: &Path, role: &str) {
    let serving = serving_from_initialization(root);
    establish_recovery_frontier(&serving);
    let scope = admitted_blob_scope(SCOPE_KEY);
    let limits = worth_store::physical_runtime::BlobReadLimits::new(
        std::num::NonZeroU64::new(1024).unwrap(),
    );
    let blobs = serving.blobs().unwrap();
    let object = blobs
        .issue_object_id(limits)
        .unwrap_or_else(|_| panic!("Store must issue an unoccupied object identity"));
    let total = if role == "crash-generation-wal" {
        CRASH_OBJECT_CHUNKS * CHUNK_BYTES
    } else {
        2 * CHUNK_BYTES
    };
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        total as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(1024).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
            .unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK_BYTES as u64, limits)
        .unwrap_or_else(|_| panic!("durable declaration must complete before crash seam"));
    let token_path = resume_token_path(root);
    let pending_token = token_path.with_extension("pending");
    std::fs::write(&pending_token, ingest.resume_token().encode()).unwrap();
    std::fs::rename(pending_token, token_path).unwrap();
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
            crate::physical_blob_journeys::blob_resume_tree::park_after_leaf_root(
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
                serving.pause_physical_mutation_at(worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterWalDurability);
            let marker = marker_path(root);
            std::thread::scope(|workers| {
                workers.spawn(|| {
                    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(180);
                    while std::time::Instant::now() < deadline {
                        if leaf_gate.await_arrival() {
                            let publication_gate = serving.pause_physical_mutation_at(
                                worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterWalDurability,
                            );
                            leaf_gate.release();
                            while std::time::Instant::now() < deadline {
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

pub(crate) fn establish_recovery_frontier(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
) {
    // Recovery before the first checkpoint denies reclaim and maintenance
    // frames, so these worlds checkpoint before any blob effect. A fresh
    // root has no durable WAL source to checkpoint, so one seed record
    // comes first.
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0xc0; 32]),
        )
        .unwrap();
    let worth_proof::TransitionOutcome::Success(
        worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
    ) = submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                b"c11-recovery-frontier".as_slice(),
            ])
            .unwrap(),
            placement(),
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
            ),
        )
        .into_raw()
    else {
        panic!("baseline C5 record must prepare before checkpoint");
    };
    let worth_store::physical_runtime::PhysicalMutationOutcome::Completed(seed) =
        prepared.execute()
    else {
        panic!("baseline C5 record must commit before checkpoint");
    };
    assert_eq!(seed.persisted_records().len(), 1);
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0xc1; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(checkpoint) =
        serving.checkpoints().start(request).into_raw()
    else {
        panic!("baseline C8 recovery checkpoint must admit before blob effects");
    };
    assert!(
        matches!(
            checkpoint.wait(),
            worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
        ),
        "baseline C8 recovery checkpoint must complete before blob effects"
    );
}

pub(crate) struct KilledBlobWorld {
    pub(crate) _directory: tempfile::TempDir,
    pub(crate) root: PathBuf,
    pub(crate) object: [u8; 16],
    pub(crate) session: [u8; 16],
    pub(crate) seam: &'static str,
}

pub(crate) fn kill_at(seam: &'static str, timeout: Duration) -> KilledBlobWorld {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    std::fs::create_dir(&root).unwrap();
    let (object, session) = kill_child_at(&root, seam, timeout);
    KilledBlobWorld {
        _directory: directory,
        root,
        object,
        session,
        seam,
    }
}

pub(crate) fn kill_child_at(
    root: &Path,
    seam: &'static str,
    timeout: Duration,
) -> ([u8; 16], [u8; 16]) {
    let marker = marker_path(root);
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
        .env(CHILD_ROLE, seam)
        .env(CHILD_ROOT, root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + timeout;
    while !marker.is_file() {
        if child.try_wait().unwrap().is_some() || std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "blob {seam} seam not reached: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let marked = std::fs::read(marker).unwrap();
    assert_eq!(marked.len(), 32, "marker must bind object and session");
    child.kill().unwrap();
    child.wait().unwrap();
    (
        marked[..16].try_into().unwrap(),
        marked[16..32].try_into().unwrap(),
    )
}

pub(crate) fn assert_reopened(world: &KilledBlobWorld, published: bool, chunks: usize) {
    recover_closed_store(&world.root);
    let serving = serving_from_open(&world.root);
    let scope = admitted_blob_scope(SCOPE_KEY);
    let limits = worth_store::physical_runtime::BlobReadLimits::new(
        std::num::NonZeroU64::new(1024).unwrap(),
    );
    let blobs = serving.blobs().unwrap();
    if published {
        let latest = serving
            .certification_selected_latest_blob_publication()
            .unwrap()
            .expect("C8 must restore the generation marker");
        assert!(
            matches!(
                blobs.resolve_publication(world.object, 1, &scope, limits),
                Err(worth_store::physical_runtime::BlobReadOpenFailure::Layout(
                    worth_store::physical_runtime::PhysicalLayoutDenial::IndexStaleRequiresRebuild { indexed: None, .. }
                ))
            ),
            "C8 publishes authority before the derived catalog exists"
        );
        let rebuilt = serving
            .layouts()
            .unwrap()
            .rebuild(
                worth_store_contracts::DurableArtifactFamilyId::BlobCatalog,
                worth_store::physical_runtime::LayoutRebuildLimits::new(
                    std::num::NonZeroU64::new(1024).unwrap(),
                    std::num::NonZeroU64::new(1024).unwrap(),
                ),
                placement(),
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
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
            Err(worth_store::physical_runtime::BlobReadOpenFailure::PublicationNotFound)
        ));
    }
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

pub(crate) fn assert_recovered_bytes(
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
