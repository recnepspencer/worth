use std::num::NonZeroU64;

use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobReadLimits, PhysicalMutationDeadline,
};
use worth_store_blob_chunks::BlobChunkSize;

use super::fixture::{admitted_blob_scope, placement, serving_from_initialization};

#[test]
fn tier_epoch_maintenance_creates_a_wal_segment_for_the_next_blob_declaration() {
    const CHUNK: u64 = 64 << 10;
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let initial = serving.record_submission().wal_observation().unwrap();
    assert_eq!(initial.active_segment_count(), 0);

    serving
        .certification_activate_tier_epoch(placement())
        .unwrap();
    let activated = serving.record_submission().wal_observation().unwrap();
    assert_eq!(activated.active_segment_count(), 1);
    assert!(activated.valid_prefix_bytes() > 0);
    let activated_lsn_end = activated.last_lsn_end().unwrap();

    let blobs = serving.blobs().unwrap();
    let scope = admitted_blob_scope("c11.blob.tier-epoch.wal-append");
    let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK).unwrap(),
        CHUNK,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    )
    .unwrap();
    let ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK / 2, limits)
        .unwrap();
    let declared = serving.record_submission().wal_observation().unwrap();
    assert_eq!(declared.active_segment_count(), 1);
    assert!(declared.valid_prefix_bytes() > activated.valid_prefix_bytes());
    assert!(declared.last_lsn_end().unwrap() > activated_lsn_end);
    drop(ingest);
    serving.close();
}
