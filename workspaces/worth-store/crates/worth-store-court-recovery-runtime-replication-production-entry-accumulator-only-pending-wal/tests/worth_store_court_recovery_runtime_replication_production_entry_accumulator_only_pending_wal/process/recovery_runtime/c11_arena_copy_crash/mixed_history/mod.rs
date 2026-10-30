#[allow(unused_imports)]
use crate::c11_arena_copy_crash::mixed_history::*;
use std::num::NonZeroU64;
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_contracts::DurableArtifactFamilyId;
use {
    crate::c11_arena_copy_crash::kill_after_final_wal, crate::c11_arena_copy_crash::load_identity,
    crate::c11_arena_copy_crash::observation::observe_copy_media,
    crate::c11_arena_copy_crash::recover, crate::c11_arena_copy_crash::selected_extent,
    std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio, std::thread,
    std::time::Duration, std::time::Instant,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::ServingPhysicalRuntime,
    worth_store_physical_format::PersistedRecordIdentity,
    worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff,
    worth_store_testkit_recovery_runtime::support::configuration,
    worth_store_testkit_recovery_runtime::support::generation,
    worth_store_testkit_recovery_runtime::support::observe_arena_in_separate_process,
    worth_store_testkit_recovery_runtime::support::open,
    worth_store_testkit_recovery_runtime::support::read_record,
    worth_store_testkit_recovery_runtime::support::scan,
    worth_store_testkit_recovery_runtime::support::EXTENT_BYTES,
};
use {
    worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReadOpenFailure,
    worth_store::physical_runtime::LayoutRebuildLimits,
    worth_store::physical_runtime::PhysicalLayoutDenial,
    worth_store::physical_runtime::RecordCountLimit,
    worth_store::physical_runtime::RecordScanOutcome,
    worth_store::physical_runtime::RecordScanRequest,
};

use crate::copy_frame;

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
    let copy_lsn = crate::copy_frame::current_copy_publication_lsn(&root);
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
    crate::copy_frame::assert_no_published_copy_resolution(&root);

    let (object, session) = kill_after_blob_generation_wal(directory.path());
    let blob_lsn = crate::copy_frame::current_blob_generation_lsn(&root);
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
    let scope = crate::mixed_history::blob_scope::admitted_blob_scope();
    let limits = worth_store::physical_runtime::BlobReadLimits::new(
        std::num::NonZeroU64::new(1024).unwrap(),
    );
    let blobs = serving.blobs().unwrap();
    let latest = serving
        .certification_selected_latest_blob_publication()
        .unwrap()
        .expect("C8 must restore the selected blob generation publication");
    let stale_latest = match blobs.resolve_publication(object, 1, &scope, limits) {
        Err(worth_store::physical_runtime::BlobReadOpenFailure::Layout(
            worth_store::physical_runtime::PhysicalLayoutDenial::IndexStaleRequiresRebuild {
                latest,
                indexed: None,
            },
        )) => latest,
        other => panic!("recovered publication must precede derived-index rebuild: {other:?}"),
    };
    assert_eq!(stale_latest, latest);
    let rebuilt = serving
        .layouts()
        .unwrap()
        .rebuild(
            worth_store_contracts::DurableArtifactFamilyId::BlobCatalog,
            worth_store::physical_runtime::LayoutRebuildLimits::new(
                std::num::NonZeroU64::new(1024).unwrap(),
                std::num::NonZeroU64::new(1024).unwrap(),
            ),
            configuration().1,
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                .unwrap(),
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
    let directory = std::path::PathBuf::from(std::env::var_os(BLOB_MARKER).unwrap());
    let root = directory.join("store");
    let serving = open(&root);
    let scope = crate::mixed_history::blob_scope::admitted_blob_scope();
    let limits = worth_store::physical_runtime::BlobReadLimits::new(
        std::num::NonZeroU64::new(1024).unwrap(),
    );
    let blobs = serving.blobs().unwrap();
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        CHUNK_BYTES as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(1024).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
            .unwrap(),
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
        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterWalDurability,
    );
    let marker = directory.join(BLOB_MARKER);
    std::thread::scope(|workers| {
        workers.spawn(|| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
            while std::time::Instant::now() < deadline {
                if leaf_gate.await_arrival() {
                    let publication_gate = serving.certification_pause_physical_mutation_at(
                        worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterWalDurability,
                    );
                    leaf_gate.release();
                    while std::time::Instant::now() < deadline {
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
