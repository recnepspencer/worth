use worth_store_physical_format::{BlobSessionDeclarationV1, PersistedRecordIdentity};

use super::{validate_selected_claims, SelectedResumeClaim};
use crate::physical_runtime::blob::ingest::resume::BlobResumeFailure;

const CHUNK_BYTES: u64 = 65_536;

fn declaration() -> BlobSessionDeclarationV1 {
    BlobSessionDeclarationV1::new(
        [1; 16],
        [2; 16],
        [3; 16],
        [4; 32],
        CHUNK_BYTES as u32,
        CHUNK_BYTES + 17,
        4 << 20,
        9,
    )
    .expect("valid format declaration")
}

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([5; 16], ordinal).expect("nonzero record identity")
}

fn chunk(ordinal: u64, record_ordinal: u64, bytes: u64) -> SelectedResumeClaim {
    SelectedResumeClaim::Chunk {
        ordinal,
        record: record(record_ordinal),
        digest: [record_ordinal as u8; 32],
        bytes,
    }
}

fn reused(ordinal: u64, record_ordinal: u64, bytes: u64) -> SelectedResumeClaim {
    SelectedResumeClaim::ReusedChunk {
        ordinal,
        record: record(record_ordinal),
        digest: [record_ordinal as u8; 32],
        bytes,
    }
}

fn frontier(
    ordinal: u64,
    durable_bytes: u64,
    last_record: PersistedRecordIdentity,
    last_digest: [u8; 32],
) -> SelectedResumeClaim {
    SelectedResumeClaim::Frontier {
        ordinal,
        durable_bytes,
        last_record,
        last_digest,
    }
}

fn denied(claims: &mut [SelectedResumeClaim]) {
    assert!(matches!(
        validate_selected_claims(claims, declaration()),
        Err(BlobResumeFailure::ConflictingClaims)
    ));
}

#[test]
fn shuffled_selected_claims_sort_to_canonical_occurrence_order() {
    let node = SelectedResumeClaim::Node {
        ordinal: 7,
        record: record(3),
        frame_digest: [7; 32],
    };
    let first = chunk(0, 1, CHUNK_BYTES);
    let short_last = chunk(1, 2, 17);
    let completed = frontier(2, CHUNK_BYTES + 17, record(2), [2; 32]);
    let mut claims = [node, completed, short_last, first];

    validate_selected_claims(&mut claims, declaration()).expect("valid selected prefix");

    assert_eq!(claims, [first, short_last, node, completed]);
    assert_eq!(
        claims.map(SelectedResumeClaim::key),
        [(0, 0), (0, 1), (1, 7), (2, 2)]
    );
}

#[test]
fn missing_duplicate_and_wrong_short_chunk_cannot_be_reused() {
    denied(&mut [chunk(1, 2, 17)]);
    denied(&mut [chunk(0, 1, CHUNK_BYTES), chunk(0, 8, CHUNK_BYTES)]);
    denied(&mut [chunk(0, 1, CHUNK_BYTES), chunk(1, 2, 18)]);
}

#[test]
fn frontier_requires_exact_selected_prefix_and_last_chunk_binding() {
    let first = chunk(0, 1, CHUNK_BYTES);
    let short_last = chunk(1, 2, 17);
    denied(&mut [
        first,
        short_last,
        frontier(2, CHUNK_BYTES + 17, record(8), [2; 32]),
    ]);
    denied(&mut [
        first,
        short_last,
        frontier(2, CHUNK_BYTES + 17, record(2), [8; 32]),
    ]);
    denied(&mut [
        first,
        short_last,
        frontier(3, CHUNK_BYTES + 17, record(2), [2; 32]),
    ]);
}

#[test]
fn native_and_reused_chunks_share_one_destination_ordinal_namespace() {
    denied(&mut [chunk(0, 1, CHUNK_BYTES), reused(0, 2, CHUNK_BYTES)]);
    let first = reused(0, 3, CHUNK_BYTES);
    let last = chunk(1, 4, 17);
    let frontier = frontier(1, CHUNK_BYTES, record(3), [3; 32]);
    let mut claims = [last, frontier, first];
    validate_selected_claims(&mut claims, declaration()).expect("valid mixed prefix");
    assert_eq!(claims, [first, last, frontier]);
}
