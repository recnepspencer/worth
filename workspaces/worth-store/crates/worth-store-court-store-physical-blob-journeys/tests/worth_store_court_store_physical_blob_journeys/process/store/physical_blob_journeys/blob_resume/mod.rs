#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_resume::*;
use std::num::NonZeroU64;

use worth_store_blob_chunks::BlobChunkSize;
use {
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestClaimDenial,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobResumeFailure,
    worth_store::physical_runtime::BlobResumeLimits,
    worth_store::physical_runtime::BlobResumeToken,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalRecordId,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1};

use {
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::admitted_blob_scope_for_replay_boundary,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

#[test]
fn selected_prefix_resumes_once_and_denials_do_not_publish() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.resume.scope");
    let wrong_scope = admitted_blob_scope_for_replay_boundary("c11.blob.resume.wrong");
    let read_limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let resume_limits = worth_store::physical_runtime::BlobResumeLimits::new(
        std::num::NonZeroU64::new(128).unwrap(),
        std::num::NonZeroU64::new(1024 * 1024).unwrap(),
    );
    let blobs = serving.blobs().unwrap();
    let object = blobs.issue_object_id(read_limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(1024).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
            .unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK as u64, read_limits)
        .unwrap();
    let token = ingest.resume_token();
    let first = patterned_chunk(0);
    ingest.push(&first).unwrap();
    assert_eq!(ingest.frontier().bytes(), CHUNK as u64);
    let selected_prefix = selected_chunk_records(&serving);
    assert_eq!(selected_prefix.len(), 1);
    assert_eq!(selected_prefix[0].0, 0);

    assert!(matches!(
        blobs.resume_ingest(
            token,
            &scope,
            placement(),
            CHUNK as u64,
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
            resume_limits,
        ),
        Err(worth_store::physical_runtime::BlobResumeFailure::Claim(
            worth_store::physical_runtime::BlobIngestClaimDenial::CompetingSession
        ))
    ));
    drop(ingest);

    let unchanged_root = selected_root(&serving);
    for forged in forged_selected_mismatches(token) {
        assert!(matches!(
            blobs.resume_ingest(
                forged,
                &scope,
                placement(),
                CHUNK as u64,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(
                    120_000
                )
                .unwrap(),
                resume_limits,
            ),
            Err(worth_store::physical_runtime::BlobResumeFailure::DeclarationMismatch)
        ));
        assert_eq!(selected_root(&serving), unchanged_root);
    }
    assert!(matches!(
        blobs.resume_ingest(
            token,
            &wrong_scope,
            placement(),
            CHUNK as u64,
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
            resume_limits,
        ),
        Err(worth_store::physical_runtime::BlobResumeFailure::ScopeMismatch)
    ));
    assert_eq!(selected_root(&serving), unchanged_root);
    assert!(matches!(
        blobs.resume_ingest(
            token,
            &scope,
            placement(),
            CHUNK as u64,
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
            worth_store::physical_runtime::BlobResumeLimits::new(
                std::num::NonZeroU64::new(128).unwrap(),
                std::num::NonZeroU64::new(1).unwrap()
            ),
        ),
        Err(worth_store::physical_runtime::BlobResumeFailure::MetadataCapacity)
    ));
    assert_eq!(selected_root(&serving), unchanged_root);

    let mut resumed = blobs
        .resume_ingest(
            token,
            &scope,
            placement(),
            CHUNK as u64,
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
            resume_limits,
        )
        .unwrap();
    assert_eq!(resumed.frontier().bytes(), CHUNK as u64);
    assert_eq!(resumed.frontier().next_chunk_ordinal(), 1);
    let observation = resumed.resume_observation().unwrap();
    assert!(observation.scanned_records() >= 2);
    assert_eq!(observation.rehashed_chunks(), 1);
    assert_eq!(observation.rehashed_bytes(), CHUNK as u64);
    assert!(observation.metadata_capacity_bytes() <= resume_limits.metadata_bytes().get());
    assert_eq!(selected_chunk_records(&serving), selected_prefix);
    resumed.push(&patterned_chunk(1)).unwrap();
    let published = resumed.finish().unwrap();
    assert_eq!(
        published.session().bytes().as_slice(),
        &token.encode()[24..40]
    );
    let final_root = selected_root(&serving);
    assert!(matches!(
        blobs.resume_ingest(
            token,
            &scope,
            placement(),
            CHUNK as u64,
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
            resume_limits,
        ),
        Err(worth_store::physical_runtime::BlobResumeFailure::AlreadyPublished)
    ));
    assert_eq!(selected_root(&serving), final_root);
    let selected = selected_chunk_records(&serving);
    assert_eq!(selected.len(), 2);
    assert_eq!(selected[0], selected_prefix[0]);
    assert_eq!(selected[1].0, 1);
    assert_eq!(selected[0].2, first);
    assert_eq!(selected[1].2, patterned_chunk(1));
    assert_eq!(selected_root(&serving), final_root);
    serving.close();

    let reopened = serving_from_open(directory.path());
    let blobs = reopened.blobs().unwrap();
    let resolved = blobs
        .resolve_publication(object.bytes(), 1, &scope, read_limits)
        .unwrap();
    assert_eq!(resolved, published);
    let mut read = blobs
        .read(resolved, &scope, 0, (2 * CHUNK) as u64, read_limits)
        .unwrap();
    let mut actual = Vec::new();
    let mut frame = [0_u8; 4096];
    loop {
        let count = read.read_next(&mut frame).unwrap();
        if count == 0 {
            break;
        }
        actual.extend_from_slice(&frame[..count]);
    }
    assert_eq!(actual, [patterned_chunk(0), patterned_chunk(1)].concat());
    drop(read);
    reopened.close();
}
