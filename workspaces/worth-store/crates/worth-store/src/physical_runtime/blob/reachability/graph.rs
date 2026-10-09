use std::collections::{BTreeMap, BTreeSet};

use crate::physical_runtime::blob::ingest::validate_selected_claims;

mod control;
mod published;
mod reuse;

use super::scan::{Role, SelectedInventory};
use super::{BlobReachabilityFailure as Failure, BlobReachabilityRecord, BlobRecordReachability};

pub(super) fn classify(
    inventory: &SelectedInventory,
) -> Result<(Vec<BlobReachabilityRecord>, u64), Failure> {
    let mut abandoned = BTreeSet::new();
    let mut published = BTreeSet::new();
    let released = control::released_sessions(inventory)?;
    let mut declared = BTreeSet::new();
    let mut declarations = BTreeMap::new();
    let mut frontier = BTreeMap::new();
    for fact in inventory.facts.values().filter(|fact| fact.current) {
        match fact.role {
            Role::Abandonment(session) => {
                abandoned.insert(session);
            }
            Role::Publication(session) => {
                published.insert(session);
            }
            Role::Declaration(session) => {
                declared.insert(session);
                let declaration = fact.declaration.ok_or(Failure::ConflictingSelectedFate)?;
                if declarations.insert(session, declaration).is_some() {
                    return Err(Failure::ConflictingSelectedFate);
                }
            }
            Role::Frontier {
                session,
                next_ordinal,
            } => {
                frontier
                    .entry(session)
                    .and_modify(|previous: &mut u64| *previous = (*previous).max(next_ordinal))
                    .or_insert(next_ordinal);
            }
            _ => {}
        }
    }
    let active = declared
        .difference(&abandoned)
        .copied()
        .filter(|session| !published.contains(session) && !released.contains(session))
        .collect::<BTreeSet<_>>();

    // C9's selected-claim validator is the authority for occurrence order,
    // exact chunk lengths and the frontier binding to its last record/digest.
    // A shared session ID and low ordinal alone are not a liveness edge.
    for session in &active {
        let mut claims = Vec::new();
        claims
            .try_reserve_exact(inventory.facts.len())
            .map_err(|_| Failure::MetadataUnavailable)?;
        for fact in inventory.facts.values().filter(|fact| fact.current) {
            let same_session = match fact.role {
                Role::Chunk { session: found, .. }
                | Role::ReuseClaim { session: found, .. }
                | Role::Frontier { session: found, .. }
                | Role::Tree(found) => found == *session,
                _ => false,
            };
            if same_session {
                claims.push(fact.claim.ok_or(Failure::ConflictingSelectedFate)?);
            }
        }
        validate_selected_claims(
            &mut claims,
            *declarations
                .get(session)
                .ok_or(Failure::ConflictingSelectedFate)?,
        )
        .map_err(|_| Failure::ConflictingSelectedFate)?;
    }

    let mut live_blob = BTreeSet::new();
    let mut traversed = 0_u64;
    published::authenticate(inventory, &mut live_blob, &mut traversed)?;
    for (record, fact) in inventory.facts.iter().filter(|(_, fact)| fact.current) {
        match fact.role {
            Role::Declaration(session)
                if active.contains(&session) || published.contains(&session) =>
            {
                live_blob.insert(*record);
            }
            Role::Frontier { session, .. } if active.contains(&session) => {
                live_blob.insert(*record);
            }
            Role::Chunk { session, ordinal } | Role::ReuseClaim { session, ordinal }
                if active.contains(&session)
                    && frontier.get(&session).is_some_and(|next| ordinal < *next) =>
            {
                live_blob.insert(*record);
            }
            Role::Control | Role::ReleasedControl(_) => {
                live_blob.insert(*record);
            }
            _ => {}
        }
    }
    reuse::authenticate(inventory, &mut live_blob, &mut traversed)?;

    let mut live_derived = BTreeSet::new();
    let mut derived_stack = inventory
        .current_derived_directory
        .into_iter()
        .collect::<Vec<_>>();
    while let Some(record) = derived_stack.pop() {
        let Some(fact) = inventory.facts.get(&record) else {
            return Err(Failure::ConflictingSelectedFate);
        };
        if !fact.current || fact.role != Role::Derived {
            return Err(Failure::ConflictingSelectedFate);
        }
        if !live_derived.insert(record) {
            continue;
        }
        for target in &fact.edges {
            traversed = traversed
                .checked_add(1)
                .ok_or(Failure::EdgeBoundExhausted)?;
            if traversed > inventory.maximum_edges() {
                return Err(Failure::EdgeBoundExhausted);
            }
            derived_stack.push(*target);
        }
    }

    let records = inventory.facts.iter().filter(|(_, fact)| fact.role != Role::Opaque).map(|(record, fact)| {
        let class = if fact.current && (live_blob.contains(record) || live_derived.contains(record)) {
            BlobRecordReachability::Reachable
        } else if fact.held {
            BlobRecordReachability::HeldOnly
        } else if matches!(fact.role, Role::Abandonment(_))
            || matches!(fact.role, Role::Declaration(session) | Role::Tree(session) if abandoned.contains(&session))
            || matches!(fact.role, Role::Chunk { session, .. } | Role::Frontier { session, .. } | Role::ReuseClaim { session, .. } if abandoned.contains(&session)) {
            BlobRecordReachability::FailedOperationResidue
        } else if matches!(fact.role,
            Role::Chunk { session, ordinal } | Role::ReuseClaim { session, ordinal }
                if active.contains(&session) && ordinal >= frontier.get(&session).copied().unwrap_or(0)) {
            BlobRecordReachability::FailedOperationResidue
        } else if fact.role == Role::Derived {
            BlobRecordReachability::DerivedResidue
        } else {
            BlobRecordReachability::Unreferenced
        };
        BlobReachabilityRecord { record: *record, class }
    }).collect();
    Ok((records, traversed))
}

#[cfg(test)]
mod tests;
