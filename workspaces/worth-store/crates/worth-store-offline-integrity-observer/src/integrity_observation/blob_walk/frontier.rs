use std::collections::BTreeMap;

use super::super::blob_record::BlobFact;
use super::super::OfflinePhysicalDamageCause as Cause;
use super::{damage, Outcome, Selected};

#[cfg(test)]
mod graph_tests;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
pub(super) struct Claim {
    pub(super) store: [u8; 16],
    pub(super) session: [u8; 16],
    pub(super) declaration_record: [u8; 24],
    pub(super) declaration_digest: [u8; 32],
    pub(super) next_chunk_ordinal: u64,
    pub(super) durable_bytes: u64,
    pub(super) last_chunk_record: [u8; 24],
    pub(super) last_chunk_digest: [u8; 32],
}

struct ChunkClaim {
    ordinal: u64,
    record: [u8; 24],
    digest: [u8; 32],
    length: u64,
    outcome: Outcome,
    duplicate_ordinal: bool,
    prefix: Prefix,
}

#[derive(Clone)]
enum Prefix {
    Complete(u64),
    Uncertain(Outcome),
    Damaged,
}

/// One admission-bounded ordinal index is built for the selected graph, never
/// one scan or sort per frontier record.
pub(super) struct ClaimIndex {
    by_session: BTreeMap<[u8; 16], BTreeMap<u64, ChunkClaim>>,
    unlocated_uncertainty: Option<Outcome>,
}

impl ClaimIndex {
    pub(super) fn new(selected: &[Selected], maximum_claims: u64) -> Option<Self> {
        if !selected
            .iter()
            .any(|row| matches!(row.fact.as_ref(), Some(BlobFact::Frontier { .. })))
        {
            return Some(Self {
                by_session: BTreeMap::new(),
                unlocated_uncertainty: None,
            });
        }
        let unlocated_uncertainty = selected.iter().find_map(|row| {
            row.fact
                .is_none()
                .then(|| match &row.outcome {
                    Outcome::Unknown(_) | Outcome::Indeterminate(_) | Outcome::Unsupported(_) => {
                        Some(row.outcome.clone())
                    }
                    Outcome::Intact | Outcome::Damaged(_) => None,
                })
                .flatten()
        });
        let mut by_session: BTreeMap<[u8; 16], BTreeMap<u64, ChunkClaim>> = BTreeMap::new();
        let mut count = 0_u64;
        for row in selected {
            let (session, ordinal, length, digest, selected_chunk) = match &row.fact {
                Some(BlobFact::Chunk {
                    session,
                    ordinal,
                    length,
                    digest,
                    ..
                }) => (session, ordinal, length, digest, row.record),
                Some(BlobFact::ReuseClaim {
                    session,
                    ordinal,
                    length,
                    digest,
                    chunk_record,
                    ..
                }) => (session, ordinal, length, digest, *chunk_record),
                _ => continue,
            };
            let claims = by_session.entry(*session).or_default();
            match claims.entry(*ordinal) {
                std::collections::btree_map::Entry::Occupied(mut occupied) => {
                    let prior = occupied.get_mut();
                    // An exact duplicate observation of the same selected
                    // claim is harmless; a distinct selected claim is not.
                    if prior.record != selected_chunk
                        || prior.digest != *digest
                        || prior.length != *length
                        || matches!(row.fact, Some(BlobFact::ReuseClaim { .. }))
                    {
                        prior.duplicate_ordinal = true;
                    }
                    prior.outcome = merge_observation(&prior.outcome, &row.outcome);
                }
                std::collections::btree_map::Entry::Vacant(vacant) => {
                    count = count.checked_add(1)?;
                    if count > maximum_claims {
                        return None;
                    }
                    vacant.insert(ChunkClaim {
                        ordinal: *ordinal,
                        record: selected_chunk,
                        digest: *digest,
                        length: *length,
                        outcome: row.outcome.clone(),
                        duplicate_ordinal: false,
                        prefix: Prefix::Complete(0),
                    });
                }
            }
        }
        for claims in by_session.values_mut() {
            let mut prefix = Prefix::Complete(0);
            for (position, claim) in claims.values_mut().enumerate() {
                prefix = next_prefix(
                    prefix,
                    claim,
                    position as u64,
                    unlocated_uncertainty.as_ref(),
                );
                claim.prefix = prefix.clone();
            }
        }
        Some(Self {
            by_session,
            unlocated_uncertainty,
        })
    }

    fn last(&self, session: [u8; 16], next_ordinal: u64) -> Option<&ChunkClaim> {
        self.by_session
            .get(&session)?
            .get(&next_ordinal.checked_sub(1)?)
    }
}

fn merge_observation(prior: &Outcome, next: &Outcome) -> Outcome {
    match (prior, next) {
        (Outcome::Damaged(_), _) => prior.clone(),
        (_, Outcome::Damaged(_)) => next.clone(),
        (Outcome::Intact, _) => next.clone(),
        _ => prior.clone(),
    }
}

fn next_prefix(
    prior: Prefix,
    claim: &ChunkClaim,
    expected_ordinal: u64,
    unlocated_uncertainty: Option<&Outcome>,
) -> Prefix {
    if claim.duplicate_ordinal {
        return Prefix::Damaged;
    }
    let Prefix::Complete(bytes) = prior else {
        return prior;
    };
    if claim.ordinal != expected_ordinal {
        return unlocated_uncertainty
            .cloned()
            .map_or(Prefix::Damaged, Prefix::Uncertain);
    }
    match &claim.outcome {
        Outcome::Intact => bytes
            .checked_add(claim.length)
            .map_or(Prefix::Damaged, Prefix::Complete),
        Outcome::Unknown(_) | Outcome::Indeterminate(_) | Outcome::Unsupported(_) => {
            Prefix::Uncertain(claim.outcome.clone())
        }
        Outcome::Damaged(_) => Prefix::Damaged,
    }
}

pub(super) fn validate(
    selected: &[Selected],
    records: &BTreeMap<[u8; 24], usize>,
    claims: &ClaimIndex,
    frontier: Claim,
) -> Outcome {
    let declaration = records
        .get(&frontier.declaration_record)
        .and_then(|index| selected.get(*index));
    let declared = declaration.is_some_and(|row| {
        row.outcome == Outcome::Intact
            && matches!(&row.fact,
            Some(BlobFact::Declaration { store, session, frame_digest, .. })
                if *store == frontier.store
                    && *session == frontier.session
                    && *frame_digest == frontier.declaration_digest)
    });
    if !declared {
        return damage(Cause::Pointer);
    }
    let Some(last) = claims.last(frontier.session, frontier.next_chunk_ordinal) else {
        return claims
            .unlocated_uncertainty
            .clone()
            .unwrap_or_else(|| damage(Cause::Pointer));
    };
    if last.ordinal.checked_add(1) != Some(frontier.next_chunk_ordinal)
        || last.record != frontier.last_chunk_record
        || last.digest != frontier.last_chunk_digest
    {
        return damage(Cause::Pointer);
    }
    match &last.prefix {
        Prefix::Complete(bytes) if *bytes == frontier.durable_bytes => Outcome::Intact,
        Prefix::Uncertain(outcome) => outcome.clone(),
        Prefix::Complete(_) | Prefix::Damaged => damage(Cause::Pointer),
    }
}
