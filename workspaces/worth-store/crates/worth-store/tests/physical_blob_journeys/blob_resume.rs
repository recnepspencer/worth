use std::num::NonZeroU64;

use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestClaimDenial, BlobIngestDeclaration, BlobReadLimits,
    BlobResumeFailure, BlobResumeLimits, BlobResumeToken, PhysicalMutationDeadline,
    PhysicalRecordId, ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::{decode_blob_record, BlobRecordV1};

use super::{
    blob_frontier::selected_blob_records,
    fixture::{
        admitted_blob_scope, admitted_blob_scope_for_replay_boundary, placement,
        serving_from_initialization, serving_from_open,
    },
};

const CHUNK: usize = 64 * 1024;

#[test]
fn selected_prefix_resumes_once_and_denials_do_not_publish() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.resume.scope");
    let wrong_scope = admitted_blob_scope_for_replay_boundary("c11.blob.resume.wrong");
    let read_limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let resume_limits = BlobResumeLimits::new(
        NonZeroU64::new(128).unwrap(),
        NonZeroU64::new(1024 * 1024).unwrap(),
    );
    let blobs = serving.blobs().unwrap();
    let object = blobs.issue_object_id(read_limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (2 * CHUNK) as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(1024).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
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
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            resume_limits,
        ),
        Err(BlobResumeFailure::Claim(
            BlobIngestClaimDenial::CompetingSession
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
                PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
                resume_limits,
            ),
            Err(BlobResumeFailure::DeclarationMismatch)
        ));
        assert_eq!(selected_root(&serving), unchanged_root);
    }
    assert!(matches!(
        blobs.resume_ingest(
            token,
            &wrong_scope,
            placement(),
            CHUNK as u64,
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            resume_limits,
        ),
        Err(BlobResumeFailure::ScopeMismatch)
    ));
    assert_eq!(selected_root(&serving), unchanged_root);
    assert!(matches!(
        blobs.resume_ingest(
            token,
            &scope,
            placement(),
            CHUNK as u64,
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            BlobResumeLimits::new(NonZeroU64::new(128).unwrap(), NonZeroU64::new(1).unwrap()),
        ),
        Err(BlobResumeFailure::MetadataCapacity)
    ));
    assert_eq!(selected_root(&serving), unchanged_root);

    let mut resumed = blobs
        .resume_ingest(
            token,
            &scope,
            placement(),
            CHUNK as u64,
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
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
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            resume_limits,
        ),
        Err(BlobResumeFailure::AlreadyPublished)
    ));
    assert_eq!(selected_root(&serving), final_root);
    let selected = selected_chunk_records(&serving);
    assert_eq!(selected.len(), 2);
    assert_eq!(selected[0], selected_prefix[0]);
    assert_eq!(selected[1].0, 1);
    assert_eq!(selected[0].2, first);
    assert_eq!(selected[1].2, patterned_chunk(1));
    assert_eq!(selected_root(&serving), final_root);
    drop(blobs);
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
    drop(blobs);
    reopened.close();
}

fn selected_root(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .records()
        .unwrap()
        .protected_root()
        .root()
        .generation()
        .get()
}

fn selected_chunk_records(
    serving: &ServingPhysicalRuntime,
) -> Vec<(u64, PhysicalRecordId, Vec<u8>)> {
    let mut chunks = selected_blob_records(serving)
        .into_iter()
        .filter_map(|(record, bytes)| match decode_blob_record(&bytes) {
            Ok(BlobRecordV1::Chunk(chunk)) => {
                Some((chunk.occurrence().ordinal(), record, chunk.bytes().to_vec()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    chunks.sort_by_key(|(ordinal, _, _)| *ordinal);
    chunks
}

fn forged_selected_mismatches(token: BlobResumeToken) -> [BlobResumeToken; 4] {
    let original = token.encode();
    let mut forged = [token; 4];
    for (index, offset) in [64, 96, 100, 108].into_iter().enumerate() {
        let mut wire = original;
        if offset == 96 {
            wire[96..100].copy_from_slice(&(128 * 1024_u32).to_le_bytes());
        } else {
            wire[offset] ^= 0x01;
        }
        forged[index] = BlobResumeToken::decode(&wire)
            .expect("each modified token must remain syntactically valid");
    }
    forged
}

fn patterned_chunk(ordinal: usize) -> Vec<u8> {
    (0..CHUNK)
        .map(|index| (ordinal as u8).wrapping_mul(17).wrapping_add(index as u8))
        .collect()
}
