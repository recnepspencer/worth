//! Fixtures move an already-owned edge into a real prepared ticket for cancellation.

use std::sync::Arc;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandState;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandInterest;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::settlement_identity::RecordedSettlementIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryInvalidationResources;

pub fn unhold(
    interest: &WorthQueryOutputDemandInterest,
    cancel: bool,
    identity: &Arc<RecordedSettlementIdentity>,
) {
    let owner =
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner::new(
            WorthQueryInvalidationResources::new(),
            1,
        );(*(*(*
    (*interest
        .owner_for_test())
        .state_for_test())
        .lock()
        .unwrap()
        .records_mut_for_test())
        .get_mut(&(*interest.key_for_test()))
        .unwrap()
        .state_mut_for_test()) = DemandState::Running;

    let context = interest
        .required_context_for_test("registry-fixture-family", &owner)
        .unwrap();
    let mut claims = context
        .prepare_prerequisites_for_test(std::iter::empty(), &owner)
        .unwrap();
    if cancel {
        // The admitted edge's vector and credit change owners, not capacity.
        // This fixture models a claim still held by its unpublished ticket.
        let mut state =(* (*interest.owner_for_test()).state_for_test()).lock().unwrap();
        (*claims.claim_batch_mut_for_test()) =(* (*state
            .records_mut_for_test())
            .get_mut(&(*interest.key_for_test()))
            .unwrap()
            .claim_batch_mut_for_test())
            .take();
        if let Some(batch) = &(*claims.claim_batch_for_test()) {
            batch.deactivate_for_test();
        }
        (*claims.predecessors_mut_for_test()) =
            std::mem::take(&mut(* (*state.records_mut_for_test()).get_mut(&(*interest.key_for_test())).unwrap().prerequisites_mut_for_test()));
        drop(state);
        drop(claims);
    } else {
        claims.reserve_identity_for_test(identity).unwrap();
        let _published = claims.publish_for_test(Arc::clone(identity));
    }
}
