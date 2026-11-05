//! Current-basis selection frees a performed observation no caller holds.
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::currentness::select_current_product;
use worth_query_execution::facade::domain_computation::WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::source_custody::PreparedOutputRootKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryPerformedOutputDemandSource;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;

#[test]
fn current_commit_basis_reclaims_an_abandoned_performed_observation() {
    let world = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let principal = world
        .application
        .with_host_advancement(|host_phase| {
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal(&host_phase.test_host_request_for_test(), &world, &request)
        })
        .expect("installed host call opens");
    let account = world
        .application
        .with_host_advancement(|host_phase| {
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account(&host_phase.test_host_request_for_test(), &world, "open", &request)
        })
        .expect("installed host call opens");
    let program = world
        .application
        .with_host_advancement(|host_phase| {
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::retained_status_program(
                &host_phase.test_host_request_for_test(),
                &world,
                &principal,
                &account,
                &request,
                "observation-release",
                worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetentionMutationBreadth::Narrow,
            )
            .with_output_demand_observation()
        })
        .expect("installed host call opens");
    let WorthQueryApplicationCommitOutcome::Committed(mut receipt) =
        world.application.compare_and_commit_application(
            program,
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency(241, 242),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the real performed write must commit");
    };
    let occurrence = receipt.product_branch().occurrence_for_test();
    let change = std::sync::Arc::new(receipt.take_performed_relational_product_change().unwrap());
    let observation = receipt
        .committed_product_publication()
        .take_output_demand_observation_for_test()
        .unwrap();
    let retained_receipt = receipt.clone();
    let registry = &(*(*world.application.runtime_for_test()).output_demands_for_test());
    let preparation = registry.begin_source_preparation_for_test(occurrence);
    let kind = PreparedOutputRootKind::Required(TypeId::of_for_test::<()>());
    let commit = registry
        .retain_performed_source_for_test(
            WorthQueryPerformedOutputDemandSource::new_for_test(receipt, change, observation, None),
            &preparation,
            kind.clone(),
            None,
        )
        .unwrap();
    registry.release_prepared_token_for_test(&commit);
    drop(preparation);
    let runtime = world.application.product_runtime();
    let retained = runtime
        .admit_product_occurrence_for_test(occurrence)
        .unwrap();
    let mut held = Vec::new();
    loop {
        match runtime.admit_product_occurrence_for_test(occurrence) {
            Ok(product) => held.push(product),
            Err(worth_query_execution::facade::domain_computation::WorthQueryProductBranchAdmissionDenial::Refused(
                AdmissionRefusal::ObservationCapacityExhausted,
            )) => break,
            Err(denial) => panic!("unexpected product refusal: {denial:?}"),
        }
    }
    registry
        .validate_recovery_root_kind_for_test(&retained_receipt, kind.clone())
        .unwrap();
    world.application.with_application_advancement(&request, |_phase| {
        let current = select_current_product(&world.application, &retained)
            .expect("current commit selection reclaims the abandoned observation");
        assert_eq!(current.selected_commit(), retained.selected_commit());
        assert_eq!(registry.validate_recovery_root_kind_for_test(&retained_receipt, kind).unwrap_err().kind(),
            worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable);
        assert!(matches!(runtime.admit_product_occurrence_for_test(occurrence),
            Err(worth_query_execution::facade::domain_computation::WorthQueryProductBranchAdmissionDenial::Refused(AdmissionRefusal::ObservationCapacityExhausted))),
            "the reclaimed slot funds the current basis while other observers stay held");
    }).unwrap();
}
