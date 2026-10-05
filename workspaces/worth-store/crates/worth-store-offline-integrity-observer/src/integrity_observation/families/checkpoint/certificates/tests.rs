//! Frozen, independently assembled tag-7 payloads. No Store encoder or digest helper.
use super::super::stream::read_checkpoint;
use crate::integrity_observation::{
    crc32c::crc32c, sha256::Sha256, OfflineIntegrityObservationCounters,
    OfflineIntegrityOutcome as Outcome, OfflinePhysicalDamageCause as Cause,
};

// Batch A is terminal for its generation; B belongs to another generation.
pub(super) const BATCH_A: &str = concat!(
    "320000000000000073746f72652e706879736963616c2e636865636b706f696e742e72656c65617365642d64726f702d",
    "637573746f64792e763101010707070707070707070707070707070703000000000000000f0000000000000001010101",
    "010101010101010101010101010101010101010101010101010101010000090909090909090909090909090909090400",
    "000000000000020202020202020202020202020202020202020202020202020202020202020203030303030303030303",
    "030303030303030303030303030303030303030303030909090909090909090909090909090905000000000000000404",
    "040404040404040404040404040404040404040404040404040404040404050505050505050505050505050505050505",
    "050505050505050505050505050506060606060606060606060606060606060606060606060606060606060606060a00",
    "000000000000140000000000000064000000000000006e00000000000000070707070707070707070707070707070707",
    "070707070707070707070707070708080808080808080808080808080808080808080808080808080808080808080c00",
    "000000000000090909090909090909090909090909090909090909090909090909090909090900000000000000000000",
    "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200",
    "0000000000000a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a01",
);
pub(super) const BATCH_B: &str = concat!(
    "320000000000000073746f72652e706879736963616c2e636865636b706f696e742e72656c65617365642d64726f702d",
    "637573746f64792e763101010707070707070707070707070707070703000000000000000f0000000000000001010101",
    "010101010101010101010101010101010101010101010101010101010100090909090909090909090909090909090600",
    "000000000000020202020202020202020202020202020202020202020202020202020202020203030303030303030303",
    "030303030303030303030303030303030303030303030909090909090909090909090909090907000000000000000404",
    "040404040404040404040404040404040404040404040404040404040404050505050505050505050505050505050505",
    "050505050505050505050505050506060606060606060606060606060606060606060606060606060606060606060a00",
    "000000000000140000000000000078000000000000008200000000000000070707070707070707070707070707070707",
    "070707070707070707070707070708080808080808080808080808080808080808080808080808080808080808080c00",
    "000000000000090909090909090909090909090909090909090909090909090909090909090900000000000000000000",
    "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300",
    "0000000000000a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a00",
);
pub(super) const ACCUMULATOR: &str = concat!(
    "320000000000000073746f72652e706879736963616c2e636865636b706f696e742e72656c65617365642d64726f702d",
    "637573746f64792e763101020707070707070707070707070707070703000000000000000f0000000000000001010101",
    "010101010101010101010101010101010101010101010101010101010000000000000000000000000000000000000000",
    "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200de2f",
    "23cf054010b335cc988ad3e56e26a875694dbbc08e5917c15fa286cb1bbe090909090909090909090909090909090600",
    "000000000000020202020202020202020202020202020202020202020202020202020202020209090909090909090909",
    "090909090909070000000000000004040404040404040404040404040404040404040404040404040404040404040505",
    "050505050505050505050505050505050505050505050505050505050505060606060606060606060606060606060606",
    "06060606060606060606060606060a000000000000001400000000000000780000000000000082000000000000000707",
    "070707070707070707070707070707070707070707070707070707070707080808080808080808080808080808080808",
    "08080808080808080808080808080c000000000000000909090909090909090909090909090909090909090909090909",
    "09090909090903000000000000000a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a00",
);

// Independent tag-6 intent/anchor/WAL payload witness vector.
pub(super) const TIER: &str = concat!(
    "2f0000000000000073746f72652e706879736963616c2e636865636b706f696e742e746965722d65706f63682d637573",
    "746f64792e76310707070707070707070707070707070703000000000000000f00000000000000010101010101010101",
    "010101010101010101010101010101010101010101010126c04a0c1ebbdcd29343a172053545846282fa4e5f10a0ba91",
    "28649a9022fda8270000000000000073746f72652e706879736963616c2e746965722d65706f63682d61637469766174",
    "696f6e2e76310107070707070707070707070707070707020202020202020202020202020202020e000000000000000b",
    "0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c",
    "0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c05000000000000000f000000000000000101010101010101010101010101010101",
    "0101010101010101010101010101010010000000000000060000000000000014000000000000001e000000000000000d",
    "0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0dd8645643560c4a5e8dde814d9629f73b0f",
    "dfde3d298fd7f0beef350b287355f91e0000000000000028000000000000000e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e",
    "0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e7ff2d018c26b5da0c54701bd61c6ca60f3cb2dedc11e5871427552c8ad2bdc71",
);

pub(super) fn literal(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0);
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
fn frame(kind: u8, payload: &[u8]) -> Vec<u8> {
    let mut bytes = b"WCP7REC\0".to_vec();
    bytes.extend([3, kind, 0, 0]);
    bytes.extend((payload.len() as u32).to_le_bytes());
    bytes.extend(payload);
    let crc = crc32c(&[&bytes]);
    bytes.extend(crc.to_le_bytes());
    bytes
}
pub(super) fn stream(batches: &[Vec<u8>], accumulator: &[u8]) -> Vec<u8> {
    let mut source = [0; 144];
    source[..16].fill(7);
    source[16..24].copy_from_slice(&3_u64.to_le_bytes());
    source[24..32].copy_from_slice(&1_u64.to_le_bytes());
    source[32..40].copy_from_slice(&10_u64.to_le_bytes());
    source[40..48].copy_from_slice(&15_u64.to_le_bytes());
    source[48..56].copy_from_slice(&1_u64.to_le_bytes());
    source[56..64].copy_from_slice(&15_u64.to_le_bytes());
    source[64] = 1;
    let header = frame(1, &source);
    let header_len = header.len() as u64;
    let mut compaction = [0; 16];
    compaction[..8].copy_from_slice(&15_u64.to_le_bytes());
    compaction[8..].copy_from_slice(&10_u64.to_le_bytes());
    let compaction = frame(3, &compaction);
    let mut bytes = Vec::new();
    bytes.extend(header);
    bytes.extend(&compaction);
    let mut digest = Sha256::new();
    let mut count = 0_u64;
    let mut length = 0_u64;
    for payload in batches
        .iter()
        .map(Vec::as_slice)
        .chain(std::iter::once(accumulator))
    {
        let record = frame(7, payload);
        digest.update(&record);
        count += 1;
        length += record.len() as u64;
        bytes.extend(record);
    }
    let mut footer = [0; 184];
    footer[..24].copy_from_slice(&source[..24]);
    footer[32..64].copy_from_slice(&Sha256::new().finish());
    footer[64..72].copy_from_slice(&header_len.to_le_bytes());
    footer[72..80].copy_from_slice(&15_u64.to_le_bytes());
    footer[80..88].copy_from_slice(&10_u64.to_le_bytes());
    footer[104..136].copy_from_slice(&Sha256::new().finish());
    footer[136..144].copy_from_slice(&count.to_le_bytes());
    footer[144..152].copy_from_slice(&length.to_le_bytes());
    footer[152..184].copy_from_slice(&digest.finish());
    bytes.extend(frame(5, &footer));
    bytes
}
pub(super) fn with_tier(mut bytes: Vec<u8>, tier: &[u8]) -> Vec<u8> {
    let cert_start = 164 + 36;
    let tier_frame = frame(6, tier);
    bytes.splice(cert_start..cert_start, tier_frame.iter().copied());
    let footer = bytes.len() - 204;
    let mut sha = Sha256::new();
    sha.update(&bytes[cert_start..footer]);
    let previous_count = u64::from_le_bytes(
        bytes[footer + 16 + 136..footer + 16 + 144]
            .try_into()
            .unwrap(),
    );
    let previous_bytes = u64::from_le_bytes(
        bytes[footer + 16 + 144..footer + 16 + 152]
            .try_into()
            .unwrap(),
    );
    bytes[footer + 16 + 136..footer + 16 + 144]
        .copy_from_slice(&(previous_count + 1).to_le_bytes());
    bytes[footer + 16 + 144..footer + 16 + 152]
        .copy_from_slice(&(previous_bytes + tier_frame.len() as u64).to_le_bytes());
    bytes[footer + 16 + 152..footer + 16 + 184].copy_from_slice(&sha.finish());
    let crc = crc32c(&[&bytes[footer..bytes.len() - 4]]);
    let end = bytes.len();
    bytes[end - 4..].copy_from_slice(&crc.to_le_bytes());
    bytes
}
pub(super) fn observed(bytes: &[u8]) -> super::super::stream::CheckpointStreamObservation {
    read_checkpoint(
        bytes,
        [7; 16],
        Some(3),
        64,
        &mut OfflineIntegrityObservationCounters::default(),
    )
}
pub(super) fn assert_rejected(bytes: &[u8]) {
    let result = observed(bytes);
    assert!(result.completed_release_claim.is_none());
    assert!(result
        .records
        .iter()
        .any(|row| row.outcome != Outcome::Intact));
}

// Independent wire offsets include the version and variant bytes after the domain.
const RELEASE_PREFIX: usize = 8 + 50 + 1 + 1;
const BATCH_ORDINAL: usize = RELEASE_PREFIX + 24 + 8 + 32;
const BATCH_DESCRIPTOR: usize = BATCH_ORDINAL + 2;
const ACCUMULATOR_BATCH_COUNT: usize = RELEASE_PREFIX + 24 + 8 + 32 + 8 + 32 + 32 + 8 + 32;
const ACCUMULATOR_BATCH_DIGEST: usize = ACCUMULATOR_BATCH_COUNT + 2;
const ACCUMULATOR_TIP_DESCRIPTOR: usize = ACCUMULATOR_BATCH_DIGEST + 32;

fn assert_roster_rejected(bytes: &[u8], cause: Cause) {
    let result = observed(bytes);
    assert!(result.completed_release_claim.is_none());
    let (footer, preceding) = result.records.split_last().expect("observed records");
    assert!(preceding.iter().all(|row| row.outcome == Outcome::Intact));
    assert_eq!(footer.kind, 5, "must reach the roster footer check");
    assert_eq!(footer.offset + footer.length, bytes.len() as u64);
    let Outcome::Damaged(damage) = &footer.outcome else {
        panic!("expected roster damage, got {:?}", footer.outcome);
    };
    assert_eq!(damage.cause(), cause);
}

fn reseal_batch_digest(accumulator: &mut [u8], batches: &[Vec<u8>]) {
    let domain = b"store.physical.checkpoint.released-drop-batches.v1";
    let mut sha = Sha256::new();
    sha.update(&(domain.len() as u64).to_le_bytes());
    sha.update(domain);
    sha.update(&(batches.len() as u16).to_le_bytes());
    for batch in batches {
        sha.update(&(batch.len() as u32).to_le_bytes());
        sha.update(batch);
    }
    let at = ACCUMULATOR_BATCH_DIGEST;
    accumulator[at..at + 32].copy_from_slice(&sha.finish());
}
#[test]
fn terminal_generation_followed_by_other_generation_is_legal_roster() {
    let bytes = stream(&[literal(BATCH_A), literal(BATCH_B)], &literal(ACCUMULATOR));
    let result = observed(&bytes);
    assert!(result
        .records
        .iter()
        .all(|row| row.outcome == Outcome::Intact));
    let roster = result
        .completed_release_claim
        .expect("complete selected roster");
    let super::ReleaseClaimKind::Released {
        batches,
        accumulator,
    } = roster.kind
    else {
        panic!("expected released roster");
    };
    assert_eq!(batches.len(), 2);
    assert!(batches[0].terminal);
    assert!(!batches[1].terminal);
    assert_eq!(&accumulator.tip_descriptor()[16..24], &6_u64.to_le_bytes());
}
#[test]
fn literal_tier_claim_is_independently_checked_and_resealed_damage_denied() {
    let base = stream(&[literal(BATCH_A), literal(BATCH_B)], &literal(ACCUMULATOR));
    let valid = with_tier(base.clone(), &literal(TIER));
    assert!(observed(&valid)
        .records
        .iter()
        .all(|row| row.outcome == Outcome::Intact));
    let mut bad_anchor = literal(TIER);
    let anchor = 8 + 47 + 24 + 8 + 32;
    bad_anchor[anchor] ^= 1;
    assert_rejected(&with_tier(base.clone(), &bad_anchor));
    let mut bad_wal_digest = literal(TIER);
    let wal_payload = 8 + 47 + 24 + 8 + 32 + 32 + (8 + 39 + 1 + 168) + 48;
    bad_wal_digest[wal_payload] ^= 1;
    assert_rejected(&with_tier(base, &bad_wal_digest));
}
#[test]
fn resealed_terminal_digest_duplicate_and_ordinal_forgeries_are_denied() {
    let a = literal(BATCH_A);
    let b = literal(BATCH_B);
    let acc = literal(ACCUMULATOR);
    assert_eq!(&b[BATCH_ORDINAL..BATCH_ORDINAL + 2], &1_u16.to_le_bytes());
    assert_eq!(
        &acc[ACCUMULATOR_BATCH_COUNT..ACCUMULATOR_BATCH_COUNT + 2],
        &2_u16.to_le_bytes()
    );
    let mut terminal = acc.clone();
    *terminal.last_mut().unwrap() = 1;
    assert_roster_rejected(
        &stream(&[a.clone(), b.clone()], &terminal),
        Cause::ScopeMismatch,
    );
    let mut digest = acc.clone();
    digest[ACCUMULATOR_BATCH_DIGEST] ^= 1;
    assert_roster_rejected(
        &stream(&[a.clone(), b.clone()], &digest),
        Cause::ScopeMismatch,
    );
    let mut duplicate = b.clone();
    let first_descriptor = &a[BATCH_DESCRIPTOR..BATCH_DESCRIPTOR + 24];
    duplicate[BATCH_DESCRIPTOR..BATCH_DESCRIPTOR + 24].copy_from_slice(first_descriptor);
    let mut resealed = acc.clone();
    // Keep the final tip coherent so only the repeated roster identity is invalid.
    resealed[ACCUMULATOR_TIP_DESCRIPTOR..ACCUMULATOR_TIP_DESCRIPTOR + 24]
        .copy_from_slice(first_descriptor);
    let duplicate_batches = [a.clone(), duplicate];
    reseal_batch_digest(&mut resealed, &duplicate_batches);
    assert_roster_rejected(
        &stream(&duplicate_batches, &resealed),
        Cause::DuplicateIdentity,
    );
    let mut ordinal = b;
    ordinal[BATCH_ORDINAL..BATCH_ORDINAL + 2].copy_from_slice(&0_u16.to_le_bytes());
    let ordinal_batches = [a, ordinal];
    let mut resealed = acc;
    reseal_batch_digest(&mut resealed, &ordinal_batches);
    assert_roster_rejected(
        &stream(&ordinal_batches, &resealed),
        Cause::DuplicateIdentity,
    );
}
#[test]
fn resealed_footer_aggregate_forgery_is_denied() {
    let mut bytes = stream(&[literal(BATCH_A), literal(BATCH_B)], &literal(ACCUMULATOR));
    let footer = bytes.len() - 204;
    bytes[footer + 16 + 152] ^= 1;
    let checksum = crc32c(&[&bytes[footer..bytes.len() - 4]]);
    let end = bytes.len();
    bytes[end - 4..].copy_from_slice(&checksum.to_le_bytes());
    assert_rejected(&bytes);
}
