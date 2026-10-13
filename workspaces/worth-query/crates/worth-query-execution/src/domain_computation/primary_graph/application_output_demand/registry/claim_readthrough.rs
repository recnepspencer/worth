//! Reader projection of claims whose physical transfer is still noted.
mod incoming;
mod visibility;
use super::{DemandRecord, DemandRegistryState, WorthQueryOutputDemandKey};
pub(super) use incoming::{ClaimBatch, IncomingClaims};
use std::sync::{Arc, Mutex};
pub(super) use visibility::observe_row;

/// The record's mutation guard publishes this constant-sized owner state.
#[derive(Default, Clone, Copy)]
pub(super) struct ClaimOwnerState {
    pub live: bool,
    pub closed_cached: bool,
}
impl ClaimOwnerState {
    pub(super) fn from_record(row: &DemandRecord) -> Self {
        Self {
            live: true,
            closed_cached: row.interests == 0
                && row.required_interests == 0
                && row.has_cached_ready(),
        }
    }
}

impl DemandRegistryState {
    /// Admit the batch and every incoming link before publishing any link.
    /// Both sides use this same link, so a transfer needs no allocation.
    pub(super) fn prepare_claim_readthrough(
        &self,
        owner: &WorthQueryOutputDemandKey,
        upstream: &[Arc<WorthQueryOutputDemandKey>],
        admission: &mut crate::domain_computation::primary_graph::InvalidationEditAdmission,
    ) -> Result<Option<ClaimBatch>, super::WorthQueryOutputDemandDenial> {
        if upstream.is_empty() {
            return Ok(None);
        }
        let bytes = incoming::batch_bytes(upstream.len())
            .ok_or_else(super::record_capacity::capacity_denial)?;
        admission
            .admit_read_scratch(bytes as u64)
            .map_err(|_| super::record_capacity::capacity_denial())?;
        let work = upstream
            .len()
            .checked_mul(3)
            .and_then(|n| n.checked_add(1))
            .ok_or_else(super::refreshed_rejoin::work_denial)?;
        admission
            .charge_external_work(work as u64)
            .map_err(|_| super::refreshed_rejoin::work_denial())?;
        let capacity = Arc::new(self.reserve_record_capacity(bytes)?);
        let owner = self.records[owner].closed_cleanup.node();
        let targets = upstream
            .iter()
            .map(|key| self.records[key].closed_cleanup.node());
        Ok(Some(ClaimBatch::prepare(&owner, targets, capacity)))
    }
}

pub(super) type OwnerState = Mutex<ClaimOwnerState>;
