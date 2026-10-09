//! Whether another demand holds custody a refused advance could wait for.
//!
//! A reservation refused custody can wait only for custody another demand's
//! advance or close moves. Rows outside the advancing demand's own chain
//! count when a demand is open on them, or when the registry keeps a
//! successor for them: closing or finishing frees what they hold. Rows of
//! the chain, its own row and the upstream rows it reads, are what its
//! advance needs held at once, so another demand open on one counts only
//! while that row is stale or still refreshing: that demand's next advance
//! lets the stale row go or finishes the refresh. A closed cached row counts
//! nowhere; reservations retire those themselves. When nothing counts, no
//! other demand changes the refusal, and the caller's advance decides its
//! stop: see the producer's `caller_custody`.

use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::super::refreshed_rejoin::{occurrence_rows, superseded};
use super::super::{
    DemandRecord, DemandRegistryState, DemandState, WorthQueryOutputAdvancement,
    WorthQueryOutputDemandKey, WorthQueryOutputDemandRegistry,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;

impl WorthQueryOutputDemandRegistry {
    /// Whether a demand other than the caller of `key` holds custody its
    /// next advance or close moves. The stop of a walk the request's work
    /// cannot pay for establishes nothing.
    pub(in crate::domain_computation::primary_graph) fn another_demand_holds_custody(
        &self,
        key: &WorthQueryOutputDemandKey,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, CompanionPreflightStop> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .another_demand_holds_custody(key, admission)
    }

    /// The same for a caller yet to hold a row: whether any demand does.
    pub(in crate::domain_computation::primary_graph) fn a_demand_holds_custody(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, CompanionPreflightStop> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        admission.charge_external_work(count(state.records.len())?)?;
        let held = state.records.values().any(holds_custody);
        Ok(held)
    }
}

impl DemandRegistryState {
    /// The caller of `key` holds one interest, on `key` itself.
    fn another_demand_holds_custody(
        &self,
        key: &WorthQueryOutputDemandKey,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, CompanionPreflightStop> {
        let rows = count(self.records.len())?;
        // The chain's occurrences, each named by one of its rows. Rows of one
        // occurrence are contiguous, so each is visited once.
        admission.charge_external_work(rows)?;
        let mut chain = vec![key.clone()];
        let mut next = 0;
        while let Some(occurrence) = chain.get(next).cloned() {
            next += 1;
            for (_, record) in occurrence_rows(&self.records, &occurrence) {
                for claimed in &record.prerequisites {
                    if !chain.iter().any(|known| known.same_occurrence(claimed)) {
                        chain.push(claimed.as_ref().clone());
                    }
                }
            }
        }
        let comparisons = rows
            .checked_mul(count(chain.len())?)
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        admission.charge_external_work(comparisons)?;
        Ok(self.records.iter().any(|(other, record)| {
            let open = record.interests > usize::from(other == key);
            if chain.iter().any(|known| known.same_occurrence(other)) {
                open && (superseded(record) || refreshing(record))
            } else {
                holds_custody(record)
            }
        }))
    }
}

/// A count of rows as work units.
fn count(rows: usize) -> Result<u64, CompanionPreflightStop> {
    u64::try_from(rows).map_err(|_| CompanionPreflightStop::WorkCounterOverflow)
}

/// A demand is open on the row, or the registry keeps a successor for it.
fn holds_custody(record: &DemandRecord) -> bool {
    record.interests != 0 || record.held_successor.is_some()
}

/// The row's work is still to finish: its owner's next advance moves it.
fn refreshing(record: &DemandRecord) -> bool {
    match &record.state {
        DemandState::Output(output) => {
            matches!(output.advancement, WorthQueryOutputAdvancement::Claimed(_))
        }
        DemandState::Failed(_) => false,
        _ => true,
    }
}
