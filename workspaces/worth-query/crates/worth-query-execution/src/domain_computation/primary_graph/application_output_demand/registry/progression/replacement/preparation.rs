//! Fallible custody preparation before either replacement record is edited.

use super::*;

type Commit = worth_runtime_world::facade::CompositeCommitIdentity;

pub(super) struct PreparedObligationTransfer<'a> {
    pub(super) mutation: &'a mut DemandRegistryState,
    pub(super) slots: Vec<PerformedOutputObligation>,
    pub(super) added_commits: Vec<Commit>,
    pub(super) commit_growth: Option<record_preparation::PreparedSourceCommitGrowth>,
    pub(super) required_member: Option<required_members::PreparedRequiredMember>,
}

pub(super) fn prepare<'a>(
    state: &'a mut DemandRegistryState,
    replaced: &WorthQueryOutputDemandKey,
    replacement: &WorthQueryOutputDemandKey,
) -> Result<Option<PreparedObligationTransfer<'a>>, WorthQueryOutputDemandDenial> {
    let old = state
        .records
        .get(replaced)
        .expect("old interest owns its record");

    let next = state
        .records
        .get(replacement)
        .expect("new interest owns its record");
    // The semantic source compares fixed owner-issued coordinates, including
    // its retained identity. No observation payload is reconstructed here.

    let moving = old
        .performed_obligations
        .iter()
        .filter(|obligation| obligation.source.same_semantic_source(&replacement.source))
        .count();
    if moving == 0 {
        return Ok(None);
    }
    let count = moving
        .checked_add(next.performed_obligations.len())
        .ok_or_else(admission::performed_obligation_capacity_denial)?;
    let mut slots = Vec::new();
    slots
        .try_reserve_exact(count)
        .map_err(|_| admission::performed_obligation_capacity_denial())?;
    if slots.capacity() > count {
        return Err(admission::performed_obligation_capacity_denial());
    }

    // Prepare only missing commits. This avoids growing retained record storage
    // for obligations whose provenance is already present in the successor.
    let mut added_commits = Vec::new();
    added_commits
        .try_reserve_exact(moving)
        .map_err(|_| admission::performed_obligation_capacity_denial())?;
    if added_commits.capacity() > moving {
        return Err(admission::performed_obligation_capacity_denial());
    }

    for obligation in &old.performed_obligations {
        if !obligation.source.same_semantic_source(&replacement.source) {
            continue;
        }

        if !next.source_commits.contains(&obligation.source_commit)
            && !added_commits.contains(&obligation.source_commit)
        {
            added_commits.push(obligation.source_commit.clone());
        }
    }
    let commit_growth = state.prepare_source_commit_growth_for(next, added_commits.len())?;

    let required_member = state.prepare_required_member(replacement)?;
    // Two mutable record lookups, initialized slot moves, and the final semantic
    // extraction are prepaid before mem::take can change either owner.

    let mutation = state;
    Ok(Some(PreparedObligationTransfer {
        mutation,
        slots,
        added_commits,
        commit_growth,
        required_member,
    }))
}
