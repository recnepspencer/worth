//! Fixtures move an already-owned edge into a real prepared ticket for cancellation.

use super::super::WorthQueryOutputDemandInterest;
use super::*;
use crate::domain_computation::execution_runtime::source_invalidation::WorthQueryInvalidationResources;

pub(in super::super) fn unhold(
    interest: &WorthQueryOutputDemandInterest,
    cancel: bool,
    identity: &Arc<RecordedSettlementIdentity>,
) {
    let owner =
        crate::domain_computation::primary_graph::output_lineage::SourceInvalidationOwner::new(
            WorthQueryInvalidationResources::new(),
            1,
        );
    interest
        .owner
        .state
        .lock()
        .unwrap()
        .records
        .get_mut(&interest.key)
        .unwrap()
        .state = DemandState::Running;

    let context = interest
        .required_context("registry-fixture-family", &owner)
        .unwrap();
    let mut claims = context
        .prepare_prerequisites(std::iter::empty(), &owner)
        .unwrap();
    if cancel {
        // The admitted edge's vector and credit change owners, not capacity.
        // This fixture models a claim still held by its unpublished ticket.
        let mut state = interest.owner.state.lock().unwrap();
        claims.claim_batch = state
            .records
            .get_mut(&interest.key)
            .unwrap()
            .claim_batch
            .take();
        if let Some(batch) = &claims.claim_batch {
            batch.deactivate();
        }
        claims.predecessors =
            std::mem::take(&mut state.records.get_mut(&interest.key).unwrap().prerequisites);
        drop(state);
        drop(claims);
    } else {
        claims.reserve_identity(identity).unwrap();
        let _published = claims.publish(Arc::clone(identity));
    }
}
