//! Hand-assembled schema-2 accumulators exercise the observer's own decoder.
//! The schema-1 literal is re-labeled and extended byte by byte; no Store
//! encoder is involved.

use super::tests::{literal, observed, stream, ACCUMULATOR, BATCH_A, BATCH_B};
use super::ReleaseClaimKind;
use crate::integrity_observation::OfflineIntegrityOutcome as Outcome;

const DOMAIN_SCHEMA_DIGIT: usize = 8 + 50 - 1;
const SCHEMA: usize = 8 + 50;
const PRIOR_SEQUENCE: usize = 8 + 50 + 2 + 24 + 8 + 32;
const SCHEMA_ONE_BYTES: usize = 623;
const HEAD_DIGEST: usize = SCHEMA_ONE_BYTES + 8;
const PRIOR_HEAD_COUNT: usize = HEAD_DIGEST + 32;
const PRIOR_HEAD_DIGEST: usize = PRIOR_HEAD_COUNT + 8;

fn relabeled(mut bytes: Vec<u8>) -> Vec<u8> {
    assert_eq!(bytes[DOMAIN_SCHEMA_DIGIT], b'1');
    bytes[DOMAIN_SCHEMA_DIGIT] = b'2';
    bytes[SCHEMA] = 2;
    bytes
}

/// A first released checkpoint: five selected heads, no prior roster.
fn first_release() -> Vec<u8> {
    let mut bytes = relabeled(literal(ACCUMULATOR));
    assert_eq!(bytes.len(), SCHEMA_ONE_BYTES);
    bytes.extend(5_u64.to_le_bytes());
    bytes.extend([0x0b; 32]);
    bytes.extend(0_u64.to_le_bytes());
    bytes.extend([0; 32]);
    bytes
}

/// The same checkpoint naming a prior released checkpoint with four heads.
fn successor_release() -> Vec<u8> {
    let mut bytes = first_release();
    let mut at = PRIOR_SEQUENCE;
    for part in [
        &2_u64.to_le_bytes()[..],
        &[0x0c; 32],
        &[0x0d; 32],
        &1_u64.to_le_bytes(),
        &[0x0e; 32],
    ] {
        bytes[at..at + part.len()].copy_from_slice(part);
        at += part.len();
    }
    bytes[PRIOR_HEAD_COUNT..PRIOR_HEAD_COUNT + 8].copy_from_slice(&4_u64.to_le_bytes());
    bytes[PRIOR_HEAD_DIGEST..PRIOR_HEAD_DIGEST + 32].fill(0x0f);
    bytes
}

fn roster(accumulator: &[u8]) -> Vec<u8> {
    stream(&[literal(BATCH_A), literal(BATCH_B)], accumulator)
}

fn assert_accumulator_malformed(accumulator: &[u8]) {
    let result = observed(&roster(accumulator));
    assert!(result.completed_release_claim.is_none());
    let rejected = result
        .records
        .iter()
        .find(|row| row.outcome != Outcome::Intact)
        .expect("a damaged record");
    assert_eq!(rejected.kind, 7, "the accumulator itself is the damage");
    assert!(matches!(&rejected.outcome, Outcome::Damaged(_)));
}

#[test]
fn head_roster_accumulators_complete_a_released_roster() {
    for accumulator in [first_release(), successor_release()] {
        let result = observed(&roster(&accumulator));
        assert!(result
            .records
            .iter()
            .all(|row| row.outcome == Outcome::Intact));
        let claim = result
            .completed_release_claim
            .expect("complete selected roster");
        assert_eq!(claim.source_root_sha(), [1; 32]);
        let ReleaseClaimKind::Released { batches, .. } = claim.kind else {
            panic!("expected released roster");
        };
        assert_eq!(batches.len(), 2);
    }
}

#[test]
fn head_roster_commitments_must_agree_with_the_prior_binding() {
    let mut no_heads = first_release();
    no_heads[HEAD_DIGEST..HEAD_DIGEST + 32].fill(0);
    assert_accumulator_malformed(&no_heads);

    let mut count_without_prior = first_release();
    count_without_prior[PRIOR_HEAD_COUNT] = 1;
    assert_accumulator_malformed(&count_without_prior);

    let mut digest_without_prior = first_release();
    digest_without_prior[PRIOR_HEAD_DIGEST] = 1;
    assert_accumulator_malformed(&digest_without_prior);

    let mut prior_without_digest = successor_release();
    prior_without_digest[PRIOR_HEAD_DIGEST..PRIOR_HEAD_DIGEST + 32].fill(0);
    assert_accumulator_malformed(&prior_without_digest);
}

#[test]
fn domain_schema_variant_and_length_never_mix() {
    let mut old_schema = first_release();
    old_schema[SCHEMA] = 1;
    assert_accumulator_malformed(&old_schema);

    let mut old_domain = first_release();
    old_domain[DOMAIN_SCHEMA_DIGIT] = b'1';
    assert_accumulator_malformed(&old_domain);

    let mut new_schema_under_old_domain = literal(ACCUMULATOR);
    new_schema_under_old_domain[SCHEMA] = 2;
    assert_accumulator_malformed(&new_schema_under_old_domain);

    // A schema-2 label on a schema-1 body, and a schema-1 label on a schema-2 body.
    assert_accumulator_malformed(&relabeled(literal(ACCUMULATOR)));
    let mut long_schema_one = literal(ACCUMULATOR);
    long_schema_one.extend(&first_release()[SCHEMA_ONE_BYTES..]);
    assert_accumulator_malformed(&long_schema_one);

    // Only the accumulator has a schema-2 form.
    let result = observed(&stream(
        &[relabeled(literal(BATCH_A)), literal(BATCH_B)],
        &literal(ACCUMULATOR),
    ));
    assert!(result.completed_release_claim.is_none());
    let first = &result.records.iter().find(|row| row.kind == 7).unwrap();
    assert!(matches!(&first.outcome, Outcome::Damaged(_)));
}
