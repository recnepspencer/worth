//! Required work outranks a cached Ready nothing holds.
//!
//! A closed row keeps its Ready for an equivalent later demand, on the
//! required custody its demand reserved. When required work would otherwise
//! be refused that custody, closed cached rows retire, with the rows only
//! their claims kept, until the work fits. A refusal that remains is one no
//! cache could have avoided.

use std::sync::atomic::Ordering;
use std::sync::Arc;

use super::super::ready_backing::PreparedReadyBacking;
use super::super::refreshed_rejoin::{awaited_by_stale_owner, superseded};
use super::super::{
    DemandRecord, DemandRegistryState, WorthQueryOutputDemandKey, WorthQueryOutputDemandRegistry,
};
use super::closed_retirement::{
    claimants_where, closed_cached, closed_superseded, held, newest_ready,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

impl WorthQueryOutputDemandRegistry {
    /// Make room for `bytes` more required custody by retiring closed cached
    /// rows. The retired rows drop after the registry lock is released.
    pub(in crate::domain_computation::primary_graph) fn reclaim_cached_rows(
        &self,
        bytes: usize,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let mut retired = Vec::new();
        let mut claims = Vec::new();
        let reclaimed = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let reclaimed = state.reclaim_cached_rows(bytes, &mut retired, &mut claims, admission);
            if !retired.is_empty() {
                state.prune_completed_custody();
            }
            reclaimed
        };
        drop(retired);
        drop(claims);
        reclaimed
    }
}

impl DemandRegistryState {
    fn reclaim_cached_rows(
        &mut self,
        bytes: usize,
        retired: &mut Vec<DemandRecord>,
        claims: &mut Vec<Arc<WorthQueryOutputDemandKey>>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        // Each retired Ready cell returns its custody once it drops.
        let mut returning = 0_usize;
        loop {
            let retained = self
                .required_custody_retained_bytes
                .load(Ordering::Acquire)
                .saturating_sub(returning);
            let fits = self
                .required_reserved_bytes
                .checked_add(retained)
                .and_then(|total| total.checked_add(bytes))
                .is_some_and(|total| total <= self.required_budget_bytes);
            if fits {
                return Ok(());
            }
            let Some(key) = self.reclaimable_row(admission)? else {
                return Ok(());
            };
            for dependent in claimants_where(&self.records, &key, |_| true) {
                self.follow_newest(&dependent, &key);
            }
            claims.extend(self.retire_row(&key, retired));
            returning = returning.saturating_add(PreparedReadyBacking::retained_bytes());
        }
    }

    /// The first closed cached row nothing holds, or else the first closed
    /// superseded row only dependents' claims hold once the newest row of its
    /// occurrence is Ready: their edges already resolve through that row.
    fn reclaimable_row(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<WorthQueryOutputDemandKey>, WorthQueryOutputDemandDenial> {
        // One pass over the rows; each candidate compares the rows of its
        // occurrence once more.
        let rows = u64::try_from(self.records.len()).map_err(|_| work_denial())?;
        admission
            .charge_external_work(rows)
            .map_err(|_| work_denial())?;
        let mut behind_ready = None;
        for (key, record) in &self.records {
            if closed_cached(record) && !superseded(record) {
                admission
                    .charge_external_work(rows)
                    .map_err(|_| work_denial())?;
                if !held(&self.records, key, record) {
                    return Ok(Some(key.clone()));
                }
            } else if behind_ready.is_none() && closed_superseded(record) {
                admission
                    .charge_external_work(rows.saturating_mul(2))
                    .map_err(|_| work_denial())?;
                if claimed_only(record)
                    && !awaited_by_stale_owner(&self.records, key)
                    && newest_ready(&self.records, key)
                {
                    behind_ready = Some(key.clone());
                }
            }
        }
        Ok(behind_ready)
    }
}

/// Only dependents' claims hold the row.
fn claimed_only(record: &DemandRecord) -> bool {
    record.prepared_prerequisite_claims == 0
        && !record.pending_cleanup_queued
        && record.performed_obligations.is_empty()
        && record.performed_source.is_none()
        && record.held_successor.is_none()
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "cached row reclamation exceeds request work",
    )
}
