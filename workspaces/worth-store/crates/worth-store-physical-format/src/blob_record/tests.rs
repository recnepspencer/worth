use sha2::{Digest, Sha256};

use super::*;
use crate::PersistedRecordIdentity;

fn identity() -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([7; 16], 11).expect("valid record")
}

fn declaration() -> BlobSessionDeclarationV1 {
    BlobSessionDeclarationV1::new(
        [1; 16],
        [2; 16],
        [3; 16],
        [4; 32],
        256 << 10,
        400_000,
        1 << 20,
        77,
    )
    .expect("valid declaration")
}

fn publication() -> BlobGenerationPublicationV1 {
    BlobGenerationPublicationV1::new(
        [1; 16],
        [2; 16],
        [3; 16],
        1,
        identity(),
        [5; 32],
        400_000,
        [6; 32],
        256 << 10,
        [4; 32],
    )
    .expect("valid publication")
}

#[test]
fn phase_two_blob_families_round_trip_as_typed_records() {
    let declaration = declaration();
    assert_eq!(
        decode_blob_record(&declaration.encode()),
        Ok(BlobRecordV1::SessionDeclared(declaration))
    );

    let occurrence = BlobChunkOccurrenceV1::new([1; 16], [2; 16], 0).expect("valid occurrence");
    let chunk = BlobChunkFrameV1::encode(occurrence, 256 << 10, &[9; 64 << 10]).expect("chunk");
    let decoded = DecodedBlobChunkFrameV1::decode(&chunk).expect("decode chunk");
    assert_eq!(decoded.occurrence(), occurrence);
    assert_eq!(decoded.bytes(), &[9; 64 << 10]);
    assert_eq!(
        decode_blob_record(&chunk).expect("typed chunk").kind(),
        BlobRecordKind::Chunk
    );

    let tree = BlobTreeNodeV1::new(
        BlobTreeOccurrenceV1::new([1; 16], [2; 16], BlobTreeNodeKind::Leaf, 0, 0)
            .expect("occurrence"),
        vec![BlobTreeEntryV1::new(decoded.stored_digest(), identity(), 64 << 10).expect("entry")],
    )
    .expect("node");
    assert_eq!(BlobTreeNodeV1::decode(&tree.encode()), Ok(tree.clone()));
    assert_eq!(
        decode_blob_record(&tree.encode()),
        Ok(BlobRecordV1::TreeNode(tree))
    );

    let publication = publication();
    assert_eq!(
        decode_blob_record(&publication.encode()),
        Ok(BlobRecordV1::GenerationPublished(publication))
    );
}

#[test]
fn session_frontier_is_a_fixed_bounded_control_frame() {
    let frontier = BlobSessionFrontierV1::new(
        [1; 16],
        [2; 16],
        identity(),
        [3; 32],
        2,
        400_000,
        PersistedRecordIdentity::new([8; 16], 19).unwrap(),
        [4; 32],
    )
    .expect("frontier");
    let bytes = frontier.encode();
    assert_eq!(bytes.len(), 208);
    assert_eq!(BlobSessionFrontierV1::decode(&bytes), Ok(frontier));
    assert_eq!(
        decode_blob_record(&bytes),
        Ok(BlobRecordV1::SessionFrontier(frontier))
    );
    assert_eq!(frontier.next_chunk_ordinal(), 2);
    assert_eq!(frontier.durable_bytes(), 400_000);
    assert_eq!(
        BlobSessionFrontierV1::new(
            [1; 16],
            [2; 16],
            identity(),
            [3; 32],
            0,
            400_000,
            PersistedRecordIdentity::new([8; 16], 19).unwrap(),
            [4; 32]
        ),
        Err(BlobRecordDenial::InvalidFrontier),
    );
}

#[test]
fn explicit_abandonment_is_a_fixed_declaration_bound_terminal() {
    let abandoned = BlobSessionAbandonedV1::new(
        [1; 16],
        [2; 16],
        identity(),
        [3; 32],
        BlobAbandonmentReasonV1::ExplicitAbort,
    )
    .unwrap();
    let bytes = abandoned.encode();
    assert_eq!(bytes.len(), 137);
    assert_eq!(bytes[8], BlobRecordKind::SessionAbandoned as u8);
    assert_eq!(&bytes[48..64], &[1; 16]);
    assert_eq!(&bytes[64..80], &[2; 16]);
    assert_eq!(&bytes[80..96], &[7; 16]);
    assert_eq!(&bytes[96..104], &11_u64.to_le_bytes());
    assert_eq!(&bytes[104..136], &[3; 32]);
    assert_eq!(bytes[136], 1);
    assert_eq!(BlobSessionAbandonedV1::decode(&bytes), Ok(abandoned));
    assert_eq!(
        decode_blob_record(&bytes),
        Ok(BlobRecordV1::SessionAbandoned(abandoned))
    );
    let mut payload = bytes[48..].to_vec();
    payload[88] = 2;
    assert_eq!(
        BlobSessionAbandonedV1::decode_payload(&payload),
        Err(BlobRecordDenial::LengthMismatch),
        "reason-specific expiry sequence is required"
    );
    assert_eq!(
        BlobSessionAbandonedV1::decode_payload(&payload[..88]),
        Err(BlobRecordDenial::LengthMismatch)
    );
}

#[test]
fn checkpoint_expiry_has_exact_nonzero_witness_and_preserves_explicit_bytes() {
    let expired = BlobSessionAbandonedV1::new(
        [1; 16],
        [2; 16],
        identity(),
        [3; 32],
        BlobAbandonmentReasonV1::CheckpointExpired {
            checkpoint_sequence: std::num::NonZeroU64::new(17).unwrap(),
        },
    )
    .unwrap();
    let bytes = expired.encode();
    assert_eq!(bytes.len(), 145);
    assert_eq!(bytes[136], 2);
    assert_eq!(&bytes[137..145], &17_u64.to_le_bytes());
    assert_eq!(BlobSessionAbandonedV1::decode(&bytes), Ok(expired));
    assert_eq!(
        decode_blob_record(&bytes),
        Ok(BlobRecordV1::SessionAbandoned(expired))
    );

    let mut payload = bytes[48..].to_vec();
    payload[89..97].fill(0);
    assert_eq!(
        BlobSessionAbandonedV1::decode_payload(&payload),
        Err(BlobRecordDenial::InvalidAbandonment)
    );
    payload[88] = 3;
    assert_eq!(
        BlobSessionAbandonedV1::decode_payload(&payload),
        Err(BlobRecordDenial::InvalidAbandonment)
    );
    payload[88] = 1;
    assert_eq!(
        BlobSessionAbandonedV1::decode_payload(&payload),
        Err(BlobRecordDenial::LengthMismatch)
    );
    payload[88] = 2;
    assert_eq!(
        BlobSessionAbandonedV1::decode_payload(&payload[..96]),
        Err(BlobRecordDenial::LengthMismatch)
    );
}

#[test]
fn content_digest_excludes_occurrence_but_includes_chunk_rule() {
    let first = BlobChunkFrameV1::encode(
        BlobChunkOccurrenceV1::new([1; 16], [2; 16], 0).expect("claim"),
        256 << 10,
        &[8; 64 << 10],
    )
    .expect("first");
    let second = BlobChunkFrameV1::encode(
        BlobChunkOccurrenceV1::new([1; 16], [3; 16], 17).expect("claim"),
        256 << 10,
        &[8; 64 << 10],
    )
    .expect("second");
    let other_rule = BlobChunkFrameV1::encode(
        BlobChunkOccurrenceV1::new([1; 16], [2; 16], 0).expect("claim"),
        128 << 10,
        &[8; 64 << 10],
    )
    .expect("other rule");
    assert_eq!(
        DecodedBlobChunkFrameV1::decode(&first)
            .expect("first")
            .stored_digest(),
        DecodedBlobChunkFrameV1::decode(&second)
            .expect("second")
            .stored_digest(),
    );
    assert_ne!(
        DecodedBlobChunkFrameV1::decode(&first)
            .expect("first")
            .stored_digest(),
        DecodedBlobChunkFrameV1::decode(&other_rule)
            .expect("other")
            .stored_digest(),
    );
    assert_ne!(first, second, "outer frame authenticates occurrence claim");
}

#[test]
fn recomputed_integrity_cannot_substitute_for_missing_occurrence_claim() {
    let mut chunk = BlobChunkFrameV1::encode(
        BlobChunkOccurrenceV1::new([1; 16], [2; 16], 0).expect("claim"),
        64 << 10,
        &[8; 64 << 10],
    )
    .expect("chunk");
    chunk[10..12].copy_from_slice(&0_u16.to_le_bytes());
    let mut hash = Sha256::new();
    hash.update(&chunk[..16]);
    hash.update(&chunk[48..]);
    chunk[16..48].copy_from_slice(&hash.finalize());
    assert_eq!(
        decode_blob_record(&chunk),
        Err(BlobRecordDenial::MissingOccurrenceClaim)
    );
}

#[test]
fn admitted_chunk_rule_is_distinct_from_frame_capacity() {
    let occurrence = BlobChunkOccurrenceV1::new([1; 16], [2; 16], 0).expect("claim");
    assert_eq!(
        BlobChunkFrameV1::encode(occurrence, 1 << 20, &[8; 64 << 10]),
        Err(BlobRecordDenial::InvalidChunkRule),
    );
    assert_eq!(
        BlobChunkFrameV1::encode(occurrence, 64 << 10, &[8; (64 << 10) + 1]),
        Err(BlobRecordDenial::InvalidChunkLength),
    );
}

#[test]
fn invalid_tree_shape_and_publication_identity_are_denied() {
    assert_eq!(
        BlobTreeOccurrenceV1::new([1; 16], [2; 16], BlobTreeNodeKind::Leaf, 1, 0),
        Err(BlobRecordDenial::InvalidTreeShape),
    );
    let mut published = publication().encode();
    published[48 + 56..48 + 80].fill(0);
    let mut hash = Sha256::new();
    hash.update(&published[..16]);
    hash.update(&published[48..]);
    published[16..48].copy_from_slice(&hash.finalize());
    assert_eq!(
        decode_blob_record(&published),
        Err(BlobRecordDenial::InvalidIdentity)
    );
}

#[test]
fn tree_root_uses_full_frame_sha_while_interior_edge_uses_canonical_digest() {
    let entry = BlobTreeEntryV1::new([8; 32], identity(), 64 << 10).expect("entry");
    let first = BlobTreeNodeV1::new(
        BlobTreeOccurrenceV1::new([1; 16], [2; 16], BlobTreeNodeKind::Leaf, 0, 0).expect("claim"),
        vec![entry],
    )
    .expect("first node");
    let second = BlobTreeNodeV1::new(
        BlobTreeOccurrenceV1::new([1; 16], [3; 16], BlobTreeNodeKind::Leaf, 0, 0).expect("claim"),
        vec![entry],
    )
    .expect("second node");
    assert_eq!(first.canonical_digest(), second.canonical_digest());
    assert_ne!(first.frame_digest(), second.frame_digest());
    let expected_full_sha: [u8; 32] = Sha256::digest(first.encode()).into();
    assert_eq!(first.frame_digest(), expected_full_sha);
    assert_ne!(first.frame_digest(), first.canonical_digest());
    let publication = BlobGenerationPublicationV1::new(
        [1; 16],
        [2; 16],
        [3; 16],
        1,
        identity(),
        first.frame_digest(),
        64 << 10,
        [6; 32],
        256 << 10,
        [4; 32],
    )
    .expect("publication");
    assert_eq!(
        BlobGenerationPublicationV1::decode(&publication.encode()),
        Ok(publication)
    );
}
