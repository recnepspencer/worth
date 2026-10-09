//! A superseded row retires once its owners close.
//!
//! A closed dependent row stays cached for an equivalent later demand, and
//! its claims keep the rows it consumed required. A claim on a superseded row
//! keeps that row only so the newest row of its occurrence stays required for
//! the reopen that refreshes over it. Once the superseded row's own demands
//! have closed and that newest row settled Ready, the claim moves to it, and
//! the superseded row, with its custody, retires. While the newest row still
//! refreshes, the superseded row stays: a World that supersedes the refresh
//! gives its Ready back. The outputs the dependent consumed stay recorded
//! in its lineage, which is what certifies it when it reopens.

use std::collections::BTreeMap;
use std::sync::Arc;

use super::super::refreshed_rejoin::{awaited_by_stale_owner, newest_of_occurrence, superseded};
use super::super::{DemandRecord, DemandRegistryState, WorthQueryOutputDemandKey};
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;

type Records = super::super::record_map::DemandRecords;
/// Each claimed row with the rows whose claims hold it.
type Claimants = BTreeMap<WorthQueryOutputDemandKey, Vec<WorthQueryOutputDemandKey>>;

impl DemandRegistryState {
    /// Move closed dependents' claims off every closed superseded row among
    /// `candidates`, then retire each terminal row nothing holds, and each
    /// upstream its release leaves the same way. The caller drops the
    /// returned rows and claims after releasing the registry lock.
    pub(super) fn retire_closed_rows(
        &mut self,
        mut candidates: Vec<WorthQueryOutputDemandKey>,
    ) -> (Vec<DemandRecord>, Vec<Arc<WorthQueryOutputDemandKey>>) {
        let mut retired = Vec::new();
        let mut claims = Vec::new();
        // Built once, on the first closed superseded candidate.
        let mut claimants: Option<Claimants> = None;
        while let Some(key) = candidates.pop() {
            let Some(record) = self.records.get(&key) else {
                continue;
            };
            if closed_cached(record) {
                let prerequisites = record.prerequisites.clone();
                for upstream in prerequisites {
                    candidates.extend(self.follow_newest(&key, &upstream));
                }
            }
            if self.records.get(&key).is_some_and(closed_superseded) {
                let index = claimants.get_or_insert_with(|| claimant_index(&self.records));
                for dependent in index.get(&key).into_iter().flatten() {
                    if self.records.get(dependent).is_some_and(closed_cached) {
                        candidates.extend(self.follow_newest(dependent, &key));
                    }
                }
            }
            let Some(record) = self.records.get(&key) else {
                continue;
            };
            if !record.terminal() || held(&self.records, &key, record) {
                continue;
            }
            let released = self.retire_row(&key, &mut retired);
            candidates.extend(released.iter().map(|upstream| upstream.as_ref().clone()));
            claims.extend(released);
        }
        (retired, claims)
    }

    /// Remove `key`'s row with its claims, which it returns. A superseded
    /// row first hands its settlements to the newest row of its occurrence.
    pub(super) fn retire_row(
        &mut self,
        key: &WorthQueryOutputDemandKey,
        retired: &mut Vec<DemandRecord>,
    ) -> Vec<Arc<WorthQueryOutputDemandKey>> {
        if self.records.get(key).is_some_and(superseded) {
            self.hand_settlements_to_newest(key);
        }
        let released = self.release_record_prerequisites(key);
        self.remove_required_member_if_released(key);
        if let Some(record) = self.records.remove(key) {
            self.obligation_reserved_bytes = self
                .obligation_reserved_bytes
                .saturating_sub(record.obligation_reserved_bytes());
            retired.push(record);
        }
        released
    }

    /// A dependent edge that read a retiring superseded row resolves through
    /// the row's settlement postings. They move to the newest row of its
    /// occurrence, ahead of that row's own, and leave with it.
    fn hand_settlements_to_newest(&mut self, key: &WorthQueryOutputDemandKey) {
        let Some(newest) = newest_of_occurrence(&self.records, key) else {
            return;
        };
        let newest = self
            .required_keys
            .get(&newest)
            .map_or_else(|| Arc::new(newest.clone()), Arc::clone);
        let Some(record) = self.records.get_mut(key) else {
            return;
        };
        if record.settlements.is_empty() {
            return;
        }
        let moved = std::mem::take(&mut record.settlements);
        for (identity, _) in &moved {
            self.settlement_keys.repoint(identity, Arc::clone(&newest));
        }
        let record = self
            .records
            .get_mut(newest.as_ref())
            .expect("the newest row of the occurrence is retained");
        let before = moved.capacity() + record.settlements.capacity();
        let mut joined = Vec::with_capacity(moved.len() + record.settlements.len());
        joined.extend(moved);
        joined.append(&mut record.settlements);
        record.settlements = joined;
        let freed = before.saturating_sub(record.settlements.capacity());
        self.required_reserved_bytes = self.required_reserved_bytes.saturating_sub(
            freed * std::mem::size_of::<(Arc<RecordedSettlementIdentity>, usize)>(),
        );
    }

    /// Move `dependent`'s claim on `upstream` to the newest row of its
    /// occurrence when `upstream` is superseded, its own demands closed, and
    /// that newest row settled Ready. Returns the row the claim left.
    pub(super) fn follow_newest(
        &mut self,
        dependent: &WorthQueryOutputDemandKey,
        upstream: &WorthQueryOutputDemandKey,
    ) -> Option<WorthQueryOutputDemandKey> {
        if !self.records.get(upstream).is_some_and(closed_superseded)
            || !newest_ready(&self.records, upstream)
        {
            return None;
        }
        // The superseded row's claims keep its newest row a required member;
        // the moved claim shares that member's key.
        let newest = newest_of_occurrence(&self.records, upstream)?;
        let newest = Arc::clone(self.required_keys.get(&newest)?);
        let prerequisites = &mut self.records.get_mut(dependent)?.prerequisites;
        let position = prerequisites
            .iter()
            .position(|claimed| claimed.as_ref() == upstream)?;
        if prerequisites.contains(&newest) {
            prerequisites.remove(position);
        } else {
            prerequisites[position] = Arc::clone(&newest);
            self.records
                .get_mut(newest.as_ref())
                .expect("the newest row of the occurrence is retained")
                .framework_required_count += 1;
        }
        self.records
            .get_mut(upstream)
            .expect("the claimed row is retained")
            .framework_required_count -= 1;
        self.remove_required_member_if_released(upstream);
        Some(upstream.clone())
    }
}

/// A row no demand holds that keeps a cached Ready.
pub(super) fn closed_cached(record: &DemandRecord) -> bool {
    record.interests == 0 && record.required_interests == 0 && record.has_cached_ready()
}

/// A superseded row no demand of its own holds.
pub(super) fn closed_superseded(record: &DemandRecord) -> bool {
    record.interests == 0 && record.required_interests == 0 && superseded(record)
}

/// The newest row of `key`'s occurrence holds a Ready no refresh replaced.
pub(super) fn newest_ready(records: &Records, key: &WorthQueryOutputDemandKey) -> bool {
    newest_of_occurrence(records, key)
        .and_then(|newest| records.get(&newest))
        .is_some_and(|newest| newest.has_cached_ready() && !superseded(newest))
}

/// The claimants of every claimed row, in one pass over the rows.
fn claimant_index(records: &Records) -> Claimants {
    let mut index = Claimants::new();
    for (dependent, record) in records {
        for claimed in &record.prerequisites {
            index
                .entry(claimed.as_ref().clone())
                .or_default()
                .push(dependent.clone());
        }
    }
    index
}

/// The rows `dependent` accepts whose claims hold `key`.
pub(super) fn claimants_where(
    records: &Records,
    key: &WorthQueryOutputDemandKey,
    dependent: impl Fn(&DemandRecord) -> bool,
) -> Vec<WorthQueryOutputDemandKey> {
    records
        .iter()
        .filter(|(_, record)| {
            dependent(record)
                && record
                    .prerequisites
                    .iter()
                    .any(|claimed| claimed.as_ref() == key)
        })
        .map(|(dependent, _)| dependent.clone())
        .collect()
}

/// A demand, a dependent's claim, or unfinished work still holds the row.
pub(super) fn held(
    records: &Records,
    key: &WorthQueryOutputDemandKey,
    record: &DemandRecord,
) -> bool {
    record.interests != 0
        || record.required_interests != 0
        || record.framework_required_count != 0
        || record.prepared_prerequisite_claims != 0
        || record.pending_cleanup_queued
        || !record.performed_obligations.is_empty()
        || record.performed_source.is_some()
        || record.held_successor.is_some()
        || awaited_by_stale_owner(records, key)
}
