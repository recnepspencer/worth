//! Frozen blob control-record frames shared with the offline observer.
//!
//! The observer holds byte-identical copies of these literals in
//! `worth-store-offline-integrity-observer/src/integrity_observation/blob_record/`
//! (`reuse_claim.rs` for kinds 11 and 15, `quarantine.rs` for kind 12) and
//! parses them with its independent reader. Change both copies together: a
//! field-order change in this encoder then fails here, not silently there.

mod support;

use support::independent_sha256;
use worth_store_physical_format::{
    BlobChunkReuseClaimV1, BlobChunkReuseClaimV2, BlobDedupeQuarantineV1,
    BlobGenerationPublicationV1, PersistedRecordIdentity,
};

/// Kind 11: store [1;16], session [2;16], ordinal 7, scope [3;32], 64 KiB
/// chunk and length, digest [4;32], chunk ([5;16], 6), source publication
/// ([7;16], 8), source ordinal 0.
const REUSE_CLAIM_V1_FRAME_HEX: &str = "5752433131424c420b010100a80000000a1601cf025dabe87098a24d1133bf460152a9c09f2d9296ade59ae6cf343df6010101010101010101010101010101010202020202020202020202020202020207000000000000000303030303030303030303030303030303030303030303030303030303030303000001000000010004040404040404040404040404040404040404040404040404040404040404040505050505050505050505050505050506000000000000000707070707070707070707070707070708000000000000000000000000000000";
/// Kind 12: store [2;16], scope [3;32], digest [4;32], 64 KiB chunk, source
/// publication ([1;16], 1) ordinal 0, source chunk ([1;16], 2), destination
/// session [5;16] ordinal 0, conflicting chunk ([1;16], 3).
const DEDUPE_QUARANTINE_V1_FRAME_HEX: &str = "5752433131424c420c010100bc000000edfb1a903ae2d0ebf5418e9eec893721608fd86e050b17cebda7f256fd0962770202020202020202020202020202020203030303030303030303030303030303030303030303030303030303030303030404040404040404040404040404040404040404040404040404040404040404000001000101010101010101010101010101010101000000000000000000000000000000010101010101010101010101010101010200000000000000050505050505050505050505050505050000000000000000010101010101010101010101010101010300000000000000";
/// Kind 15: the kind-11 claim for ordinal 0, chunk ([7;16], 6), source
/// publication ([7;16], 9), carrying the canonical kind-4 publication of one
/// 64 KiB generation (session [9;16], object [4;16], generation 1, root
/// ([7;16], 8), root digest [6;32], logical digest [5;32]) and its SHA-256.
const REUSE_CLAIM_V2_FRAME_HEX: &str = "5752433131424c420f010100b40100006a666da631f38b480e8e2701a6f56018fbbd2a0184b416aad35e9a1c74db6b360101010101010101010101010101010102020202020202020202020202020202000000000000000003030303030303030303030303030303030303030303030303030303030303030000010000000100040404040404040404040404040404040404040404040404040404040404040407070707070707070707070707070707060000000000000007070707070707070707070707070707090000000000000000000000000000001b2475c7d7b109d3d4e7afda0af5a55848b76e7a5367ac749fc019da4fd247835752433131424c4204010000bc000000008500f2dbd62317aa4d9b340e60645c805c8f5b6e245fb89d606ad78d549ade0101010101010101010101010101010109090909090909090909090909090909040404040404040404040404040404040100000000000000070707070707070707070707070707070800000000000000060606060606060606060606060606060606060606060606060606060606060600000100000000000505050505050505050505050505050505050505050505050505050505050505000001000303030303030303030303030303030303030303030303030303030303030303";

const CHUNK: u32 = 64 << 10;

fn record(epoch: u8, ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([epoch; 16], ordinal).expect("nonzero record")
}

fn claim(
    ordinal: u64,
    chunk: PersistedRecordIdentity,
    source: PersistedRecordIdentity,
) -> BlobChunkReuseClaimV1 {
    BlobChunkReuseClaimV1::new(
        [1; 16], [2; 16], ordinal, [3; 32], CHUNK, CHUNK, [4; 32], chunk, source, 0,
    )
    .expect("valid reuse claim")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn reuse_claim_v1_encoder_matches_the_shared_golden() {
    let encoded = claim(7, record(5, 6), record(7, 8)).encode();
    assert_eq!(hex(&encoded), REUSE_CLAIM_V1_FRAME_HEX);
}

#[test]
fn dedupe_quarantine_v1_encoder_matches_the_shared_golden() {
    let encoded = BlobDedupeQuarantineV1::new(
        [2; 16],
        [3; 32],
        [4; 32],
        CHUNK,
        record(1, 1),
        0,
        record(1, 2),
        [5; 16],
        0,
        record(1, 3),
    )
    .expect("valid dedupe quarantine")
    .encode();
    assert_eq!(hex(&encoded), DEDUPE_QUARANTINE_V1_FRAME_HEX);
}

#[test]
fn reuse_claim_v2_encoder_matches_the_shared_golden() {
    let publication = BlobGenerationPublicationV1::new(
        [1; 16],
        [9; 16],
        [4; 16],
        1,
        record(7, 8),
        [6; 32],
        u64::from(CHUNK),
        [5; 32],
        CHUNK,
        [3; 32],
    )
    .expect("valid publication");
    let digest = independent_sha256(&publication.encode());
    let encoded =
        BlobChunkReuseClaimV2::new(claim(0, record(7, 6), record(7, 9)), publication, digest)
            .expect("valid versioned reuse claim")
            .encode();
    assert_eq!(hex(&encoded), REUSE_CLAIM_V2_FRAME_HEX);
}
