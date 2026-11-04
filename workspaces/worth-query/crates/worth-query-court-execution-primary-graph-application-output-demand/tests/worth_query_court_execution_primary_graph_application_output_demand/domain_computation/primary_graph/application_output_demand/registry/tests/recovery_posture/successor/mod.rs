use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::ReadyCompletion;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryCompletedOutputDemand;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::recovery_posture::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputAdvancement;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::refresh_predecessor::OutputRefreshPredecessor;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryPerformedOutputDemandSource;

#[test]
fn stale_ready_successor_admission_forces_a_new_execution_cycle() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let occurrence = receipt.product_branch().occurrence_for_test();
    let registry = WorthQueryOutputDemandRegistry::default();
    let demand_key = key("preserve", 8, 1);
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(root(1));
    let wake = Arc::new(DemandWake {
        generation: Mutex::new(0),
        changed: Condvar::new(),
    });
    (*(*registry.state_for_test()).lock().unwrap().records_mut_for_test()).insert_for_test(
        demand_key.clone(),
        DemandRecord {


            source_scope: Some(scope),
            wake: Arc::clone(&wake),
            ..record(
                occurrence,
                DemandState::Output(WorthQueryOutputProgress::new_for_test(
                    WorthQueryOutputCheckpoint::Ready(ReadyCompletion::for_test(WorthQueryCompletedOutputDemand {
                        authority: WorthQueryAcceptedOutputAuthority::Committed(receipt.clone()),
                        readiness: worth_query_execution::facade::domain_computation::WorthQueryOutputReadinessDeliveryEvidence::for_test_for_test(),
                    })),
                )),
                1,
            )
        },
    );

    let replacement = registry
        .admit_for_test(
            demand_key.clone(),
            None,
            scope,
            occurrence,
            DemandAdmissionKind::Ordinary,
            None,
            Some(OutputRefreshPredecessor::Committed(&receipt)),
        )
        .expect("the exact stale ready receipt admits its forced successor")
        .0;
    assert_eq!((*replacement.key_for_test()), demand_key);
    let state = (*registry.state_for_test()).lock().unwrap();
    let record = &(*state.records_for_test())[&demand_key];
    assert!(matches!((*record.state_for_test()), DemandState::Admitted));
    assert_eq!(
        (*record
            .successor_of_for_test())
            .as_ref()
            .map(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::succession::Succession::predecessor_for_test),
        Some(*receipt.idempotency_binding().key_identity())
    );
    assert_eq!((*record.interests_for_test()), 2);
    drop(state);

    // A claimed refresh that stops without publishing returns the row to the
    // Ready it reopened, so the next wave certifies and claims it again. A
    // performed source attached during that refresh is newer than the
    // reopen and stays with the row.
    let mut performed = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application_with_output_demand_observation();
    let change = performed
        .take_performed_relational_product_change()
        .expect("the real performed application retains its product change");
    let observation = performed
        .committed_product_publication()
        .take_output_demand_observation_for_test()
        .expect("the real performed application retains its read basis");
    {
        let mut state = (*registry.state_for_test()).lock().unwrap();
        let mut record = (*state.records_mut_for_test()).get_mut(&demand_key).unwrap();
        (*record.state_mut_for_test()) = DemandState::Running;
        (*record.performed_source_mut_for_test()) = Some(WorthQueryPerformedOutputDemandSource {
            receipt: performed,
            change: Arc::new(change),
            observation,
            output_source_identity: None,
        });
    }
    registry.relinquish_execution(&replacement);
    let state = (*registry.state_for_test()).lock().unwrap();
    let record = &(*state.records_for_test())[&demand_key];
    assert!(record.has_cached_ready_for_test());
    assert!((*record.successor_of_for_test()).is_none());
    assert!(
        (*record.performed_source_for_test()).is_some(),
        "the performed source attached during the refresh is kept"
    );
}

#[test]
fn stopped_ready_record_rejects_successor_without_reviving_custody() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let occurrence = receipt.product_branch().occurrence_for_test();
    let registry = WorthQueryOutputDemandRegistry::default();
    let demand_key = key("preserve", 8, 1);
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(root(1));
    let mut output = WorthQueryOutputProgress::new_for_test(WorthQueryOutputCheckpoint::Ready(
        ReadyCompletion::for_test(WorthQueryCompletedOutputDemand {
            authority: WorthQueryAcceptedOutputAuthority::Committed(receipt.clone()),
            readiness: worth_query_execution::facade::domain_computation::WorthQueryOutputReadinessDeliveryEvidence::for_test_for_test(),
        }),
    ));
    output.stop_for_test(WorthQueryOutputDemandDenial::new_for_test(
        WorthQueryOutputDemandDenialKind::Superseded,
        "newer source owns the cycle",
    ));
    (*(*registry.state_for_test()).lock().unwrap().records_mut_for_test()).insert_for_test(
        demand_key.clone(),
        DemandRecord {
            source_scope: Some(scope),
            ..record(occurrence, DemandState::Output(output), 1)
        },
    );

    let denial = match registry.admit_for_test(
        demand_key.clone(),
        None,
        scope,
        occurrence,
        DemandAdmissionKind::Ordinary,
        None,
        Some(OutputRefreshPredecessor::Committed(&receipt)),
    ) {
        Ok(_) => panic!("a stopped ready record was revived"),
        Err(denial) => denial,
    };
    assert_eq!(denial.kind(), WorthQueryOutputDemandDenialKind::Superseded);
    let state = (*registry.state_for_test()).lock().unwrap();
    let record = &(*state.records_for_test())[&demand_key];
    assert_eq!((*record.interests_for_test()), 1);
    assert!(matches!(
        &(*record.state_for_test()),
        DemandState::Output(output)
            if matches!(output.advancement_for_test(), WorthQueryOutputAdvancement::Stopped { .. })
                && matches!((*output.checkpoint_for_test()), Some(WorthQueryOutputCheckpoint::Ready(_)))
    ));
}

#[test]
fn failed_successor_admission_preserves_ready_custody() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let occurrence = receipt.product_branch().occurrence_for_test();
    let registry = WorthQueryOutputDemandRegistry::default();
    let ready_key = key_with_identity("preserve", 8, 1, 80);
    let stale_request = key_with_identity("preserve", 7, 1, 70);
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(root(1));
    (*(*registry.state_for_test()).lock().unwrap().records_mut_for_test()).insert_for_test(
        ready_key.clone(),
        DemandRecord {


            source_scope: Some(scope),
            ..record(
                occurrence,
                DemandState::Output(WorthQueryOutputProgress::new_for_test(
                    WorthQueryOutputCheckpoint::Ready(ReadyCompletion::for_test(WorthQueryCompletedOutputDemand {
                        authority: WorthQueryAcceptedOutputAuthority::Committed(receipt.clone()),
                        readiness: worth_query_execution::facade::domain_computation::WorthQueryOutputReadinessDeliveryEvidence::for_test_for_test(),
                    })),
                )),
                1,
            )
        },
    );

    let denial = match registry.admit_for_test(
        stale_request,
        None,
        scope,
        occurrence,
        DemandAdmissionKind::Ordinary,
        None,
        Some(OutputRefreshPredecessor::Committed(&receipt)),
    ) {
        Ok(_) => panic!("an older successor admission was accepted"),
        Err(denial) => denial,
    };
    assert_eq!(denial.kind(), WorthQueryOutputDemandDenialKind::Superseded);
    let state = (*registry.state_for_test()).lock().unwrap();
    let record = &(*state.records_for_test())[&ready_key];
    assert_eq!((*record.interests_for_test()), 1);
    assert!(matches!(
        &(*record.state_for_test()),
        DemandState::Output(output)
            if matches!((*output.checkpoint_for_test()), Some(WorthQueryOutputCheckpoint::Ready(_)))
    ));
}
