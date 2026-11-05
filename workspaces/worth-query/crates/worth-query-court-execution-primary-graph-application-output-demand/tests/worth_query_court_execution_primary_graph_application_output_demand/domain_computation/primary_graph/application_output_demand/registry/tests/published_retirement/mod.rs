//! Interest closure retires undelivered output and its prerequisite claims.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::PreparedReadyBacking;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::lifecycle::selected_execution_finish::RunningSelectedExecution;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission as Admission;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryPendingOutputDelivery;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;

pub fn admit(
    registry: &WorthQueryOutputDemandRegistry,
    key: &WorthQueryOutputDemandKey,
) -> WorthQueryOutputDemandInterest {
    registry.admit_for_test(key.clone(), None, worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(root(1)), occurrence(), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Ordinary, None, None).map(|(interest, _pause)| interest).unwrap()
}

pub fn publish(
    registry: &WorthQueryOutputDemandRegistry,
    interest: &WorthQueryOutputDemandInterest,
    receipt: worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt,
) {
    assert!(matches!(
        registry.begin_for_test(interest),
        Admission::Schedule(None)
    ));
    registry.finish_scheduling_for_test(
        interest,
        None,
        &mut Ok(WorthQueryOutputSchedulingResult::Scheduled),
    );
    let finish = registry
        .prepare_selected_execution_finish_for_test(interest)
        .unwrap();
    assert!(matches!(
        registry.begin_for_test(interest),
        Admission::Execute { .. }
    ));
    let backing = PreparedReadyBacking::new_for_test();
    RunningSelectedExecution::new(finish)
        .publish_for_test(WorthQueryOutputCheckpoint::Published {
            receipt,
            delivery: WorthQueryPendingOutputDelivery::NoChange,
            ready_backing: backing,
        })
        .unwrap_or_else(|_| panic!("the real prepared finish publishes"));
}

#[test]
fn last_published_interest_releases_its_own_prerequisite_claims() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let _occurrence = occurrence();
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|_| {
        let registry = WorthQueryOutputDemandRegistry::default();
        let upstream_key = key("held-upstream", 1, 1);
        let upstream = admit(&registry, &upstream_key);
        let output_key = key("undelivered", 1, 2);
        let output = admit(&registry, &output_key);
        publish(&registry, &output, receipt);
        {
            // The local registry fixture declares one already funded consumed edge.
            // Native certification is not under test; release of that edge is.
            let mut state = (*registry.state_for_test()).lock().unwrap();
            let claims = vec![Arc::new(upstream_key.clone())];

            (*(*state.records_mut_for_test()).get_mut(&output_key).unwrap().prerequisites_mut_for_test()) = claims;
            (*(*state
                .records_mut_for_test())
                .get_mut(&upstream_key)
                .unwrap()
                .framework_required_count_mut_for_test()) += 1;
        }
        drop(output);
        let state = (*registry.state_for_test()).lock().unwrap();
        assert!(!(*state.records_for_test()).contains_key_for_test(&output_key));
        assert_eq!((*(*state.records_for_test())[&upstream_key].framework_required_count_for_test()), 0);
        assert_eq!((*(*state.records_for_test())[&upstream_key].interests_for_test()), 1);
        drop(state);
        drop(upstream);
    });
}

#[test]
fn one_remaining_interest_keeps_the_undelivered_publication() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let _occurrence = occurrence();
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|_| {
        let registry = WorthQueryOutputDemandRegistry::default();
        let upstream_key = key("remaining-interest-upstream", 1, 1);
        let upstream = admit(&registry, &upstream_key);
        let output_key = key("two-interests", 1, 1);
        let first = admit(&registry, &output_key);
        let second = admit(&registry, &output_key);
        // The admitted rows already fund their index; the extra custody is
        // one prerequisite claim and one ready backing declared below.
        publish(&registry, &first, receipt);
        let claims = vec![Arc::new(upstream_key.clone())];
        {
            let mut state = (*registry.state_for_test()).lock().unwrap();

            (*(*state.records_mut_for_test()).get_mut(&output_key).unwrap().prerequisites_mut_for_test()) = claims.clone();
            (*(*state
                .records_mut_for_test())
                .get_mut(&upstream_key)
                .unwrap()
                .framework_required_count_mut_for_test()) += 1;
        }
        drop(first);
        {
            let state = (*registry.state_for_test()).lock().unwrap();
            assert_eq!((*(*state.records_for_test())[&output_key].interests_for_test()), 1);
            assert_eq!(
                (*(*state.records_for_test())[&output_key].prerequisites_for_test()), claims,
                "the remaining interest still owns the exact prerequisite claim"
            );
            assert_eq!((*(*state.records_for_test())[&upstream_key].framework_required_count_for_test()), 1);

            assert!(
                matches!(&(*(*state.records_for_test())[&output_key].state_for_test()), DemandState::Output(output)
                if matches!( *output.checkpoint_for_test(), Some(WorthQueryOutputCheckpoint::Published { .. })))
            );
        }
        drop(second);
        assert!(!(*(*registry
            .state_for_test())
            .lock()
            .unwrap()
            .records_for_test())
            .contains_key_for_test(&output_key));
        assert_eq!(
            (*(*(*registry.state_for_test()).lock().unwrap().records_for_test())[&upstream_key].framework_required_count_for_test()),
            0
        );
        drop(upstream);
    });
}

#[test]
fn a_registry_successor_is_moved_to_exactly_one_taker() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let _occurrence = occurrence();
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|_| {
        let registry = WorthQueryOutputDemandRegistry::default();
        let row_key = key("single-successor", 1, 2);
        let first = admit(&registry, &row_key);
        let second = admit(&registry, &row_key);
        publish(&registry, &first, receipt);
        let successor: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::held_successor::HeldRequiredSuccessor = Box::new(row_key.clone());
        assert!(registry
            .hold_required_successor_for_test(successor, |held| held
                .downcast_ref::<WorthQueryOutputDemandKey>())
            .is_none());
        let moved = registry
            .take_published_held_successor_for_test(&row_key)
            .unwrap()
            .unwrap();
        assert_eq!(
            moved.downcast_ref::<WorthQueryOutputDemandKey>(),
            Some(&row_key)
        );
        assert!(
            registry
                .take_published_held_successor_for_test(&row_key)
                .unwrap()
                .is_none(),
            "a second advance receives no successor after the custody move"
        );
        drop((moved, first, second));
    });
}
