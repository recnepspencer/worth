#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_frontier::reused_last_chunk::*;
use {std::num::NonZeroU64, std::path::Path};

use worth_store_blob_chunks::BlobChunkSize;
use {
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobIngestSession,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::PhysicalBlobFacade,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1};

use {
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_with_limits,
    crate::physical_blob_journeys::blob_ingest_process::observed_without_damage,
    crate::physical_blob_journeys::blob_ingest_process::selected_root,
};
use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
};
use {
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::assert_record_binding,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::CHUNK,
};

#[test]
fn unfinished_session_whose_sixty_fourth_chunk_is_reused_observes_clean() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.frontier.reused-last-chunk.scope");
    let serving = serving_from_initialization(directory.path());
    publish_reused_chunk(&serving, &scope);

    let blobs = serving.blobs().unwrap();
    let mut ingest = begin(&blobs, &scope, 66);
    let mut half = [0_u8; CHUNK / 2];
    for ordinal in 0..=REUSED_FILL {
        half.fill(ordinal);
        ingest.push(&half).unwrap();
        ingest.push(&half).unwrap();
    }
    assert_eq!(ingest.frontier().next_chunk_ordinal(), 64);
    drop(ingest);
    serving.close();

    let reopened = serving_from_open(directory.path());
    let mut frontier = None;
    let mut reuse = None;
    for (record, bytes) in selected_blob_records(&reopened) {
        match decode_blob_record(&bytes) {
            Ok(worth_store_physical_format::BlobRecordV1::SessionFrontier(value)) => {
                assert!(frontier.replace(value).is_none(), "one automatic frontier");
            }
            Ok(worth_store_physical_format::BlobRecordV1::ChunkReuseClaimV2(value)) => {
                assert!(reuse.replace((record, value.claim())).is_none());
            }
            _ => {}
        }
    }
    reopened.close();
    let frontier = frontier.expect("the 64th chunk publishes the automatic frontier");
    let (claim_record, claim) = reuse.expect("the 64th chunk reuses the published chunk");
    assert_eq!(claim.destination_session(), frontier.session());
    assert_eq!(claim.destination_ordinal(), 63);
    assert_eq!(frontier.next_chunk_ordinal(), 64);
    assert_record_binding(claim_record, frontier.last_chunk_record());
    assert_ne!(
        frontier.last_chunk_record(),
        claim.selected_chunk(),
        "the frontier names the claim row, not the chunk it reuses"
    );

    let artifacts = observed_clean(directory.path(), "reused-sixty-fourth-chunk");
    assert_eq!(
        count(&artifacts, "blob_resume_session"),
        4,
        "two declarations, one reuse claim and one frontier"
    );
    assert_eq!(
        count(&artifacts, "blob_chunk_frame"),
        64,
        "the published chunk and 63 chunks of the unfinished session"
    );
}

#[test]
fn published_objects_made_of_a_reused_chunk_observe_clean() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.frontier.reused-published.scope");
    let serving = serving_from_initialization(directory.path());
    publish_reused_chunk(&serving, &scope);
    publish_reused_chunk(&serving, &scope);
    publish_reused_chunk(&serving, &scope);
    let reuse_claims = selected_blob_records(&serving)
        .into_iter()
        .filter(|(_, bytes)| {
            matches!(
                decode_blob_record(bytes),
                Ok(worth_store_physical_format::BlobRecordV1::ChunkReuseClaimV2(_))
            )
        })
        .count();
    assert_eq!(reuse_claims, 2, "both later objects reuse the first chunk");
    serving.close();

    let artifacts = observed_clean(directory.path(), "reused-published-chunk");
    assert_eq!(count(&artifacts, "blob_chunk_frame"), 1);
    assert_eq!(count(&artifacts, "blob_tree_node"), 3);
    assert_eq!(count(&artifacts, "blob_generation_publication"), 3);
}

/// A walk that stops at an entry bound has not visited every routed record. A
/// claim on a record that it did not visit is undecided, not damaged: this
/// store is healthy under every bound.
#[test]
fn a_walk_stopped_at_any_entry_bound_damages_no_record_of_a_reused_chunk() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.frontier.reused-entry-bound.scope");
    let serving = serving_from_initialization(directory.path());
    publish_reused_chunk(&serving, &scope);
    publish_reused_chunk(&serving, &scope);
    serving.close();

    let run = "c11-blob-frontier";
    let complete = observe_closed_store_named(directory.path(), run, "reused-chunk-unbounded");
    assert_eq!(complete["completeness"], "complete", "{complete}");
    let entries = complete["consumed"]["entries"].as_u64().unwrap();
    let mut undecided_bounds = 0;
    for bound in 1..entries {
        let report = observe_closed_store_with_limits(
            directory.path(),
            run,
            "reused-chunk-entry-bound",
            bound,
            268_435_456,
            2_097_152,
            600_000,
        );
        let artifacts = report["artifacts"].as_array().unwrap();
        let blob_records = artifacts.iter().filter(|artifact| {
            let identity = artifact["identity"].as_str();
            identity.is_some_and(|identity| identity.starts_with("blob-record:"))
        });
        let mut undecided = false;
        for artifact in blob_records {
            let posture = &artifact["outcome"]["posture"];
            assert_ne!(posture, "damaged", "entry bound {bound}: {artifact}");
            undecided |= posture != "intact";
        }
        undecided_bounds += u64::from(undecided);
    }
    assert_ne!(
        undecided_bounds, 0,
        "no bound below {entries} stopped the walk among the blob records"
    );
}
