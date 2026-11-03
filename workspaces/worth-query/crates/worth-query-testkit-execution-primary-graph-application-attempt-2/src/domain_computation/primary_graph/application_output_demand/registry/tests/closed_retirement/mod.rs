use std::sync::Arc;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandRecord;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandState;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandInterest;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::demand_key::WorthQueryOutputDemandKey;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::interest;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::key_with_identity;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::occurrence;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::record;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::retain_owned_key;

use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint;
pub fn ready() -> DemandState {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let completion = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryCompletedOutputDemand {
        authority: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryAcceptedOutputAuthority::Committed(receipt),
        readiness: worth_query_execution::facade::domain_computation::WorthQueryOutputReadinessDeliveryEvidence::for_test(),
    };
    DemandState::Output(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress::new(
        WorthQueryOutputCheckpoint::Ready(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::ReadyCompletion::for_test(completion)),
    ))
}
pub fn claim_on_superseded(
    newest_state: DemandState,
) -> (
    WorthQueryOutputDemandRegistry,
    WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandKey,
    WorthQueryOutputDemandKey,
) {
    let registry = WorthQueryOutputDemandRegistry::default();
    let old = key_with_identity("upstream", 5, 1, 90);
    let newest = key_with_identity("upstream", 6, 1, 91);
    let dependent = key_with_identity("dependent", 5, 2, 50);
    let dependent_record = DemandRecord {
        prerequisites: vec![Arc::new(old.clone())],
        ..record(occurrence(), ready(), 1)
    };
    let wake = Arc::clone(&dependent_record.wake);
    {
        let mut state = registry.state.lock().unwrap();
        state.records.insert(
            old.clone(),
            DemandRecord {
                framework_required_count: 1,
                ..record(
                    occurrence(),
                    DemandState::Failed(WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::Superseded,
                        "",
                    )),
                    0,
                )
            },
        );
        state
            .records
            .insert(newest.clone(), record(occurrence(), newest_state, 1));
        state.required_keys.insert(Arc::new(newest.clone()));
        state.records.insert(dependent.clone(), dependent_record);
        retain_owned_key(&mut state, &dependent);
    }
    let interest = interest(&registry, dependent, wake);
    (registry, interest, old, newest)
}
pub fn claims(record: &DemandRecord) -> Vec<WorthQueryOutputDemandKey> {
    record
        .prerequisites
        .iter()
        .map(|claimed| claimed.as_ref().clone())
        .collect()
}
