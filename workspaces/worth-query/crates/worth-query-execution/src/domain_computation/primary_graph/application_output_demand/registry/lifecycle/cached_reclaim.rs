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

use std::sync::Arc;

use super::super::refreshed_rejoin::superseded;
use super::super::DemandRecord;
use super::super::DemandRegistryState;
use super::super::WorthQueryOutputDemandKey;
use super::super::WorthQueryOutputDemandRegistry;
use super::closed_retirement::claimants_where;
use super::closed_retirement::closed_cached;
use super::closed_retirement::closed_superseded;
use super::closed_retirement::held_by_row;
use super::closed_retirement::ready_at_newest;

use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

impl WorthQueryOutputDemandRegistry {
    /// Release one product observation nothing holds: retire a closed
    /// cached row, or with none left release an unheld performed source.
    /// Whether one was released.
    pub(in crate::domain_computation::primary_graph) fn reclaim_unheld_observation(
        &self,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let mut retired = Vec::new();
        let mut claims = Vec::new();
        let mut released = None;
        let reclaimed = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let reclaimed = state.retire_next_cached_row(&mut None, &mut retired, &mut claims);
            match reclaimed {
                Ok(false) => state.release_unheld_performed_source().map(|source| {
                    released = source;
                    released.is_some()
                }),
                reclaimed => reclaimed,
            }
        };
        drop(retired);
        drop(claims);
        drop(released);
        if reclaimed.is_ok() {
            self.drain_foreign_cleanup();
        }
        reclaimed
    }
}

impl DemandRegistryState {
    /// Retire the next reclaimable row. Whether one was left to retire.
    fn retire_next_cached_row(
        &mut self,
        still_claimed: &mut Option<WorthQueryOutputDemandKey>,
        retired: &mut Vec<DemandRecord>,
        claims: &mut Vec<Arc<WorthQueryOutputDemandKey>>,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        loop {
            let Some(key) = self.reclaimable_row(still_claimed.as_ref())? else {
                return Ok(false);
            };
            let dependents = claimants_where(&self.records, &key, |_| true);
            for dependent in dependents {
                self.follow_newest(&dependent, &key);
            }
            // A claim that did not move still resolves through this row: pass
            // over it and keep looking, so a refusal names no reclaimable row.
            let claimed = self.records.get(&key).is_some_and(|record| {
                let claimed = record.framework_required_count != 0;
                if !claimed {
                    crate::domain_computation::primary_graph::fact_verification_observation::registry_custody(
                        crate::domain_computation::primary_graph::WorthQueryRegistryCustodyTransitionForTest::CachedReclamation,
                        record.settlements.len(), record.prerequisites.len(),
                    );
                }
                claimed
            });
            if !claimed {
                claims.extend(self.retire_row(&key, retired));
                return Ok(true);
            }
            *still_claimed = Some(key);
        }
    }

    /// The first closed cached row nothing holds, or else the first closed
    /// superseded row only dependents' claims hold once the newest row of its
    /// occurrence is Ready: their edges already resolve through that row.
    /// Superseded rows up to `still_claimed` kept a claim and are passed over.
    fn reclaimable_row(
        &self,
        still_claimed: Option<&WorthQueryOutputDemandKey>,
    ) -> Result<Option<WorthQueryOutputDemandKey>, WorthQueryOutputDemandDenial> {
        // One pass over the rows; each candidate compares the rows of its
        // occurrence once more.

        let mut behind_ready = None;
        for (key, record) in &self.records {
            if closed_cached(record) && !superseded(record) {
                let prepaid = &self.records;
                if !held_by_row(record) && !crate::domain_computation::primary_graph::application_output_demand::registry::refreshed_rejoin::awaited_by_stale_owner(prepaid, key) {
                    return Ok(Some(key.clone()));
                }
            } else if behind_ready.is_none()
                && closed_superseded(record)
                && still_claimed.is_none_or(|claimed| key > claimed)
            {
                let prepaid = &self.records;
                if claimed_only(record)
                    && !crate::domain_computation::primary_graph::application_output_demand::registry::refreshed_rejoin::awaited_by_stale_owner(prepaid, key)
                    && ready_at_newest(&self.records, crate::domain_computation::primary_graph::application_output_demand::registry::refreshed_rejoin::newest_refresh(prepaid, key))
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
