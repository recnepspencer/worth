//! Release at most two rows' owned edges, without claimant or registry discovery.
use super::super::super::refreshed_rejoin::superseded;
use super::super::super::DemandRegistryState;
use super::super::super::DemandState;
use super::super::super::WorthQueryOutputDemandKey;
use std::sync::Arc;

const CLOSED_CLAIM_RELEASE_SLOTS: usize = 2;

#[derive(Default)]
struct ReleasedRowCustody {
    ready: Option<super::super::super::WorthQueryOutputCheckpoint>,
    row: Option<super::super::super::DemandRecord>,
    prior: Vec<Arc<WorthQueryOutputDemandKey>>,
    batch: Option<super::super::super::claim_readthrough::ClaimBatch>,
    member: super::super::super::required_members::DetachedRequiredMember,
}

/// Stack slots are fixed; moved buffers keep their original admitted backing.
pub(super) struct ReleasedClosedCustody {
    rows: [ReleasedRowCustody; CLOSED_CLAIM_RELEASE_SLOTS],
}

impl DemandRegistryState {
    /// Only the first existing edge is followed beyond this row. Every other
    /// edge's terminal transition leaves its own owned note for the admission.
    pub(super) fn release_closed_claim_custody(
        &mut self,
        dependent: &WorthQueryOutputDemandKey,
    ) -> ReleasedClosedCustody {
        let mut released = ReleasedClosedCustody {
            rows: std::array::from_fn(|_| Default::default()),
        };
        let mut cursor: Option<Arc<WorthQueryOutputDemandKey>> = None;
        for slot in &mut released.rows {
            let key = cursor.as_deref().unwrap_or(dependent);
            let Some(row) = self.records.get(key) else {
                break;
            };
            if !super::super::closed_retirement::closed_cached(row) {
                break;
            }
            if !superseded(row) || row.framework_required_count != 0 {
                // A retained cached claimant can move its first closed leaf
                // claim without a cascade, using that leaf's own occurrence.
                let Some(upstream) = row.prerequisites.first().map(Arc::clone) else {
                    break;
                };
                if !self
                    .records
                    .get(upstream.as_ref())
                    .is_some_and(|old| old.has_cached_ready() && old.prerequisites.is_empty())
                    || !self.transfer_closed_claim(key, upstream.as_ref())
                {
                    break;
                }
                cursor = Some(upstream);
            }
            let key = cursor.as_deref().unwrap_or(dependent);
            let Some(row) = self.records.get(key) else {
                break;
            };
            if !superseded(row)
                || row.interests != 0
                || row.required_interests != 0
                || row.framework_required_count != 0
                || row.prepared_prerequisite_claims != 0
                || row.pending_cleanup_queued
                || !row.performed_obligations.is_empty()
                || row.performed_source.is_some()
                || row.held_successor.is_some()
                || !super::super::closed_retirement::newest_ready(&self.records, key)
            {
                break;
            }
            if !row.has_cached_ready() {
                break;
            }
            let mut row = self.records.get_mut(key).expect("selected row is retained");
            slot.batch = row.claim_batch.take();
            if let Some(batch) = &slot.batch {
                batch.deactivate();
            }
            slot.prior = std::mem::take(&mut row.prerequisites);
            let DemandState::Output(output) = &mut row.state else {
                break;
            };
            slot.ready = output.checkpoint.take();
            row.finish();

            for upstream in &slot.prior {
                self.records
                    .get_mut(upstream)
                    .expect("claimed row is retained")
                    .framework_required_count -= 1;
            }
            slot.member = self.remove_required_member_if_released_detached(key);

            if slot.prior.is_empty()
                && self
                    .records
                    .get(key)
                    .is_some_and(|row| row.settlements.is_empty())
            {
                slot.row = self.remove_record(key);
            }
            cursor = slot.prior.first().map(Arc::clone);
            if cursor.is_none() {
                break;
            }
        }
        released
    }
}
