//! Required work outranks a cached Ready nothing holds.
//!
//! A closed row keeps its Ready for an equivalent later demand, on the
//! required custody its demand reserved. When required work would otherwise
//! be refused that custody, closed cached rows retire, with the rows only
//! their claims kept, until the work fits. A refusal that remains is one no
//! cache could have avoided.
//!
//! A cached Ready's settlement also keeps the product observation it reads.
//! A caller the branch refuses an observation retires one closed cached row,
//! or with none left releases a performed source nobody holds, and asks
//! again: a refusal it still reports is one no unheld observation caused.

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

    /// Release one product observation nothing holds: retire a closed
    /// cached row, or with none left release an unheld performed source.
    /// Whether one was released.
    pub(in crate::domain_computation::primary_graph) fn reclaim_unheld_observation(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let mut retired = Vec::new();
        let mut claims = Vec::new();
        let mut released = None;
        let reclaimed = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let reclaimed =
                state.retire_next_cached_row(&mut None, &mut retired, &mut claims, admission);
            if !retired.is_empty() {
                state.prune_completed_custody();
            }
            match reclaimed {
                Ok(false) => state
                    .release_unheld_performed_source(admission)
                    .map(|source| {
                        released = source;
                        released.is_some()
                    }),
                reclaimed => reclaimed,
            }
        };
        drop(retired);
        drop(claims);
        drop(released);
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
        // The last superseded row a claim still held; rows are tried in order.
        let mut still_claimed = None;
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
            if fits
                || !self.retire_next_cached_row(&mut still_claimed, retired, claims, admission)?
            {
                return Ok(());
            }
            returning = returning.saturating_add(PreparedReadyBacking::retained_bytes());
        }
    }

    /// Retire the next reclaimable row. Whether one was left to retire.
    fn retire_next_cached_row(
        &mut self,
        still_claimed: &mut Option<WorthQueryOutputDemandKey>,
        retired: &mut Vec<DemandRecord>,
        claims: &mut Vec<Arc<WorthQueryOutputDemandKey>>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        loop {
            let Some(key) = self.reclaimable_row(still_claimed.as_ref(), admission)? else {
                return Ok(false);
            };
            for dependent in claimants_where(&self.records, &key, |_| true) {
                self.follow_newest(&dependent, &key);
            }
            // A claim that did not move still resolves through this row.
            if self
                .records
                .get(&key)
                .is_some_and(|record| record.framework_required_count != 0)
            {
                *still_claimed = Some(key);
                continue;
            }
            claims.extend(self.retire_row(&key, retired));
            return Ok(true);
        }
    }

    /// The first closed cached row nothing holds, or else the first closed
    /// superseded row only dependents' claims hold once the newest row of its
    /// occurrence is Ready: their edges already resolve through that row.
    /// Superseded rows up to `still_claimed` kept a claim and are passed over.
    fn reclaimable_row(
        &self,
        still_claimed: Option<&WorthQueryOutputDemandKey>,
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
            } else if behind_ready.is_none()
                && closed_superseded(record)
                && still_claimed.is_none_or(|claimed| key > claimed)
            {
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

pub(super) fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "unheld custody reclamation exceeds request work",
    )
}
