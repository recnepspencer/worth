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

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint;
pub fn ready() -> DemandState {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let completion = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryCompletedOutputDemand::new_for_test(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryAcceptedOutputAuthority::Committed(receipt), worth_query_execution::facade::domain_computation::WorthQueryOutputReadinessDeliveryEvidence::for_test_for_test());
    DemandState::Output(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress::new_for_test(
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
    // A functional update over private fields: start from the base record and set the field.
    let dependent_record = {
        let mut base = record(occurrence(), ready(), 1);
        *base.prerequisites_mut_for_test() = vec![Arc::new(old.clone())];
        base
    };
    let wake = Arc::clone(dependent_record.wake_for_test());
    {
        let mut state = (*registry.state_for_test()).lock().unwrap();
        (*state.records_mut_for_test()).insert_for_test(old.clone(), {
            let mut base = record(
                occurrence(),
                DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "",
                )),
                0,
            );
            *base.framework_required_count_mut_for_test() = 1;
            base
        });
        (*state.records_mut_for_test())
            .insert_for_test(newest.clone(), record(occurrence(), newest_state, 1));
        (*state.required_keys_mut_for_test()).insert(Arc::new(newest.clone()));
        (*state.records_mut_for_test()).insert_for_test(dependent.clone(), dependent_record);
        retain_owned_key(&mut state, &dependent);
    }
    let interest = interest(&registry, dependent, wake);
    (registry, interest, old, newest)
}
pub fn claims(record: &DemandRecord) -> Vec<WorthQueryOutputDemandKey> {
    (*record.prerequisites_for_test())
        .iter()
        .map(|claimed| claimed.as_ref().clone())
        .collect()
}
