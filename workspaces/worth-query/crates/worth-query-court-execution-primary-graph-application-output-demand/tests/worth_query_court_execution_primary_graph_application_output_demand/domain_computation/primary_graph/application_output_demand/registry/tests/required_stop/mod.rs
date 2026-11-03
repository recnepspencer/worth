use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::DemandState;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::occurrence;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::record;

use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::record_map::DemandRecords;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
#[test]
fn only_a_stop_intrinsic_to_the_head_row_fails_its_dependents() {
    let head = key("required-stop-head", 1, 1);
    let failed = |kind| {
        let mut records = DemandRecords::new();
        records.insert(
            head.clone(),
            record(
                occurrence(),
                DemandState::Failed(WorthQueryOutputDemandDenial::new(kind, "")),
                0,
            ),
        );
        records
    };
    for kind in [
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        WorthQueryOutputDemandDenialKind::Cancelled,
        WorthQueryOutputDemandDenialKind::TimedOut,
    ] {
        assert!(
            worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::required_stop::head_stop(&failed(kind.clone()), &head).is_none(),
            "{kind:?} belongs to the advance that met it"
        );
    }
    let intrinsic = worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::required_stop::head_stop(
        &failed(WorthQueryOutputDemandDenialKind::ProducerUnavailable),
        &head,
    );
    assert_eq!(
        intrinsic.map(|denial| denial.kind()),
        Some(WorthQueryOutputDemandDenialKind::ProducerUnavailable)
    );

    let mut recorded = failed(WorthQueryOutputDemandDenialKind::Cancelled);
    recorded.get_mut(&head).unwrap().required_stop =
        Some(WorthQueryOutputDemandDenialKind::MissingApplicableProducer);
    assert_eq!(
        worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::required_stop::head_stop(&recorded, &head).map(|denial| denial.kind()),
        Some(WorthQueryOutputDemandDenialKind::MissingApplicableProducer)
    );
}
