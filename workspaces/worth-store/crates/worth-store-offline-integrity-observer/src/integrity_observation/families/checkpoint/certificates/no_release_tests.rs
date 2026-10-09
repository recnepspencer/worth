//! Hand-assembled tag-7 kind-3 vectors exercise the observer's own decoder.

use super::tests::{literal, observed, stream, with_tier, ACCUMULATOR, BATCH_A, TIER};
use super::ReleaseClaimKind;
use crate::integrity_observation::{
    crc32c::crc32c, sha256::Sha256, OfflineIntegrityOutcome as Outcome,
};

const GENESIS: &str = concat!(
    "320000000000000073746f72652e706879736963616c2e636865636b706f696e742e72656c65617365642d64726f702d",
    "637573746f64792e76310103",
    "070707070707070707070707070707070300000000000000",
    "0f00000000000000",
    "0101010101010101010101010101010101010101010101010101010101010101",
    "0000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
);

const PREFIX: usize = 8 + 50 + 2;
const ROOT_GENERATION: usize = PREFIX + 24;
const ROOT_SHA: usize = ROOT_GENERATION + 8;
const PRIOR_SEQUENCE: usize = ROOT_SHA + 32;
const PRIOR_ROOT_SHA: usize = PRIOR_SEQUENCE + 8;
const PRIOR_MARKER_SHA: usize = PRIOR_ROOT_SHA + 32;

fn marker() -> Vec<u8> {
    let bytes = literal(GENESIS);
    assert_eq!(bytes.len(), PREFIX + 24 + 8 + 32 + 8 + 32 + 32);
    bytes
}

fn assert_tag_seven_rejected(bytes: &[u8]) {
    let result = observed(bytes);
    assert!(result.completed_release_claim.is_none());
    let (rejected, preceding) = result.records.split_last().expect("checkpoint records");
    assert_eq!(rejected.kind, 7, "must reach the tag-7 decoder or roster");
    assert!(preceding.iter().all(|row| row.outcome == Outcome::Intact));
    assert!(matches!(&rejected.outcome, Outcome::Damaged(_)));
}

#[test]
fn literal_genesis_and_successor_are_positive_format_claims() {
    let genesis = marker();
    for bytes in [
        stream(&[], &genesis),
        with_tier(stream(&[], &genesis), &literal(TIER)),
    ] {
        let result = observed(&bytes);
        assert!(result
            .records
            .iter()
            .all(|row| row.outcome == Outcome::Intact));
        let claim = result
            .completed_release_claim
            .expect("positive tag-7 claim");
        assert_eq!(claim.source_root_sha(), [1; 32]);
        let ReleaseClaimKind::NoRelease(marker) = claim.kind else {
            panic!("expected no-release marker");
        };
        assert_eq!(marker.prior_sequence, 0);
        assert_eq!(marker.prior_root_sha, [0; 32]);
        assert_eq!(marker.prior_marker_payload_sha, [0; 32]);
    }
    let mut successor = genesis;
    successor[PRIOR_SEQUENCE..PRIOR_SEQUENCE + 8].copy_from_slice(&2_u64.to_le_bytes());
    successor[PRIOR_ROOT_SHA..PRIOR_ROOT_SHA + 32].fill(2);
    successor[PRIOR_MARKER_SHA..PRIOR_MARKER_SHA + 32].fill(3);
    let claim = observed(&stream(&[], &successor))
        .completed_release_claim
        .expect("positive successor");
    let ReleaseClaimKind::NoRelease(marker) = claim.kind else {
        panic!("expected successor marker");
    };
    assert_eq!(marker.prior_sequence, 2);
    assert_eq!(marker.prior_root_sha, [2; 32]);
    assert_eq!(marker.prior_marker_payload_sha, [3; 32]);
}

#[test]
fn resealed_marker_mutations_and_mixed_rosters_are_denied() {
    let genesis = marker();
    for offset in [PREFIX - 2, PREFIX - 1] {
        let mut malformed = genesis.clone();
        malformed[offset] = 0;
        assert_tag_seven_rejected(&stream(&[], &malformed));
    }
    let mut zero_root = genesis.clone();
    zero_root[ROOT_SHA..ROOT_SHA + 32].fill(0);
    assert_tag_seven_rejected(&stream(&[], &zero_root));
    let mut wrong_checkpoint = genesis.clone();
    wrong_checkpoint[PREFIX] = 8;
    assert_tag_seven_rejected(&stream(&[], &wrong_checkpoint));
    let mut wrong_generation = genesis.clone();
    wrong_generation[ROOT_GENERATION..ROOT_GENERATION + 8].copy_from_slice(&16_u64.to_le_bytes());
    assert_tag_seven_rejected(&stream(&[], &wrong_generation));
    let mut incomplete_prior = genesis.clone();
    incomplete_prior[PRIOR_SEQUENCE..PRIOR_SEQUENCE + 8].copy_from_slice(&1_u64.to_le_bytes());
    assert_tag_seven_rejected(&stream(&[], &incomplete_prior));
    let mut non_ancestor = genesis.clone();
    non_ancestor[PRIOR_SEQUENCE..PRIOR_SEQUENCE + 8].copy_from_slice(&3_u64.to_le_bytes());
    non_ancestor[PRIOR_ROOT_SHA..PRIOR_ROOT_SHA + 32].fill(2);
    non_ancestor[PRIOR_MARKER_SHA..PRIOR_MARKER_SHA + 32].fill(3);
    assert_tag_seven_rejected(&stream(&[], &non_ancestor));
    assert_tag_seven_rejected(&stream(&[genesis.clone()], &genesis));
    assert_tag_seven_rejected(&stream(&[literal(BATCH_A)], &marker()));
    assert_tag_seven_rejected(&stream(&[marker()], &literal(ACCUMULATOR)));
}

#[test]
fn empty_tag_seven_stream_does_not_infer_no_release() {
    let mut bytes = stream(&[], &marker());
    let footer_start = bytes.len() - 204;
    bytes.drain(200..footer_start);
    let footer_start = bytes.len() - 204;
    let footer_payload = footer_start + 16;
    bytes[footer_payload + 136..footer_payload + 152].fill(0);
    bytes[footer_payload + 152..footer_payload + 184].copy_from_slice(&Sha256::new().finish());
    let end = bytes.len();
    let crc = crc32c(&[&bytes[footer_start..end - 4]]);
    bytes[end - 4..].copy_from_slice(&crc.to_le_bytes());
    let result = observed(&bytes);
    assert!(result
        .records
        .iter()
        .all(|row| row.outcome == Outcome::Intact));
    assert!(result.completed_release_claim.is_none());
}
