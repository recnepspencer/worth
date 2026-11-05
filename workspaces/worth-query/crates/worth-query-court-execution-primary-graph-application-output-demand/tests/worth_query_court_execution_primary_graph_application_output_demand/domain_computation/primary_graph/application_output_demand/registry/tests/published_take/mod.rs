//! A previously Published observation grants no take after the row moves.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::row_stage::OutputRowStage;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission as Admission;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::published_retirement::admit;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::published_retirement::publish;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;

#[test]
fn a_take_checks_the_live_stage_under_its_own_lock() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let _occurrence = occurrence();
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|_| {
        for stopped in [false, true] {
            let registry = WorthQueryOutputDemandRegistry::default();
            let row = key("checked-take", 1, 1);
            let first = admit(&registry, &row);
            let _peer = admit(&registry, &row);
            publish(&registry, &first, receipt.clone());
            let held: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::held_successor::HeldRequiredSuccessor = Box::new(row.clone());
            assert!(registry
                .hold_required_successor_for_test(held, |held| held
                    .downcast_ref::<WorthQueryOutputDemandKey>())
                .is_none());
            assert!(matches!(
                registry.row_stage(&first),
                OutputRowStage::Published
            ));
            let restore = if stopped {
                let mut state = (*registry.state_for_test()).lock().unwrap();
                let mut record = (*state.records_mut_for_test()).get_mut(&row).unwrap();
                let DemandState::Output(output) = &mut (*record.state_mut_for_test())
                else {
                    panic!("published output")
                };
                output.stop_for_test(WorthQueryOutputDemandDenial::new_for_test(
                    WorthQueryOutputDemandDenialKind::Closed,
                    "",
                ));
                None
            } else {
                let Admission::AdvanceCheckpoint { claim, checkpoint } = registry.begin_for_test(&first)
                else {
                    panic!("the intervening call claims delivery")
                };
                Some((claim, checkpoint))
            };
            let count = (*(*registry
                .state_for_test())
                .lock()
                .unwrap()
                .records_for_test())
                .publication_count_for_test();
            assert!(
                registry
                    .take_published_held_successor_for_test(&row)
                    .unwrap()
                    .is_none(),
                "stopped={stopped}: the stale observation cannot move a successor"
            );
            assert!(registry
                .take_published_held_successor_for_test(&row)
                .unwrap()
                .is_none());
            let state = (*registry.state_for_test()).lock().unwrap();
            assert_eq!((*state.records_for_test()).publication_count_for_test(), count);
            assert!(
                (*(*state.records_for_test())[&row].held_successor_for_test()).is_some(),
                "a refused take preserves custody"
            );
            drop(state);
            if let Some((claim, checkpoint)) = restore {
                registry
                    .finish_checkpoint_for_test(&first, claim, checkpoint, None)
                    .unwrap();
                assert!(registry
                    .take_published_held_successor_for_test(&row)
                    .unwrap()
                    .is_some());
                assert!(registry
                    .take_published_held_successor_for_test(&row)
                    .unwrap()
                    .is_none());
                assert_eq!(
                    (*(*registry
                        .state_for_test())
                        .lock()
                        .unwrap()
                        .records_for_test())
                        .publication_count_for_test(),
                    count
                );
            }
        }
    });
}

#[test]
fn an_erased_successor_without_a_row_key_is_returned_without_effects() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let held = registry
        .hold_required_successor_for_test(Box::new(7_u32), |held| {
            held.downcast_ref::<WorthQueryOutputDemandKey>()
        })
        .expect("an invalid successor is returned to its owner");
    assert_eq!(*held.downcast::<u32>().unwrap(), 7);
    assert_eq!(
        (*(*registry.state_for_test())
            .lock()
            .unwrap()
            .records_for_test())
        .publication_count_for_test(),
        0
    );
}
