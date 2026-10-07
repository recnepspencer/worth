use super::*;
use worth_store_physical_format::{
    BlobChunkReuseClaimV1, BlobDedupeQuarantineV1, BlobSessionFrontierV1,
};

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([1; 16], ordinal).unwrap()
}

#[test]
fn selected_reuse_claim_protects_its_original_chunk_edge() {
    let source = record(7);
    let claim = BlobChunkReuseClaimV1::new(
        [3; 16],
        [2; 16],
        0,
        [4; 32],
        64 << 10,
        64 << 10,
        [5; 32],
        source,
        record(8),
        0,
    )
    .unwrap()
    .encode();
    let mut candidates = vec![ResidueCandidate {
        record: source,
        chunk_ordinal: None,
        referenced: false,
    }];
    mark_incoming_edges(&claim, [2; 16], &mut candidates, &mut 0, &mut Vec::new()).unwrap();
    assert!(
        candidates[0].referenced,
        "shared original chunk is not residue"
    );
}

#[test]
fn selected_quarantine_protects_both_originals_and_source_publication() {
    let ids = [record(7), record(8), record(9)];
    let claim = BlobDedupeQuarantineV1::new(
        [3; 16],
        [4; 32],
        [5; 32],
        64 << 10,
        ids[0],
        0,
        ids[1],
        [2; 16],
        0,
        ids[2],
    )
    .unwrap()
    .encode();
    let mut candidates: Vec<_> = ids
        .into_iter()
        .map(|record| ResidueCandidate {
            record,
            chunk_ordinal: None,
            referenced: false,
        })
        .collect();
    mark_incoming_edges(&claim, [2; 16], &mut candidates, &mut 0, &mut Vec::new()).unwrap();
    assert!(candidates.iter().all(|candidate| candidate.referenced));
}

#[test]
fn surviving_frontier_protects_earlier_prefix_chunks_before_single_record_drop() {
    let session = [2; 16];
    // The first chunk sorts before the frontier; protecting only the explicit
    // last-chunk edge would choose this chunk for a one-record drop.
    let mut candidates = vec![
        ResidueCandidate {
            record: record(1),
            chunk_ordinal: Some(0),
            referenced: false,
        },
        ResidueCandidate {
            record: record(2),
            chunk_ordinal: Some(1),
            referenced: false,
        },
        ResidueCandidate {
            record: record(3),
            chunk_ordinal: None,
            referenced: false,
        },
        ResidueCandidate {
            record: record(4),
            chunk_ordinal: Some(2),
            referenced: false,
        },
    ];
    let frontier = BlobSessionFrontierV1::new(
        [3; 16],
        session,
        record(9),
        [4; 32],
        2,
        128 << 10,
        record(2),
        [5; 32],
    )
    .unwrap()
    .encode();
    let mut prefix = 0;
    mark_incoming_edges(
        &frontier,
        session,
        &mut candidates,
        &mut prefix,
        &mut Vec::new(),
    )
    .unwrap();
    protect_frontier_prefix(&mut candidates, prefix);
    let first = candidates
        .iter()
        .find(|candidate| !candidate.referenced)
        .unwrap();
    assert_eq!(first.record, record(3));
    assert!(candidates[0].referenced);
    assert!(candidates[1].referenced);
    assert!(
        !candidates[3].referenced,
        "unclaimed suffix is not protected by the prefix"
    );

    // With the frontier absent from the next selected root, its implicit
    // prefix no longer keeps either earlier chunk alive.
    for candidate in &mut candidates {
        candidate.referenced = false;
    }
    protect_frontier_prefix(&mut candidates, 0);
    assert!(!candidates[0].referenced);
    assert!(!candidates[1].referenced);
}

#[test]
fn another_sessions_frontier_does_not_claim_this_sessions_prefix() {
    let mut candidates = vec![ResidueCandidate {
        record: record(1),
        chunk_ordinal: Some(0),
        referenced: false,
    }];
    let frontier = BlobSessionFrontierV1::new(
        [3; 16],
        [8; 16],
        record(9),
        [4; 32],
        2,
        128 << 10,
        record(7),
        [5; 32],
    )
    .unwrap()
    .encode();
    let mut prefix = 0;
    mark_incoming_edges(
        &frontier,
        [2; 16],
        &mut candidates,
        &mut prefix,
        &mut Vec::new(),
    )
    .unwrap();
    protect_frontier_prefix(&mut candidates, prefix);
    assert_eq!(prefix, 0);
    assert!(!candidates[0].referenced);
}
