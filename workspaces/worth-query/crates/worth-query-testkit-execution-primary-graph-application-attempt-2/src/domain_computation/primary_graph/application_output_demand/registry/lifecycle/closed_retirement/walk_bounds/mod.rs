//! Actual mutation walk sites stay local when another occurrence grows.

use crate::domain_computation::primary_graph::application_output_demand::registry::tests as registry_tests;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::key;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::occurrence;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::record;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandState;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress;
use std::sync::Arc;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandRegistryState;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::lifecycle::closed_retirement::held;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::lifecycle::closed_retirement::newest_ready;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
pub trait WalkBoundsDemandRegistryStateForTest: Sized {
    fn assert_walk_site(site: usize);
}
impl WalkBoundsDemandRegistryStateForTest for DemandRegistryState {
    fn assert_walk_site(site: usize) {
        let ready = crate::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready();
        let worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandState::Output(output) = ready else { panic!("Ready fixture") };
        let Some(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint::Ready(completion)) = output.checkpoint_into_for_test() else { panic!("Ready fixture") };
        let receipts = crate::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::receipt_batch();
        let mut tally = 0;
        let mut contacts = Vec::new();
        for (neighbors, entries) in [(1, 1), (64, 4)] {
            let old = key("own", 1, 1);
            let newest = key("own", 2, 1);
            let dependent = key("dependent", 1, 2);
            let mut state = DemandRegistryState::default();
            let mut prior = record(
                occurrence(),
                DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "",
                )),
                0,
            );
            (*prior.framework_required_count_mut_for_test()) = 1;
            (*state.records_mut_for_test()).insert_for_test(old.clone(), prior);
            let ready = || {
                DemandState::Output(WorthQueryOutputProgress::new_for_test(
                    WorthQueryOutputCheckpoint::Ready(completion.clone()),
                ))
            };
            (*state
                .records_mut_for_test())
                .insert_for_test(newest.clone(), record(occurrence(), ready(), 0));
            let mut row = record(occurrence(), ready(), 0);
            (*row.prerequisites_mut_for_test()).push(Arc::new(old.clone()));

            (*state.records_mut_for_test()).insert_for_test(dependent.clone(), row);
            for selected in [&old, &newest] {
                let member = state.prepare_required_member_for_test(selected).unwrap();
                state.install_required_member_for_test(member);
            }
            for generation in 1..=neighbors {
                (*state.records_mut_for_test()).insert_for_test(
                    key("larger-other", generation, 3),
                    record(occurrence(), DemandState::Admitted, 0),
                );
            }
            if site == 6 {
                let mut stopped = ready();
                if let DemandState::Output(output) = &mut stopped {
                    output.stop_for_test(WorthQueryOutputDemandDenial::new_for_test(
                        WorthQueryOutputDemandDenialKind::Superseded,
                        "",
                    ));
                }
                (*(*state.records_mut_for_test()).get_mut(&old).unwrap().state_mut_for_test()) = stopped;
                (*(*state.records_mut_for_test()).get_mut(&newest).unwrap().state_mut_for_test()) =
                    DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
                        WorthQueryOutputDemandDenialKind::Superseded,
                        "",
                    ));
            }
            crate::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::add_custody(&mut state, &receipts[..entries], neighbors);
            let before = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::record_map::occurrence_rows_visited();
            match site {
                0 => {(*
                    // The dependent has moved its claim before retirement.
                    (*state
                        .records_mut_for_test())
                        .get_mut(&old)
                        .unwrap()
                        .framework_required_count_mut_for_test()) = 0;(*
                    (*state
                        .records_mut_for_test())
                        .get_mut(&newest)
                        .unwrap()
                        .framework_required_count_mut_for_test()) = 1;(*
                    (*state.records_mut_for_test()).get_mut(&dependent).unwrap().prerequisites_mut_for_test())[0] =
                        Arc::new(newest.clone());
                    state.retire_row_for_test(&old, &mut Vec::new());
                }
                1 => {
                    state.follow_newest_for_test(&dependent, &old);
                }
                2 => {
                    assert!(newest_ready(&(*state.records_for_test()), &old));
                }
                3 => {
                    assert!(held(&(*state.records_for_test()), &newest, &(*state.records_for_test())[&newest]));
                }
                4 => {
                    state.release_record_prerequisites_for_test(&dependent);
                }
                5 => {
                    state.remove_required_member_if_released_for_test(&newest);
                }
                6 => {
                    assert!(state.revive_replaced_ready_for_test(&newest));
                }
                _ => panic!("unknown walk site"),
            };
            let visited = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::record_map::occurrence_rows_visited() - before;
            assert!(visited > 0, "site {site} was not reached");
            tally += 1;
            contacts.push(visited);
        }
        assert_eq!(contacts[0], contacts[1], "walk site {site}");
        assert_eq!(tally, 2);
    }
}
