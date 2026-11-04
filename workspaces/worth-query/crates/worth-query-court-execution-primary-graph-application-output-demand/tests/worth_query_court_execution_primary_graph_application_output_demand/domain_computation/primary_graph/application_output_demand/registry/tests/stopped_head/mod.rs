//! Every stopped-checkpoint finish releases its exact head hold.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputAdvancement;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission as Admission;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::published_retirement::admit;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::published_retirement::publish;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;

#[test]
fn stopped_head_is_released_by_every_finish_route() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let _occurrence = occurrence();
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|_| {
        for route in 0..3 {
            let registry = WorthQueryOutputDemandRegistry::default();
            let row = key("stopped-head", 1, 1);
            let owner = admit(&registry, &row);
            let peer = admit(&registry, &row);
            publish(&registry, &owner, receipt.clone());
            assert_eq!(
                (*(*registry
                    .state_for_test())
                    .lock()
                    .unwrap()
                    .records_for_test())
                    .publication_count_for_test(),
                1
            );
            let stop = WorthQueryOutputDemandDenial::new_for_test(
                WorthQueryOutputDemandDenialKind::ProductDelivery(Box::new(
                    worth_query_execution::facade::product::WorthQueryPerformedRelationalProductChangeDeliveryDenialKind::ForeignProductRoot)), "");
            match route {
                0 => {
                    let mut state = (*registry.state_for_test()).lock().unwrap();
                    let record = (*state.records_mut_for_test()).get_mut(&row).unwrap();
                    let DemandState::Output(output) = &mut (*record.state_mut_for_test())
                    else {
                        panic!("published row")
                    };
                    output.stop_for_test(stop.clone());
                }
                1 => {
                    let Admission::AdvanceCheckpoint { claim, checkpoint } = registry.begin_for_test(&owner)
                    else {
                        panic!("Published is claimed")
                    };
                    registry
                        .finish_checkpoint_for_test(&owner, claim, checkpoint, Some(&stop))
                        .unwrap();
                }
                2 => {
                    let (finish, claim, checkpoint) = registry
                        .prepare_selected_checkpoint_finish_for_test(&owner)
                        .unwrap()
                        .claim_for_test()
                        .unwrap()
                        .unwrap();
                    assert!(finish.finish_for_test(claim, checkpoint, Some(&stop)).is_ok());
                }
                _ => unreachable!(),
            }
            let state = (*registry.state_for_test()).lock().unwrap();
            let count = (*state.records_for_test()).publication_count_for_test();
            let DemandState::Output(output) = &(*(*state.records_for_test())[&row].state_for_test()) else {
                panic!("checkpoint retained")
            };
            assert!(matches!(
                output.advancement_for_test(),
                WorthQueryOutputAdvancement::Stopped { .. }
            ));
            assert_eq!(
                count, 0,
                "route {route}: a stopped row cannot freeze its head"
            );
            drop(state);
            assert!(
                matches!(registry.begin_for_test(&peer), Admission::Failed(ref denial) if denial.kind() == stop.kind())
            );
        }
    });
}
