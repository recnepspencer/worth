//! An open interest follows a refresh of its own output.
//!
//! A refresh admits the newer source of the same producer occurrence and
//! supersedes the older row. Any demand still holding the older row rejoins
//! the newest row of that occurrence instead of failing; a stop that was not
//! a refresh, or a refresh that itself stopped, stays terminal.

use std::cmp::Ordering;
use std::sync::Arc;

use super::admission::{accepts_semantic_join, interest};
use super::source_readmission::RequiredOutputReadmission;
use super::{
    DemandRecord, DemandState, WorthQueryOutputAdvancement, WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandKey, WorthQueryOutputDemandRegistry,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

mod walk_charge;
pub(super) use walk_charge::{
    charge_whole_map, infallible, unpaid_under_request, OccurrenceSearchCharge, WholeMapPasses,
};

impl WorthQueryOutputDemandRegistry {
    /// Issue an interest in the newest joinable row of the same occurrence
    /// when `stale`'s row was superseded by a newer source. The caller swaps
    /// it in and drops `stale`, which releases the superseded row.
    pub(in crate::domain_computation::primary_graph) fn rejoin_refreshed(
        &self,
        stale: &WorthQueryOutputDemandInterest,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        Option<(
            WorthQueryOutputDemandInterest,
            Arc<RequiredOutputReadmission>,
        )>,
        WorthQueryOutputDemandDenial,
    > {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.charge_record_lookup(&stale.key, admission)?;
        let Some(record) = state.records.get(&stale.key) else {
            return Ok(None);
        };
        if !superseded(record) {
            return Ok(None);
        }
        let (occurrence, scope) = (record.product_occurrence, record.source_scope);
        // Only this occurrence is traversed; unrelated records affect the
        // ordered navigation report, never this request's declared work.
        let Some(newest) =
            newest_matching_row(&state.records, &stale.key, admission, |key, record| {
                refresh_order(key, &stale.key) == Some(Ordering::Greater)
                    && record.product_occurrence == occurrence
                    && record.source_scope == scope
            })?
        else {
            return Ok(None);
        };
        state.charge_record_lookup(&newest, admission)?;
        let member = state.prepare_required_member(&newest)?;
        let record = state.records.get_mut(&newest).expect("newest row exists");
        let joinable = accepts_semantic_join(record)
            && record.product_occurrence == occurrence
            && record.source_scope == scope;
        let Some(readmission) = record.readmission_source.as_ref().filter(|_| joinable) else {
            return Ok(None);
        };
        let readmission = Arc::clone(readmission);
        record.interests = record.interests.saturating_add(1);
        record.required_interests += usize::from(stale.requires_output);
        let rejoined = interest(self, newest, record, stale.requires_output);
        state.install_required_member(member);
        Ok(Some((rejoined, readmission)))
    }
}

/// A row its owner still holds after a newer source of its occurrence
/// replaced it. The owner rejoins on its next advance.
pub(super) fn superseded(record: &DemandRecord) -> bool {
    match &record.state {
        DemandState::Output(output) => matches!(
            &output.advancement,
            WorthQueryOutputAdvancement::Stopped { denial, .. }
                if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded
        ),
        DemandState::Failed(denial) => {
            denial.kind() == WorthQueryOutputDemandDenialKind::Superseded
        }
        _ => false,
    }
}

/// The newest row of an occurrence belongs to the owners still holding its
/// superseded rows, not to whichever advance refreshed it: a demand, or a
/// dependent whose settled row consumed one. While a dependent is open the
/// outputs it consumes stay required, through every refresh of them. The row
/// keeps its required membership and source custody until each owner rejoins
/// or releases. A newer row takes the occurrence over once it publishes:
/// one that ends first gives the occurrence back to this row.
/// Charge only the occurrence-local rows actually inspected. A row that is
/// already required does not need this traversal at all.
pub(super) fn awaited_by_stale_owner<C: OccurrenceSearchCharge>(
    records: &super::record_map::DemandRecords,
    key: &WorthQueryOutputDemandKey,
    charge: &mut C,
) -> Result<bool, C::Refusal> {
    let mut awaited = false;
    for (other, record) in occurrence_rows(records, key, charge)? {
        charge.inspect_row()?;
        if other == key {
            continue;
        }
        match refresh_order(other, key) {
            Some(Ordering::Greater) if !record.unpublished_new_key() => return Ok(false),
            Some(Ordering::Less) => {
                awaited |= (record.interests != 0 || record.framework_required_count != 0)
                    && superseded(record);
            }
            _ => {}
        }
    }
    Ok(awaited)
}

/// Whether the Ready of `key`'s row, replaced by a refresh that has yet to
/// publish, still answers for a stale owner. A refresh that ends unpublished
/// gives the occurrence back to the Ready it replaced, never to an older one.
pub(super) fn replaced_under_refresh<C: OccurrenceSearchCharge>(
    records: &super::record_map::DemandRecords,
    key: &WorthQueryOutputDemandKey,
    charge: &mut C,
) -> Result<bool, C::Refusal> {
    let mut newer = false;
    for (other, _) in occurrence_rows(records, key, charge)? {
        charge.inspect_row()?;
        if refresh_order(other, key) == Some(Ordering::Greater) {
            newer = true;
            break;
        }
    }
    Ok(newer && awaited_by_stale_owner(records, key, charge)?)
}

/// The closed superseded rows the published row at `key` replaced. Kept
/// while it refreshed, they answer for nothing once it has published.
pub(super) fn replaced_by_published<C: OccurrenceSearchCharge>(
    records: &super::record_map::DemandRecords,
    key: &WorthQueryOutputDemandKey,
    charge: &mut C,
) -> Result<Vec<WorthQueryOutputDemandKey>, C::Refusal> {
    if !matches!(records.get(key), Some(record) if matches!(record.state, DemandState::Output(_))) {
        return Ok(Vec::new());
    }
    let mut replaced = Vec::new();
    for (other, record) in occurrence_rows(records, key, charge)? {
        charge.inspect_row()?;
        if refresh_order(other, key) == Some(Ordering::Less)
            && record.interests == 0
            && superseded(record)
        {
            replaced.push(other.clone());
        }
    }
    Ok(replaced)
}

/// The newest row of `key`'s occurrence. A closing stale owner of `key` may
/// have been the last one awaiting it.
pub(super) fn newest_of_occurrence<C: OccurrenceSearchCharge>(
    records: &super::record_map::DemandRecords,
    key: &WorthQueryOutputDemandKey,
    charge: &mut C,
) -> Result<Option<WorthQueryOutputDemandKey>, C::Refusal> {
    let rows = occurrence_rows(records, key, charge)?;
    newest_lawful_successor(rows.map(|(other, _)| other), key, charge).map(|newest| newest.cloned())
}

/// Incomparable bindings never become a successor through iteration order.
pub(super) fn newest_lawful_successor<'keys, C: OccurrenceSearchCharge>(
    keys: impl Iterator<Item = &'keys WorthQueryOutputDemandKey>,
    predecessor: &WorthQueryOutputDemandKey,
    charge: &mut C,
) -> Result<Option<&'keys WorthQueryOutputDemandKey>, C::Refusal> {
    newest_matching_key(keys.map(|key| (key, ())), charge, |candidate, _| {
        refresh_order(candidate, predecessor) == Some(Ordering::Greater)
    })
}

/// Select a lawful row with the caller's charge for the walk and inspected rows.
pub(super) fn newest_matching_row<C: OccurrenceSearchCharge>(
    records: &super::record_map::DemandRecords,
    requested: &WorthQueryOutputDemandKey,
    charge: &mut C,
    accepts: impl Fn(&WorthQueryOutputDemandKey, &DemandRecord) -> bool,
) -> Result<Option<WorthQueryOutputDemandKey>, C::Refusal> {
    let rows = occurrence_rows(records, requested, charge)?;
    newest_matching_key(rows, charge, |key, record| accepts(key, record))
        .map(|newest| newest.cloned())
}

/// Incomparable candidates cannot become newest through iteration order.
fn newest_matching_key<'keys, R, C: OccurrenceSearchCharge>(
    rows: impl Iterator<Item = (&'keys WorthQueryOutputDemandKey, R)>,
    charge: &mut C,
    accepts: impl Fn(&WorthQueryOutputDemandKey, &R) -> bool,
) -> Result<Option<&'keys WorthQueryOutputDemandKey>, C::Refusal> {
    let mut newest: Option<&WorthQueryOutputDemandKey> = None;
    for (candidate, record) in rows {
        charge.inspect_row()?;
        if !accepts(candidate, &record) {
            continue;
        }
        match newest {
            None => newest = Some(candidate),
            Some(current) => match refresh_order(candidate, current) {
                Some(Ordering::Greater) => newest = Some(candidate),
                Some(Ordering::Less) => {}
                Some(Ordering::Equal) if candidate == current => {}
                _ => return Ok(None),
            },
        }
    }
    Ok(newest)
}

/// How `row` orders against `other` among the rows of one occurrence. A
/// refreshed Stable alias keeps its semantic source and takes the newer
/// observation, so two rows of one semantic source order by generation.
pub(super) fn refresh_order(
    row: &WorthQueryOutputDemandKey,
    other: &WorthQueryOutputDemandKey,
) -> Option<Ordering> {
    row.replacement_order(other).map(|order| {
        order.then_with(|| {
            row.source
                .observation_generation()
                .cmp(&other.source.observation_generation())
        })
    })
}

/// The rows of `key`'s occurrence. Keys order by family and source occurrence
/// before generation, so they are the contiguous run of rows around `key`.
pub(super) fn occurrence_rows<'a, C: OccurrenceSearchCharge>(
    records: &'a super::record_map::DemandRecords,
    key: &'a WorthQueryOutputDemandKey,
    charge: &mut C,
) -> Result<impl Iterator<Item = (&'a WorthQueryOutputDemandKey, &'a DemandRecord)>, C::Refusal> {
    charge.occurrence_path(records.len())?;
    Ok(records.occurrence_rows(key))
}

pub(super) fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "refreshed output rejoin exceeds request work",
    )
}
