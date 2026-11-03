//! Fixtures move an already-owned edge into a real prepared ticket for cancellation.

use std::sync::Arc;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::DemandState;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandInterest;
use worth_query_execution::domain_computation::primary_graph::output_lineage::settlement_identity::RecordedSettlementIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryInvalidationResources;

pub fn unhold(
    interest: &WorthQueryOutputDemandInterest,
    cancel: bool,
    identity: &Arc<RecordedSettlementIdentity>,
) {
    let owner =
        worth_query_execution::domain_computation::primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner::new(
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
