use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::application_attempt::compare_and_commit::commit_receipt::certification::CertificationWorthQueryApplicationCommitReceiptForTest;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::BoundOutputSource;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::source_custody::PreparedOutputRootKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryPerformedOutputDemandSource;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;

#[test]
fn an_open_ordinary_interest_is_required_until_its_last_handle_closes() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let output = key("ordinary-interest", 1, 1);
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(root(1));
    let admit = || {
        registry
            .admit_for_test(
                output.clone(),
                None,
                scope,
                occurrence(),
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Ordinary,
                None,
                None,
            )
            .unwrap()
            .0
    };
    let first = admit();
    let second = admit();
    drop(first);
    assert!((*(*registry
        .state_for_test())
        .lock()
        .unwrap()
        .required_keys_for_test())
        .contains(&output));
    drop(second);
    let state = (*registry.state_for_test()).lock().unwrap();
    assert!(!(*state.records_for_test()).contains_key_for_test(&output));
    assert!((*state.required_keys_for_test()).is_empty());
}

#[test]
fn closing_a_required_interest_removes_it_from_the_required_set() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let output = key("required-interest", 1, 1);
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(root(1));
    let first = registry
        .admit_for_test(
            output.clone(),
            None,
            scope,
            occurrence(),
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Required,
            None,
            None,
        )
        .expect("a source-backed required interest admits")
        .0;
    assert!((*(*registry.state_for_test()).lock().unwrap().records_for_test())[&output].is_required_for_test());
    assert!((*(*registry
        .state_for_test())
        .lock()
        .unwrap()
        .required_keys_for_test())
        .contains(&output));
    drop(first);
    {
        let state = (*registry.state_for_test()).lock().unwrap();
        assert!(!(*state.records_for_test()).contains_key_for_test(&output));
        assert!((*state.required_keys_for_test()).is_empty());
    }

    let reopened = registry
        .admit_for_test(
            output.clone(),
            None,
            scope,
            occurrence(),
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Required,
            None,
            None,
        )
        .expect("closing an interest permits exact reopening")
        .0;
    assert!((*(*registry.state_for_test()).lock().unwrap().records_for_test())[&output].is_required_for_test());
    assert!((*(*registry
        .state_for_test())
        .lock()
        .unwrap()
        .required_keys_for_test())
        .contains(&output));
    drop(reopened);
    assert!(!(*(*registry.state_for_test()).lock().unwrap().records_for_test()).contains_key_for_test(&output));
}

#[test]
fn performed_source_obligation_survives_handle_close_until_occurrence_retirement() {
    let mut receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application_with_output_demand_observation();
    let occurrence = receipt.product_branch().occurrence_for_test();
    let scope = receipt.principal_scope().scope();
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let change = receipt
        .take_performed_relational_product_change()
        .expect("the real performed application retains its product change");
    let observation = receipt
        .committed_product_publication()
        .take_output_demand_observation_for_test()
        .expect("the real performed application retains its read basis");
    let registry = WorthQueryOutputDemandRegistry::default();
    let preparation = registry.begin_source_preparation_for_test(occurrence);
    registry
        .retain_performed_source_for_test(
            WorthQueryPerformedOutputDemandSource {
                receipt,
                change: Arc::new(change),
                observation,
                output_source_identity: None,
            },
            &preparation,
            PreparedOutputRootKind::Required(TypeId::of_for_test::<()>()),
            None,
        )
        .expect("the performed source enters native custody");
    let output = key("performed-required", 1, 1);
    registry
        .ensure_prepared_output_source_bound_for_test(
            &commit,
            BoundOutputSource {
                scope,
                identity: (*output.source_for_test()).clone(),
            },
        )
        .expect("the exact output source is bound");
    let interest = registry
        .admit_performed_for_test(output.clone(), &commit, scope, occurrence)
        .expect("the bound performed source admits one output obligation");
    assert_eq!(
        (*(*(*registry.state_for_test()).lock().unwrap().records_for_test())[&output]
            .performed_obligations_for_test())
            .len(),
        1
    );
    drop(interest);
    {
        let state = (*registry.state_for_test()).lock().unwrap();
        let record = &(*state.records_for_test())[&output];
        assert_eq!((*record.required_interests_for_test()), 0);
        assert!(
            record.is_required_for_test(),
            "the performed operation still requires its output"
        );
        assert_eq!((*(*record.performed_obligations_for_test())[0].source_commit_for_test()), commit);
        assert!((*state.required_keys_for_test()).contains(&output));
    }
    registry.release_product_occurrence_for_test(occurrence);
    let state = (*registry.state_for_test()).lock().unwrap();
    assert!(!(*state.records_for_test()).contains_key_for_test(&output));

    assert!((*state.required_keys_for_test()).is_empty());
}

#[test]
fn settlement_releases_only_the_ready_authority_it_accepted() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let replacement = receipt
        .clone()
        .with_provider_runtime_instance_id_for_test(u64::MAX);
    let output = key("ready-replaced", 1, 1);
    let expected = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryAcceptedOutputAuthority::Committed(receipt);
    let actual = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryAcceptedOutputAuthority::Committed(replacement.clone());
    let completion = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryCompletedOutputDemand {
        authority: actual.clone(),
        readiness: worth_query_execution::facade::domain_computation::WorthQueryOutputReadinessDeliveryEvidence::for_test_for_test(),
    };
    let mut retained = record(
        occurrence(),
        DemandState::Output(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress::new_for_test(
            WorthQueryOutputCheckpoint::Ready(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::ReadyCompletion::for_test(completion)),
        )),
        1,
    );
    (*retained.performed_obligations_mut_for_test()) = vec![worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::PerformedOutputObligation {
        source_commit: replacement
            .committed_product_publication()
            .composite_commit()
            .clone(),
        source: (*output.source_for_test()).clone(),
    }];

    let wake = Arc::clone(&(*retained.wake_for_test()));
    let registry = WorthQueryOutputDemandRegistry::default();
    {
        let mut state = (*registry.state_for_test()).lock().unwrap();
        (*state.records_mut_for_test()).insert_for_test(output.clone(), retained);
    }
    let interest = interest(&registry, output.clone(), wake);
    let denial = registry
        .finish_settlement_for_test(&interest, &expected)
        .expect_err("a replaced Ready authority cannot discharge its performed source");
    assert_eq!(denial.kind(), WorthQueryOutputDemandDenialKind::Superseded);
    assert_eq!(
        (*(*(*registry.state_for_test()).lock().unwrap().records_for_test())[&output]
            .performed_obligations_for_test())
            .len(),
        1
    );
    registry
        .finish_settlement_for_test(&interest, &actual)
        .expect("the exact accepted Ready authority may settle its obligation");
    let state = (*registry.state_for_test()).lock().unwrap();
    assert!((*(*state.records_for_test())[&output].performed_obligations_for_test()).is_empty());
}
