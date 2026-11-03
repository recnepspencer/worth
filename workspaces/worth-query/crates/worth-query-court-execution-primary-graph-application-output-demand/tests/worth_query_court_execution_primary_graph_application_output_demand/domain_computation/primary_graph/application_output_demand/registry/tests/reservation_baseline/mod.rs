//! An unrelated retained publication remains present through neighboring cleanup.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::PerformedOutputObligation;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::PreparedReadyBacking;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryPendingOutputDelivery;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;

pub fn install(
    state: &mut DemandRegistryState,
    receipt: worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt,
) -> WorthQueryOutputDemandKey {
    let key = key("refund-baseline", 1, 7);
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let backing = PreparedReadyBacking::new();
    let output = WorthQueryOutputProgress::new(WorthQueryOutputCheckpoint::Published {
        receipt,
        delivery: WorthQueryPendingOutputDelivery::NoChange,
        ready_backing: backing,
    });
    // The receipt is real; this local lifetime fixture declares an interested
    // checkpoint and its already funded performed obligation directly.
    let mut row = record(occurrence(), DemandState::Output(output), 1);
    row.performed_obligations.push(PerformedOutputObligation {
        source_commit: commit,
        source: key.source.clone(),
    });
    state.records.insert(key.clone(), row);
    let member = state.prepare_required_member(&key).unwrap();
    state.install_required_member(member);
    key
}
