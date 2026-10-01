use super::*;
use std::num::NonZeroU64;
use worth_store::physical_runtime::{
    certification::CertificationPhysicalMutationCheckpoint, BlobCheckpointLimit,
    BlobIngestDeclaration, BlobReadLimits, BlobReadOpenFailure, LayoutRebuildLimits,
    PhysicalLayoutDenial, RecordCountLimit, RecordScanOutcome, RecordScanRequest,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_contracts::DurableArtifactFamilyId;

#[path = "mixed_history/blob_scope.rs"]
mod blob_scope;
use super::copy_frame;

const BLOB_CHILD: &str = "mixed_history::blob_child";
const BLOB_MARKER: &str = "mixed-current-copy-blob-ready";
const CHUNK_BYTES: usize = 64 * 1024;
const SOURCE_WINDOW_BYTES: u64 = 32 * 1024;

#[test]
fn selected_current_source_copy_and_blob_generation_recover_together() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    kill_after_final_wal(directory.path());
    let baseline = std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap();
    assert_eq!(
        baseline,
        std::fs::read(directory.path().join("baseline-catalog")).unwrap(),
        "the durable final copy WAL has not published a root"
    );
    let record = load_identity(directory.path());
    let copy_lsn = copy_frame::current_copy_publication_lsn(&root);
    observe_copy_media(&root, directory.path(), "killed");

    let first = recover(&root);
    let source = selected_extent(&first, record);
    assert_eq!(
        generation(&baseline),
        first
            .selected_sources()
            .root()
            .selected()
            .selector()
            .root_generation(),
        "the first C8 view is the pre-redo selected root"
    );
    assert!(selected_wal_contains(&first, copy_lsn));
    drop(first);
    let copy_recovered = recover(&root);
    let destination = selected_extent(&copy_recovered, record);
    assert_eq!(destination.extent(), source.extent());
    assert_eq!(
        destination.extent_generation(),
        source.extent_generation() + 1
    );
    assert_ne!(
        destination.arena_range().arena(),
        source.arena_range().arena()
    );
    assert!(selected_wal_contains(&copy_recovered, copy_lsn));
    drop(copy_recovered);
    let stable_copy = recover(&root);
    assert_eq!(selected_extent(&stable_copy, record), destination);
    drop(stable_copy);
    copy_frame::assert_no_published_copy_resolution(&root);

    let (object, session) = kill_after_blob_generation_wal(directory.path());
    let blob_lsn = copy_frame::current_blob_generation_lsn(&root);
    assert!(copy_lsn < blob_lsn);
    let replayed = recover(&root);
    assert_eq!(selected_extent(&replayed, record), destination);
    assert!(selected_wal_contains(&replayed, copy_lsn));
    assert!(selected_wal_contains(&replayed, blob_lsn));
    drop(replayed);
    let settled = recover(&root);
    assert_eq!(selected_extent(&settled, record), destination);
    assert!(selected_wal_contains(&settled, copy_lsn));
    assert!(selected_wal_contains(&settled, blob_lsn));
    drop(settled);

    let serving = open(&root);
    assert_eq!(read_copied_record(&serving, record), vec![93; EXTENT_BYTES]);
    let scope = blob_scope::admitted_blob_scope();
    let limits = BlobReadLimits::new(NonZeroU64::new(1024).unwrap());
    let blobs = serving.blobs().unwrap();
    let latest = serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("C8 must restore the selected blob generation publication");
    let stale_latest = match blobs.resolve_publication(object, 1, &scope, limits) {
        Err(BlobReadOpenFailure::Layout(PhysicalLayoutDenial::IndexStaleRequiresRebuild {
            latest,
            indexed: None,
        })) => latest,
        other => panic!("recovered publication must precede derived-index rebuild: {other:?}"),
    };
    assert_eq!(stale_latest, latest);
    let rebuilt = serving
        .layouts()
        .unwrap()
        .rebuild(
            DurableArtifactFamilyId::BlobCatalog,
            LayoutRebuildLimits::new(
                NonZeroU64::new(1024).unwrap(),
                NonZeroU64::new(1024).unwrap(),
            ),
            configuration().1,
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        )
        .expect("Store must rebuild the derived catalog from selected publication authority");
    assert_eq!(rebuilt.source(), latest);
    assert_eq!(rebuilt.validated_chunks(), 1);
    let published = blobs
        .resolve_publication(object, 1, &scope, limits)
        .expect("current generation must publish during recovery");
    assert_eq!(published.session().bytes(), session);
    let mut read = blobs.read(published, &scope, 0, 128, limits).unwrap();
    let mut observed = [0; 128];
    let mut used = 0;
    while used < observed.len() {
        let count = read.read_next(&mut observed[used..]).unwrap();
        assert!(count > 0);
        used += count;
    }
    assert_eq!(observed, [42; 128]);
    drop((read, blobs));
    serving.close();
    let report = observe_arena_in_separate_process(
        &root,
        &directory.path().join("mixed-history.json"),
        "copy-current-blob-current",
        "recovered",
    );
    let artifacts = report["artifacts"].as_array().unwrap();
    assert_eq!(
        artifacts
            .iter()
            .filter(
                |artifact| artifact["family"] == "blob_generation_publication"
                    && artifact["outcome"]["posture"] == "intact"
            )
            .count(),
        1,
        "{report}"
    );
}

#[test]
#[ignore = "parent kills after current blob generation WAL durability"]
fn blob_child() {
    let directory = PathBuf::from(std::env::var_os(BLOB_MARKER).unwrap());
    let root = directory.join("store");
    let serving = open(&root);
    let scope = blob_scope::admitted_blob_scope();
    let limits = BlobReadLimits::new(NonZeroU64::new(1024).unwrap());
    let blobs = serving.blobs().unwrap();
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        CHUNK_BYTES as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(1024).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, configuration().1, SOURCE_WINDOW_BYTES, limits)
        .expect("selected current copy must permit blob ingest");
    let session = ingest.session_id().bytes();
    ingest
        .push(&vec![42; SOURCE_WINDOW_BYTES as usize])
        .unwrap();
    ingest
        .push(&vec![42; SOURCE_WINDOW_BYTES as usize])
        .unwrap();
    let leaf_gate = serving.certification_pause_physical_mutation_at(
        CertificationPhysicalMutationCheckpoint::AfterWalDurability,
    );
    let marker = directory.join(BLOB_MARKER);
    thread::scope(|workers| {
        workers.spawn(|| {
            let deadline = Instant::now() + Duration::from_secs(120);
            while Instant::now() < deadline {
                if leaf_gate.await_arrival() {
                    let publication_gate = serving.certification_pause_physical_mutation_at(
                        CertificationPhysicalMutationCheckpoint::AfterWalDurability,
                    );
                    leaf_gate.release();
                    while Instant::now() < deadline {
                        if publication_gate.await_arrival() {
                            let mut identity = object.bytes().to_vec();
                            identity.extend_from_slice(&session);
                            let temporary = marker.with_extension("tmp");
                            std::fs::write(&temporary, identity).unwrap();
                            std::fs::rename(temporary, marker).unwrap();
                            return;
                        }
                    }
                    panic!("generation publication did not reach durable WAL");
                }
            }
            panic!("tree leaf did not reach durable WAL");
        });
        let _ = ingest.finish();
        panic!("generation publication escaped WAL durability pause");
    });
}

fn kill_after_blob_generation_wal(directory: &Path) -> ([u8; 16], [u8; 16]) {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", BLOB_CHILD, "--ignored", "--nocapture"])
        .env(BLOB_MARKER, directory)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let marker = directory.join(BLOB_MARKER);
    let deadline = Instant::now() + Duration::from_secs(120);
    while !marker.is_file() {
        if child.try_wait().unwrap().is_some() || Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "generation seam not reached: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    let identity = std::fs::read(marker).unwrap();
    assert_eq!(identity.len(), 32);
    child.kill().unwrap();
    child.wait().unwrap();
    (
        identity[..16].try_into().unwrap(),
        identity[16..32].try_into().unwrap(),
    )
}

fn selected_wal_contains(handoff: &RecoveredPhysicalRuntimeHandoff, lsn: u64) -> bool {
    handoff
        .selected_sources()
        .wal_tail()
        .frame_facts()
        .any(|frame| {
            frame.lsn_range().start().get() <= lsn && lsn < frame.lsn_range().end_exclusive().get()
        })
}

fn read_copied_record(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    record: PersistedRecordIdentity,
) -> Vec<u8> {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(RecordScanRequest::from_start().with_batch_limit(RecordCountLimit::new(1).unwrap()))
        .unwrap();
    let mut scratch = vec![0; 128_000];
    let mut selected = None;
    while let RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
        for row in batch.records() {
            let id = row.record_id();
            if id.allocation_epoch() == record.allocation_epoch()
                && id.ordinal() == record.ordinal()
            {
                assert!(selected.replace(id).is_none(), "copied record appears once");
            }
        }
        if batch.is_complete() {
            break;
        }
    }
    drop(scan);
    read_record(serving, selected.expect("copied record remains readable"))
}
