#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_copy_observer::*;

use worth_store_physical_format::decode_blob_record;

use {
    crate::physical_blob_journeys::blob_abort::unfinished,
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

#[test]
fn held_historical_blob_copy_is_not_a_second_selected_chunk() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let serving = serving_from_initialization(root);
    let scope = admitted_blob_scope("c11.blob.copy.observer.provenance");
    let ingest = unfinished(&serving, &scope);
    drop(ingest);
    let selected = selected_blob_records(&serving);
    let (chunk, bytes) = selected
        .iter()
        .find(|(_, bytes)| {
            matches!(
                decode_blob_record(bytes),
                Ok(worth_store_physical_format::BlobRecordV1::Chunk(_))
            )
        })
        .expect("one real selected blob chunk");
    let chunk = *chunk;
    let bytes = bytes.clone();
    let source = current_extent_route(root, chunk);
    let retained_source_frame = extent_frame(root, source);

    // Only selection is certification-assisted. Source pin, arena transfer,
    // WAL, root publication, and unresolved custody use production owners.
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([113; 32]),
        )
        .unwrap();
    let request = worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
        key,
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
            .unwrap(),
    );
    let identity = worth_store_physical_format::PersistedRecordIdentity::new(
        chunk.allocation_epoch(),
        chunk.ordinal(),
    )
    .unwrap();
    let Ok(mut progress) = submission
        .certification_begin_selected_extent_copy(placement(), request, identity)
        .unwrap()
    else {
        panic!("actual selected blob extent must admit bounded copy");
    };
    for _ in 0..128 {
        if progress.phase
            == worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption
        {
            break;
        }
        progress = submission.advance_extent_copy().unwrap();
    }
    assert_eq!(
        progress.phase,
        worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption
    );
    let prepared = submission.prepare_completed_extent_copy().unwrap();
    assert!(matches!(
        prepared.execute(),
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_)
    ));
    let destination = current_extent_route(root, chunk);
    assert_ne!(source, destination, "the selected route must actually move");
    assert_eq!(
        extent_frame(root, source),
        retained_source_frame,
        "copy publication may not overwrite its protected old source frame"
    );
    assert!(
        extent_frame(root, destination)
            .iter()
            .any(|byte| *byte != 0),
        "the selected destination has a physical frame"
    );
    assert_eq!(
        selected_blob_records(&serving)
            .iter()
            .find(|(record, _)| *record == chunk)
            .unwrap()
            .1,
        bytes
    );
    let source_range = source.arena_range();
    let source_file = root.join(format!(
        "families/records/arenas/arena-{:016x}.data",
        source_range.arena().get()
    ));
    assert!(
        std::fs::metadata(source_file).unwrap().len() > source_range.offset(),
        "unresolved copy retains the original physical extent"
    );
    drop(submission);
    serving.close();

    let report = observe_closed_store_named(root, "c11-blob-copy", "held-source-provenance");
    assert_eq!(report["completeness"], "complete", "{report}");
    let chunk_rows: Vec<_> = report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["family"] == "blob_chunk_frame")
        .collect();
    assert_eq!(
        chunk_rows.len(),
        1,
        "proof-only old extent is not a selected claim: {report}"
    );
    let row = chunk_rows[0];
    assert_eq!(
        row["identity"],
        format!("blob-record:{}", hex_record(chunk))
    );
    assert_eq!(row["outcome"]["posture"], "intact", "{row}");
    assert_eq!(
        row["path"],
        format!(
            "families/records/arenas/arena-{:016x}.data",
            destination.arena_range().arena().get()
        ),
        "the blob claim must follow the selected destination"
    );
}
