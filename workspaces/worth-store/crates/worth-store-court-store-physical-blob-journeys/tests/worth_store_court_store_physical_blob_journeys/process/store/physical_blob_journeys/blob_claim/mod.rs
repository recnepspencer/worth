#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_claim::*;
use {std::num::NonZeroU64, std::sync::Barrier, std::thread};

use worth_store_blob_chunks::BlobChunkSize;
use {
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobObjectId,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::PhysicalMutationDeadline,
};

use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
};

#[test]
fn selected_declaration_is_a_durable_single_object_claim() {
    let parent = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.claim.scope");
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(32).unwrap());
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
        Err(worth_store::physical_runtime::BlobIngestFailure::ObjectAlreadyDeclared)
    ));
    drop(first);
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
        Err(worth_store::physical_runtime::BlobIngestFailure::ObjectAlreadyDeclared)
    ));
    reopened.close();
}

#[test]
fn racing_declarations_of_one_object_admit_exactly_one() {
    let parent = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.claim.concurrent.scope");
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(32).unwrap());
    let serving = serving_from_initialization(parent.path());
    let object = serving.blobs().unwrap().issue_object_id(limits).unwrap();
    let barrier = std::sync::Barrier::new(2);
    let outcomes = std::thread::scope(|threads| {
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
                Err(worth_store::physical_runtime::BlobIngestFailure::ObjectAlreadyDeclared) => {
                    2_u8
                }
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
