#[allow(unused_imports)] use crate::physical_blob_journeys::blob_unread_record_observation::*;
use {std::collections::BTreeMap, std::fs, std::num::NonZeroU64, std::path::Path};

use {worth_store::physical_runtime::AdmittedBlobScope, worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::ServingPhysicalRuntime};
use {worth_store_blob_chunks::BlobChunkSize};

use {crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named, crate::physical_blob_journeys::blob_ingest_process::observed_without_damage};
use {worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization};

#[test]
fn a_record_whose_extent_manifest_is_damaged_keeps_its_row_and_excuses_no_other() {
    a_record_with_a_damaged_frame_keeps_its_row_and_excuses_no_other(
        "damaged-manifest",
        crate::physical_blob_journeys::blob_unread_record_observation::DamagedFrame::ExtentManifest,
    );
}

#[test]
fn a_record_whose_first_extent_chunk_is_damaged_keeps_its_row_and_excuses_no_other() {
    a_record_with_a_damaged_frame_keeps_its_row_and_excuses_no_other(
        "damaged-first-chunk",
        crate::physical_blob_journeys::blob_unread_record_observation::DamagedFrame::FirstExtentChunk,
    );
}

/// The observer reads a file once. An arena that is a second name of a file
/// it has already seen is not reinspected, so no record in it was read. Each
/// record keeps its row, of the family its route declares, as not reinspected:
/// none is absent, and none is damaged for what it claims of another.
#[test]
fn records_of_an_arena_that_is_not_reinspected_keep_their_rows_undamaged() {
    let directory = tempfile::tempdir().unwrap();
    let clean = two_objects_of_one_chunk(directory.path(), "aliased-arena");
    std::fs::hard_link(
        directory.path().join(ARENA),
        directory.path().join("namespace/arena-alias"),
    )
    .expect("same-volume hard-link fixture must be supported");

    let artifacts = observed_without_damage(directory.path(), RUN, "aliased-arena");
    let observed = blob_records(&artifacts);
    assert_eq!(
        observed.keys().collect::<Vec<_>>(),
        clean.keys().collect::<Vec<_>>(),
        "every routed blob record keeps one row"
    );
    for (record, row) in &observed {
        assert_eq!(row["family"], clean[record]["family"], "{row}");
        assert_eq!(row["outcome"]["posture"], "unknown", "{row}");
        assert_eq!(
            row["outcome"]["reason"], "physical_alias_not_reinspected",
            "{row}"
        );
    }
}
