//! Actual mutation walk sites stay local when another occurrence grows.

use crate::domain_computation::primary_graph::application_output_demand::registry::tests as registry_tests;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::key;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::occurrence;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::record;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::DemandState;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress;
use std::sync::Arc;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::DemandRegistryState;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::lifecycle::closed_retirement::held;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::lifecycle::closed_retirement::newest_ready;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
pub trait WalkBoundsDemandRegistryStateForTest: Sized {
    fn assert_walk_site(site: usize);
}
impl WalkBoundsDemandRegistryStateForTest for DemandRegistryState {
    fn assert_walk_site(site: usize) {
        let ready = crate::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready();
        let worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::DemandState::Output(output) = ready else { panic!("Ready fixture") };
        let Some(worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint::Ready(completion)) = output.checkpoint else { panic!("Ready fixture") };
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
                DemandState::Failed(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "",
                )),
                0,
            );
            prior.framework_required_count = 1;
            state.records.insert(old.clone(), prior);
            let ready = || {
                DemandState::Output(WorthQueryOutputProgress::new(
                    WorthQueryOutputCheckpoint::Ready(completion.clone()),
                ))
            };
            state
                .records
                .insert(newest.clone(), record(occurrence(), ready(), 0));
            let mut row = record(occurrence(), ready(), 0);
            row.prerequisites.push(Arc::new(old.clone()));

            state.records.insert(dependent.clone(), row);
            for selected in [&old, &newest] {
                let member = state.prepare_required_member(selected).unwrap();
                state.install_required_member(member);
            }
            for generation in 1..=neighbors {
                state.records.insert(
                    key("larger-other", generation, 3),
                    record(occurrence(), DemandState::Admitted, 0),
                );
            }
            if site == 6 {
                let mut stopped = ready();
                if let DemandState::Output(output) = &mut stopped {
                    output.stop(WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::Superseded,
                        "",
                    ));
                }
                state.records.get_mut(&old).unwrap().state = stopped;
                state.records.get_mut(&newest).unwrap().state =
                    DemandState::Failed(WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::Superseded,
                        "",
                    ));
            }
            crate::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::add_custody(&mut state, &receipts[..entries], neighbors);
            let before = worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::record_map::occurrence_rows_visited();
            match site {
                0 => {
                    // The dependent has moved its claim before retirement.
                    state
                        .records
                        .get_mut(&old)
                        .unwrap()
                        .framework_required_count = 0;
                    state
                        .records
                        .get_mut(&newest)
                        .unwrap()
                        .framework_required_count = 1;
                    state.records.get_mut(&dependent).unwrap().prerequisites[0] =
                        Arc::new(newest.clone());
                    state.retire_row(&old, &mut Vec::new());
                }
                1 => {
                    state.follow_newest(&dependent, &old);
                }
                2 => {
                    assert!(newest_ready(&state.records, &old));
                }
                3 => {
                    assert!(held(&state.records, &newest, &state.records[&newest]));
                }
                4 => {
                    state.release_record_prerequisites(&dependent);
                }
                5 => {
                    state.remove_required_member_if_released(&newest);
                }
                6 => {
                    assert!(state.revive_replaced_ready(&newest));
                }
                _ => panic!("unknown walk site"),
            };
            let visited = worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::record_map::occurrence_rows_visited() - before;
            assert!(visited > 0, "site {site} was not reached");
            tally += 1;
            contacts.push(visited);
        }
        assert_eq!(contacts[0], contacts[1], "walk site {site}");
        assert_eq!(tally, 2);
    }
}
