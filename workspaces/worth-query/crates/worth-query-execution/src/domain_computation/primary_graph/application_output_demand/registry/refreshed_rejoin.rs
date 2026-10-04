//! An open interest follows a refresh of its own output.
//!
//! A refresh admits the newer source of the same producer occurrence and
//! supersedes the older row. Any demand still holding the older row rejoins
//! the newest row of that occurrence instead of failing; a stop that was not
//! a refresh, or a refresh that itself stopped, stays terminal.

use std::collections::BTreeMap;
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
        // One pass over the rows compares each key's occurrence once.
        admission
            .charge_external_work(u64::try_from(state.records.len()).map_err(|_| work_denial())?)
            .map_err(|_| work_denial())?;
        let newest = state
            .records
            .iter()
            .filter(|(key, _)| key.same_occurrence(&stale.key))
            .max_by_key(|(key, _)| key.source.observation_generation())
            .map(|(key, _)| key.clone());
        let Some(newest) = newest
            .filter(|key| key.replacement_order(&stale.key) == Some(std::cmp::Ordering::Greater))
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
/// or releases.
pub(super) fn awaited_by_stale_owner(
    records: &BTreeMap<WorthQueryOutputDemandKey, DemandRecord>,
    key: &WorthQueryOutputDemandKey,
) -> bool {
    let mut awaited = false;
    for (other, record) in occurrence_rows(records, key) {
        if other == key {
            continue;
        }
        match other.replacement_order(key) {
            Some(std::cmp::Ordering::Greater) => return false,
            Some(std::cmp::Ordering::Less) => {
                awaited |= (record.interests != 0 || record.framework_required_count != 0)
                    && superseded(record);
            }
            _ => {}
        }
    }
    awaited
}

/// The newest row of `key`'s occurrence. A closing stale owner of `key` may
/// have been the last one awaiting it.
pub(super) fn newest_of_occurrence(
    records: &BTreeMap<WorthQueryOutputDemandKey, DemandRecord>,
    key: &WorthQueryOutputDemandKey,
) -> Option<WorthQueryOutputDemandKey> {
    occurrence_rows(records, key)
        .map(|(other, _)| other)
        .filter(|other| other.replacement_order(key) == Some(std::cmp::Ordering::Greater))
        .max_by_key(|other| other.source.observation_generation())
        .cloned()
}

/// The rows of `key`'s occurrence. Keys order by producer and occurrence
/// before generation, so they are the contiguous run of rows around `key`.
pub(super) fn occurrence_rows<'records>(
    records: &'records BTreeMap<WorthQueryOutputDemandKey, DemandRecord>,
    key: &'records WorthQueryOutputDemandKey,
) -> impl Iterator<Item = (&'records WorthQueryOutputDemandKey, &'records DemandRecord)> {
    use std::ops::Bound::{Excluded, Included, Unbounded};
    let before = records
        .range::<WorthQueryOutputDemandKey, _>((Unbounded, Excluded(key)))
        .rev()
        .take_while(move |(other, _)| other.same_occurrence(key));
    let from = records
        .range::<WorthQueryOutputDemandKey, _>((Included(key), Unbounded))
        .take_while(move |(other, _)| other.same_occurrence(key));
    before.chain(from)
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "refreshed output rejoin exceeds request work",
    )
}
