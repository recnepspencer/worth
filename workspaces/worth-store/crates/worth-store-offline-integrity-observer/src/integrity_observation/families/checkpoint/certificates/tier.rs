//! Independent bounded interpretation of a schema-3 tier custody claim.

use crate::integrity_observation::sha256::{sha256, Sha256};

const DOMAIN: &[u8] = b"store.physical.checkpoint.tier-epoch-custody.v1";
const INTENT_DOMAIN: &[u8] = b"store.physical.tier-epoch-activation.v1";
const ANCHOR_DOMAIN: &[u8] = b"worth.store.tier-epoch-root-anchor.v1";
const INTENT_BYTES: usize = 8 + INTENT_DOMAIN.len() + 1 + 168;
const BYTES: usize = 8 + DOMAIN.len() + 24 + 8 + 32 + 32 + INTENT_BYTES + 160;

pub(super) fn valid(
    bytes: &[u8],
    source_store: [u8; 16],
    source_sequence: u64,
    source_generation: u64,
) -> bool {
    if bytes.len() != BYTES
        || bytes[..8] != (DOMAIN.len() as u64).to_le_bytes()
        || &bytes[8..8 + DOMAIN.len()] != DOMAIN
    {
        return false;
    }
    let mut p = 8 + DOMAIN.len();
    let checkpoint = &bytes[p..p + 24];
    p += 24;
    let root_generation = u64_at(bytes, p);
    p += 8;
    let root_sha = &bytes[p..p + 32];
    p += 32;
    let anchor = &bytes[p..p + 32];
    p += 32;
    let intent = &bytes[p..p + INTENT_BYTES];
    p += INTENT_BYTES;
    let first = &bytes[p..p + 80];
    p += 80;
    let second = &bytes[p..p + 80];
    if checkpoint[..16] != source_store
        || u64_at(checkpoint, 16) != source_sequence
        || root_generation != source_generation
        || root_sha == [0; 32]
        || intent[..8] != (INTENT_DOMAIN.len() as u64).to_le_bytes()
        || &intent[8..8 + INTENT_DOMAIN.len()] != INTENT_DOMAIN
    {
        return false;
    }
    let mut q = 8 + INTENT_DOMAIN.len();
    if intent[q] != 1 {
        return false;
    }
    let phase_offset = q;
    q += 1;
    let store = &intent[q..q + 16];
    q += 16;
    let attempt = &intent[q..q + 16];
    q += 16;
    let source_root_generation = u64_at(intent, q);
    q += 8;
    let source_sha = &intent[q..q + 32];
    q += 32;
    let source_free = &intent[q..q + 32];
    q += 32;
    let epoch = u64_at(intent, q);
    q += 8;
    let candidate_generation = u64_at(intent, q);
    q += 8;
    let candidate_sha = &intent[q..q + 32];
    q += 32;
    let retained = u64_at(intent, q);
    q += 8;
    let publication = u64_at(intent, q);
    if store != source_store
        || attempt == [0; 16]
        || source_root_generation == 0
        || source_sha == [0; 32]
        || source_free == [0; 32]
        || epoch == 0
        || source_root_generation.checked_add(1) != Some(candidate_generation)
        || candidate_sha == [0; 32]
        || retained == 0
        || publication == 0
        || root_generation < candidate_generation
        || (root_generation == candidate_generation && root_sha != candidate_sha)
        || !valid_wal(first)
        || !valid_wal(second)
        || u64_at(first, 8) > u64_at(second, 0)
        || first[48..80] != sha256(intent)
    {
        return false;
    }
    let mut completed = intent.to_vec();
    completed[phase_offset] = 2;
    if second[48..80] != sha256(&completed) {
        return false;
    }
    let mut digest = Sha256::new();
    digest.update(ANCHOR_DOMAIN);
    digest.update(store);
    digest.update(attempt);
    digest.update(&source_root_generation.to_le_bytes());
    digest.update(source_sha);
    digest.update(source_free);
    digest.update(&epoch.to_le_bytes());
    anchor == digest.finish()
}

fn valid_wal(bytes: &[u8]) -> bool {
    let start = u64_at(bytes, 0);
    start != 0 && start < u64_at(bytes, 8) && bytes[16..48] != [0; 32] && bytes[48..80] != [0; 32]
}

fn u64_at(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().expect("fixed bounded field"))
}
