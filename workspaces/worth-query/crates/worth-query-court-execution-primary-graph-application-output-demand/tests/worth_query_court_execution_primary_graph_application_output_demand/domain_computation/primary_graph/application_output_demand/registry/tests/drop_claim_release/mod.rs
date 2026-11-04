//! Direct Drop release stays bounded and leaves unvisited debt noted.
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
pub fn stopped_ready() -> DemandState {
    let DemandState::Output(mut output) = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready() else {
        panic!("Ready")
    };
    output.stop_for_test(
        worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial::new_for_test(
            WorthQueryOutputDemandDenialKind::Superseded,
            "newer",
        ),
    );
    DemandState::Output(output)
}

#[test]
fn registry_drop_closed_claim_release_is_two_rows_without_a_registry_pass() {
    let mut contacts = Vec::new();
    for neighbors in [1, 64] {
        let registry = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry::default();
        let old: Vec<_> = (1..=5).map(|slot| key("quota-chain", 1, slot)).collect();
        let mut state = (*registry.state_for_test()).lock().unwrap();
        for (index, key) in old.iter().enumerate() {
            let mut row = record(occurrence(), stopped_ready(), usize::from(index == 0));
            (*row.framework_required_count_mut_for_test()) = usize::from(index != 0);
            if let Some(upstream) = old.get(index + 1) {
                (*row.prerequisites_mut_for_test()).push(Arc::new(upstream.clone()));
            }
            (*state.records_mut_for_test()).insert_for_test(key.clone(), row);
            let member = state.prepare_required_member_for_test(key).unwrap();
            state.install_required_member_for_test(member);
            let newest = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key("quota-chain", 2, index as u8 + 1);
            (*state.records_mut_for_test()).insert_for_test(
                newest.clone(),
                record(occurrence(), worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready(), 0),
            );
            let member = state.prepare_required_member_for_test(&newest).unwrap();
            state.install_required_member_for_test(member);
        }
        for slot in 0..neighbors {
            (*state.records_mut_for_test()).insert_for_test(
                key("neighbor", 1, 100 + slot),
                record(occurrence(), DemandState::Admitted, 1),
            );
        }
        retain_owned_key(&mut state, &old[0]);
        let wake = Arc::clone(&(*(*state.records_for_test())[&old[0]].wake_for_test()));
        drop(state);
        let whole = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::record_map::whole_rows().expect("bounded registry observation");
        let occurrence = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::record_map::occurrence_rows_visited();
        drop(interest(&registry, old[0].clone(), wake));
        contacts.push(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::record_map::occurrence_rows_visited() - occurrence);
        assert_eq!(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::record_map::whole_rows().expect("bounded registry observation"),
            whole
        );
        let state = (*registry.state_for_test()).lock().unwrap();
        for key in &old[..2] {
            assert!(!(*state.records_for_test())[key].has_cached_ready_for_test());
            assert!(
                (*(*state.records_for_test())[key].closed_cleanup_for_test()).queued_for_test()
            );
        }
        assert!((*state.records_for_test())[&old[2]].has_cached_ready_for_test());
        assert!(
            (*(*state.records_for_test())[&old[2]].closed_cleanup_for_test()).queued_for_test()
        );
        assert_eq!(
            (*(*state.records_for_test())[&old[2]].framework_required_count_for_test()),
            0
        );
    }
    assert_eq!(contacts[0], contacts[1]);
}
