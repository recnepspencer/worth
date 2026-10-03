//! Fallible custody preparation before either replacement record is edited.

use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;

type Commit = worth_runtime_world::facade::CompositeCommitIdentity;

pub(super) struct PreparedObligationTransfer {
    pub(super) slots: Vec<PerformedOutputObligation>,
    pub(super) added_commits: Vec<Commit>,
    pub(super) commit_growth: Option<record_capacity::PreparedSourceCommitGrowth>,
    pub(super) required_member: Option<required_members::PreparedRequiredMember>,
    pub(super) bytes: usize,
}

pub(super) fn prepare(
    state: &DemandRegistryState,
    replaced: &WorthQueryOutputDemandKey,
    replacement: &WorthQueryOutputDemandKey,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<PreparedObligationTransfer>, WorthQueryOutputDemandDenial> {
    state.charge_record_lookup(replaced, admission)?;
    let old = state
        .records
        .get(replaced)
        .expect("old interest owns its record");
    state.charge_record_lookup(replacement, admission)?;
    let next = state
        .records
        .get(replacement)
        .expect("new interest owns its record");
    // The semantic source compares fixed owner-issued coordinates, including
    // its retained identity. No observation payload is reconstructed here.
    charge(admission, old.performed_obligations.len().checked_mul(7))?;
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
    let bytes = count
        .checked_mul(std::mem::size_of::<PerformedOutputObligation>())
        .ok_or_else(admission::performed_obligation_capacity_denial)?;
    if state
        .obligation_reserved_bytes
        .checked_add(bytes)
        .is_none_or(|peak| peak > state.obligation_budget_bytes)
    {
        return Err(admission::performed_obligation_capacity_denial());
    }
    scratch(admission, bytes)?;
    let mut slots = Vec::new();
    slots
        .try_reserve_exact(count)
        .map_err(|_| admission::performed_obligation_capacity_denial())?;
    if slots.capacity() > count {
        return Err(admission::performed_obligation_capacity_denial());
    }

    // Prepare only missing commits. This avoids growing retained record storage
    // for obligations whose provenance is already present in the successor.
    let commit_bytes = moving
        .checked_mul(std::mem::size_of::<Commit>())
        .ok_or_else(admission::performed_obligation_capacity_denial)?;
    scratch(admission, commit_bytes)?;
    let mut added_commits = Vec::new();
    added_commits
        .try_reserve_exact(moving)
        .map_err(|_| admission::performed_obligation_capacity_denial())?;
    if added_commits.capacity() > moving {
        return Err(admission::performed_obligation_capacity_denial());
    }
    charge(admission, old.performed_obligations.len().checked_mul(7))?;
    for obligation in &old.performed_obligations {
        if !obligation.source.same_semantic_source(&replacement.source) {
            continue;
        }
        charge(
            admission,
            next.source_commits
                .len()
                .checked_add(added_commits.len())
                .and_then(|comparisons| comparisons.checked_mul(2)),
        )?;
        if !next.source_commits.contains(&obligation.source_commit)
            && !added_commits.contains(&obligation.source_commit)
        {
            charge(admission, Some(2))?;
            added_commits.push(obligation.source_commit.clone());
        }
    }
    let commit_growth =
        state.prepare_source_commit_growth_for(next, added_commits.len(), admission)?;

    let required_count = state.required_keys.len();
    let levels = usize::BITS as usize - required_count.max(1).leading_zeros() as usize;
    navigate(
        admission,
        required_count
            .min(11)
            .checked_mul(levels)
            .and_then(|comparisons| {
                replacement
                    .producer
                    .len()
                    .checked_add(7)
                    .and_then(|width| comparisons.checked_mul(width))
            })
            .and_then(|work| work.checked_mul(3))
            .and_then(|work| work.checked_add(5)),
    )?;
    // Preparation contains, installation contains/get, and old member removal.
    for key in [replacement, replacement, replaced, replaced] {
        state.charge_record_lookup(key, admission)?;
    }
    let member_bytes = required_members::member_bytes(replacement)
        .ok_or_else(admission::performed_obligation_capacity_denial)?;
    scratch(admission, member_bytes)?;
    charge(admission, replacement.producer.len().checked_add(7))?;
    let required_member = state.prepare_required_member(replacement)?;
    // Two mutable record lookups, initialized slot moves, and the final semantic
    // extraction are prepaid before mem::take can change either owner.
    state.charge_record_lookup(replaced, admission)?;
    state.charge_record_lookup(replacement, admission)?;
    charge(
        admission,
        old.performed_obligations
            .len()
            .checked_mul(7)
            .and_then(|work| work.checked_add(count))
            .and_then(|work| work.checked_add(added_commits.len()))
            // extract_if may compact every unmatched initialized row.
            .and_then(|work| work.checked_add(old.performed_obligations.len() - moving))
            // Retired backing credit is refunded by a scalar owner relock.
            .and_then(|work| work.checked_add(1)),
    )?;
    Ok(Some(PreparedObligationTransfer {
        slots,
        added_commits,
        commit_growth,
        required_member,
        bytes,
    }))
}

fn charge(
    admission: &mut InvalidationEditAdmission,
    work: Option<usize>,
) -> Result<(), WorthQueryOutputDemandDenial> {
    admission
        .charge_external_work(
            u64::try_from(work.ok_or_else(replacement_work_denial)?)
                .map_err(|_| replacement_work_denial())?,
        )
        .map_err(|_| replacement_work_denial())
}

/// Physical required-set navigation is reported apart from declared work.
fn navigate(
    admission: &mut InvalidationEditAdmission,
    work: Option<usize>,
) -> Result<(), WorthQueryOutputDemandDenial> {
    admission
        .charge_ordered_operations(
            1,
            u64::try_from(work.ok_or_else(replacement_work_denial)?)
                .map_err(|_| replacement_work_denial())?,
        )
        .map_err(|_| replacement_work_denial())
}

fn scratch(
    admission: &mut InvalidationEditAdmission,
    bytes: usize,
) -> Result<(), WorthQueryOutputDemandDenial> {
    admission
        .admit_read_scratch(
            u64::try_from(bytes).map_err(|_| admission::performed_obligation_capacity_denial())?,
        )
        .map_err(|stop| match stop {
            worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted { .. }
            | worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow => {
                replacement_work_denial()
            }
            _ => admission::performed_obligation_capacity_denial(),
        })
}
