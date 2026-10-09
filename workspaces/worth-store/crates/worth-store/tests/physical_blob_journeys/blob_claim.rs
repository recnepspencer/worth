use std::{num::NonZeroU64, sync::Barrier, thread};

use worth_store::physical_runtime::{
    AdmittedBlobScope, BlobCheckpointLimit, BlobIngestDeclaration, BlobIngestFailure, BlobObjectId,
    BlobReadLimits, PhysicalMutationDeadline,
};
use worth_store_blob_chunks::BlobChunkSize;

use super::fixture::{
    admitted_blob_scope, placement, serving_from_initialization, serving_from_open,
};

const CHUNK_BYTES: u64 = 64 * 1024;

fn declaration(object: BlobObjectId, scope: &AdmittedBlobScope) -> BlobIngestDeclaration {
    BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK_BYTES).unwrap(),
        2 * CHUNK_BYTES,
        scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    )
    .unwrap()
}

#[test]
fn selected_declaration_is_a_durable_single_object_claim() {
    let parent = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.claim.scope");
    let limits = BlobReadLimits::new(NonZeroU64::new(32).unwrap());
    let serving = serving_from_initialization(parent.path());
    let blobs = serving.blobs().unwrap();
    let object = blobs.issue_object_id(limits).unwrap();
    let first = blobs
        .begin_ingest(
            declaration(object, &scope),
            placement(),
            CHUNK_BYTES,
            limits,
        )
        .unwrap();
    assert!(matches!(
        blobs.begin_ingest(
            declaration(object, &scope),
            placement(),
            CHUNK_BYTES,
            limits
        ),
        Err(BlobIngestFailure::ObjectAlreadyDeclared)
    ));
    drop(first);
    drop(blobs);
    serving.close();

    let reopened = serving_from_open(parent.path());
    let blobs = reopened.blobs().unwrap();
    assert!(matches!(
        blobs.begin_ingest(
            declaration(object, &scope),
            placement(),
            CHUNK_BYTES,
            limits
        ),
        Err(BlobIngestFailure::ObjectAlreadyDeclared)
    ));
    drop(blobs);
    reopened.close();
}

#[test]
fn racing_declarations_of_one_object_admit_exactly_one() {
    let parent = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.claim.concurrent.scope");
    let limits = BlobReadLimits::new(NonZeroU64::new(32).unwrap());
    let serving = serving_from_initialization(parent.path());
    let object = serving.blobs().unwrap().issue_object_id(limits).unwrap();
    let barrier = Barrier::new(2);
    let outcomes = thread::scope(|threads| {
        let attempt = || {
            barrier.wait();
            let blobs = serving.blobs().unwrap();
            match blobs.begin_ingest(
                declaration(object, &scope),
                placement(),
                CHUNK_BYTES,
                limits,
            ) {
                Ok(session) => {
                    drop(session);
                    1_u8
                }
                Err(BlobIngestFailure::ObjectAlreadyDeclared) => 2_u8,
                Err(error) => panic!("unexpected same-object claim fate: {error:?}"),
            }
        };
        let first = threads.spawn(attempt);
        let second = threads.spawn(attempt);
        [first.join().unwrap(), second.join().unwrap()]
    });
    assert!(
        outcomes.contains(&1),
        "one declaration must commit: {outcomes:?}"
    );
    assert!(
        outcomes.contains(&2),
        "one contender must be denied: {outcomes:?}"
    );
    serving.close();
}
