//! Reader projection of claims whose physical transfer is still noted.
mod incoming;
mod visibility;
use super::DemandRecord;
use super::DemandRegistryState;
use super::WorthQueryOutputDemandKey;
pub(super) use incoming::ClaimBatch;
pub(super) use incoming::IncomingClaims;
use std::sync::Arc;
use std::sync::Mutex;
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
    ) -> Result<Option<ClaimBatch>, super::WorthQueryOutputDemandDenial> {
        if upstream.is_empty() {
            return Ok(None);
        }
        let owner = self.records[owner].closed_cleanup.node();
        let targets = upstream
            .iter()
            .map(|key| self.records[key].closed_cleanup.node());
        Ok(Some(ClaimBatch::prepare(&owner, targets)))
    }
}

pub(super) type OwnerState = Mutex<ClaimOwnerState>;
