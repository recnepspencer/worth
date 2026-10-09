#![cfg(feature = "certification-test-authority")]

use std::num::NonZeroU64;

use sha2::{Digest, Sha256};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobReadLimits, PhysicalMutationDeadline,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationProvenNoEffectCause,
    PhysicalMutationRequest, RecordAppendDenial,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::{
    decode_blob_record, BlobChunkReuseClaimV1, BlobChunkReuseClaimV2, BlobGenerationPublicationV1,
    BlobRecordV1, PersistedRecordIdentity,
};

use super::{
    blob_frontier::selected_blob_records,
    fixture::{admitted_blob_scope, placement, serving_from_initialization},
};

const CHUNK: usize = 64 * 1024;

#[test]
fn classified_reuse_claim_denies_wrong_selected_destination_before_effect() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.layout.reuse.destination");
    let limits = BlobReadLimits::new(NonZeroU64::new(256).unwrap());
    let deadline = PhysicalMutationDeadline::after_milliseconds(30_000).unwrap();
    let blobs = serving.blobs().unwrap();
    let source_object = blobs.issue_object_id(limits).unwrap();
    let source_declaration = BlobIngestDeclaration::new(
        source_object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        CHUNK as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        deadline,
    )
    .unwrap();
    let mut source_ingest = blobs
        .begin_ingest(source_declaration, placement(), (CHUNK / 2) as u64, limits)
        .unwrap();
    let source_bytes = vec![0x63; CHUNK];
    source_ingest.push(&source_bytes[..CHUNK / 2]).unwrap();
    source_ingest.push(&source_bytes[CHUNK / 2..]).unwrap();
    source_ingest.finish().unwrap();

    let selected = selected_blob_records(&serving);
    let mut source_chunk = None;
    let mut source_publication = None;
    let mut source_publication_frame = None;
    let mut digest = None;
    for (record, bytes) in &selected {
        let Ok(blob) = decode_blob_record(bytes) else {
            continue;
        };
        match blob {
            BlobRecordV1::Chunk(chunk) => {
                source_chunk = Some(persisted(*record));
                digest = Some(chunk.stored_digest());
            }
            BlobRecordV1::GenerationPublished(publication) => {
                source_publication = Some(persisted(*record));
                source_publication_frame = Some(publication);
            }
            _ => {}
        }
    }
    let destination_object = blobs.issue_object_id(limits).unwrap();
    let destination_declaration = BlobIngestDeclaration::new(
        destination_object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        CHUNK as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        deadline,
    )
    .unwrap();
    let destination = blobs
        .begin_ingest(
            destination_declaration,
            placement(),
            (CHUNK / 2) as u64,
            limits,
        )
        .unwrap();
    let token = destination.resume_token().encode();
    let declaration_record = PersistedRecordIdentity::new(
        token[40..56].try_into().unwrap(),
        u64::from_le_bytes(token[56..64].try_into().unwrap()),
    )
    .unwrap();
    let declaration_digest: [u8; 32] = token[64..96].try_into().unwrap();
    let destination_session: [u8; 16] = token[24..40].try_into().unwrap();
    let publication = source_publication_frame.unwrap();
    // A claim's scope must agree with the source frame it carries, so a
    // foreign-scope claim carries a correspondingly rescoped source frame.
    let rescoped = |scope| {
        BlobGenerationPublicationV1::new(
            publication.store(),
            publication.session(),
            publication.object(),
            publication.generation(),
            publication.root_record(),
            publication.root_digest(),
            publication.total_bytes(),
            publication.logical_digest(),
            publication.chunk_size(),
            scope,
        )
        .unwrap()
    };
    let claim = |session, ordinal, scope| {
        let source = rescoped(scope);
        let source_sha256 = Sha256::digest(source.encode()).into();
        let base = BlobChunkReuseClaimV1::new(
            serving.store_identity().bytes(),
            session,
            ordinal,
            scope,
            CHUNK as u32,
            CHUNK as u32,
            digest.unwrap(),
            source_chunk.unwrap(),
            source_publication.unwrap(),
            0,
        )
        .unwrap();
        BlobChunkReuseClaimV2::new(base, source, source_sha256)
            .unwrap()
            .encode()
    };
    let submission = serving.certification_record_submission();
    let valid = claim(destination_session, 0, publication.key_scope());
    assert_eq!(
        submission.verify_blob_reuse_claim_before_append(
            valid.clone(),
            declaration_record,
            declaration_digest,
        ),
        Ok(())
    );
    for encoded in [
        claim([0x7f; 16], 0, publication.key_scope()),
        claim(destination_session, 1, publication.key_scope()),
        claim(destination_session, 0, [0x7e; 32]),
    ] {
        assert_eq!(
            submission.verify_blob_reuse_claim_before_append(
                encoded,
                declaration_record,
                declaration_digest,
            ),
            Err(RecordAppendDenial::ReuseDestinationInvalid)
        );
    }
    let mut wrong_digest = declaration_digest;
    wrong_digest[0] ^= 1;
    assert_eq!(
        submission.verify_blob_reuse_claim_before_append(
            valid.clone(),
            declaration_record,
            wrong_digest,
        ),
        Err(RecordAppendDenial::ReuseDestinationInvalid)
    );

    let before = selected_blob_records(&serving).len();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0x31; 32]))
        .unwrap();
    let prepared = submission.prepare_blob_reuse_claim_append(
        valid,
        declaration_record,
        wrong_digest,
        placement(),
        PhysicalMutationRequest::platform_durable(key, deadline),
    );
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        prepared.into_raw()
    else {
        panic!("classified claim must reach selected proof planning")
    };
    let PhysicalMutationOutcome::ProvenNoEffect(fate) = prepared.execute() else {
        panic!("wrong destination must not publish")
    };
    assert_eq!(
        fate.cause(),
        PhysicalMutationProvenNoEffectCause::AdmissionDeniedBeforeGroupSeal
    );
    assert_eq!(selected_blob_records(&serving).len(), before);
    drop(destination);
    drop(blobs);
    serving.close();
}

fn persisted(record: worth_store::physical_runtime::PhysicalRecordId) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new(record.allocation_epoch(), record.ordinal()).unwrap()
}
