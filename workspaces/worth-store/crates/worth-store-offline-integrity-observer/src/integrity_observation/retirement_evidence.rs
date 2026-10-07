//! Independent retirement facts, accepted only at verified WAL payload boundaries.
//! These are observations, never runtime recovery or mutation capabilities.

mod historical;
mod rewrite;

use super::copy_evidence::{CopyFinalFact, CopyIntentFact, CopyResolutionFact};
use super::families::durable_frame::read_u64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RetirementFact {
    pub(crate) completion: bool,
    pub(crate) arena_only: bool,
    pub(crate) source_root: u64,
    pub(crate) id: u64,
    pub(crate) generation: u64,
    pub(crate) bytes: u64,
    pub(crate) range: Option<(u64, u64, u64)>,
    pub(crate) releasing_root: u64,
    pub(crate) candidate: u64,
    pub(crate) digest: [u8; 32],
    metadata_bytes: u64,
    publication: u64,
}

impl RetirementFact {
    pub(crate) fn decode(payload: &[u8]) -> Option<Self> {
        const DOMAIN: &[u8] = b"store.physical.retirement.v2";
        if payload.len() != 8 + DOMAIN.len() + 121
            || read_u64(payload, 0) != DOMAIN.len() as u64
            || &payload[8..8 + DOMAIN.len()] != DOMAIN
        {
            return None;
        }
        let action = payload[8 + DOMAIN.len()];
        if !matches!(action, 3..=6) {
            return None;
        }
        let body = &payload[9 + DOMAIN.len()..];
        let arena_only = action >= 5;
        let source_root = read_u64(body, 0);
        let id = read_u64(body, 8);
        let generation = read_u64(body, 16);
        let bytes = read_u64(body, 24);
        let arena = read_u64(body, 32);
        let offset = read_u64(body, 40);
        let length = read_u64(body, 48);
        let releasing_root = read_u64(body, 56);
        let candidate = read_u64(body, 64);
        let digest: [u8; 32] = body[72..104].try_into().ok()?;
        let metadata_bytes = read_u64(body, 104);
        let publication = read_u64(body, 112);
        if source_root == 0
            || id == 0
            || generation == 0
            || metadata_bytes == 0
            || publication == 0
            || digest == [0; 32]
            || releasing_root.checked_add(1) != Some(candidate)
        {
            return None;
        }
        let range = if arena_only {
            if (arena, offset, length) != (0, 0, 0)
                || generation != source_root
                || releasing_root < source_root
            {
                return None;
            }
            None
        } else {
            if arena == 0
                || length == 0
                || length != bytes
                || offset.checked_add(length).is_none()
                || releasing_root <= source_root
            {
                return None;
            }
            Some((arena, offset, length))
        };
        Some(Self {
            completion: action % 2 == 0,
            arena_only,
            source_root,
            id,
            generation,
            bytes,
            range,
            releasing_root,
            candidate,
            digest,
            metadata_bytes,
            publication,
        })
    }
    fn identity(&self) -> (bool, u64, u64, Option<(u64, u64, u64)>) {
        (self.arena_only, self.id, self.generation, self.range)
    }
}

#[derive(Default)]
pub(crate) struct RetirementEvidence {
    frames: Vec<(u64, RetirementFact)>,
    rewrites: Vec<rewrite::RewriteFact>,
    copy_intents: Vec<CopyIntentFact>,
    copy_resolutions: Vec<CopyResolutionFact>,
    copy_finals: Vec<CopyFinalFact>,
}

impl RetirementEvidence {
    pub(crate) fn observe(&mut self, lsn: (u64, u64), store: Option<[u8; 16]>, payload: &[u8]) {
        if let Some(record) = RetirementFact::decode(payload) {
            self.frames.push((lsn.0, record));
        }
        if let Some(rewrite) =
            store.and_then(|store| rewrite::RewriteFact::decode(payload, store, lsn))
        {
            self.rewrites.push(rewrite);
        }
        if let Some(intent) = CopyIntentFact::decode(payload, lsn) {
            self.copy_intents.push(intent);
        }
        if let Some(resolution) = CopyResolutionFact::decode(payload, lsn) {
            self.copy_resolutions.push(resolution);
        }
        if let Some(final_copy) = store.and_then(|store| CopyFinalFact::decode(payload, store, lsn))
        {
            self.copy_finals.push(final_copy);
        }
    }
    fn unresolved(mut self) -> Vec<RetirementFact> {
        self.frames.sort_by_key(|(lsn, _)| *lsn);
        let mut pending = std::collections::BTreeMap::new();
        for (_, mut record) in self.frames {
            let key = record.identity();
            if record.completion {
                record.completion = false;
                if pending.get(&key) == Some(&record) {
                    pending.remove(&key);
                }
            } else {
                pending.entry(key).or_insert(record);
            }
        }
        pending.into_values().collect()
    }
}
