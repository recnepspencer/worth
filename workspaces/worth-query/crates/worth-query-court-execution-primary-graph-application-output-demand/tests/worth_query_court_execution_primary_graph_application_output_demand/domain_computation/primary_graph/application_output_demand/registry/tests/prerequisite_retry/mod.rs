use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandRecoveryPosture;

#[test]
fn pending_required_prerequisite_preserves_its_retryable_operation() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let demand_key = key("dependent", 70, 1);
    let wake = Arc::new(DemandWake::new_for_test(Mutex::new(0), Condvar::new()));
    (*(*registry.state_for_test())
        .lock()
        .unwrap()
        .records_mut_for_test())
    .insert_for_test(demand_key.clone(), {
        let __required_interests = 1;
        let __wake = Arc::clone(&wake);
        let mut __base = record(occurrence(), DemandState::Running, 1);
        *__base.required_interests_mut_for_test() = __required_interests;
        *__base.wake_mut_for_test() = __wake;
        __base
    });
    let demand_interest = required_interest(&registry, demand_key.clone(), wake);
    let mut denial = WorthQueryOutputDemandDenial::new_for_test(
        WorthQueryOutputDemandDenialKind::IncompleteDependencyCoverage,
        "selected upstream settlement has not entered the managed registry",
    )
    .with_recovery_posture_for_test(WorthQueryOutputDemandRecoveryPosture::Retryable);

    registry.finish_execution_failure_for_test(&demand_interest, &mut denial);

    assert_eq!(
        denial.recovery_posture(),
        WorthQueryOutputDemandRecoveryPosture::Retryable
    );
    assert!(matches!(
        *(*(*registry.state_for_test())
            .lock()
            .unwrap()
            .records_for_test())[&demand_key]
            .state_for_test(),
        DemandState::Scheduled
    ));
}

/// An execution pinned before its upstream refreshed read the older row, and
/// the refresh retired it before the claim. The claim is stale and retryable:
/// the retry reads the current upstream row.
#[test]
fn an_upstream_retired_after_the_execution_read_it_is_a_retryable_stale_claim() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use worth_query_execution::facade::domain_computation::WorthQueryInvalidationResources;
    let registry = WorthQueryOutputDemandRegistry::default();
    let reader_key = key("reader", 71, 1);
    let wake = Arc::new(DemandWake::new_for_test(Mutex::new(0), Condvar::new()));
    (*(*registry.state_for_test())
        .lock()
        .unwrap()
        .records_mut_for_test())
    .insert_for_test(reader_key.clone(), {
        let __required_interests = 1;
        let __wake = Arc::clone(&wake);
        let mut __base = record(occurrence(), DemandState::Running, 1);
        *__base.required_interests_mut_for_test() = __required_interests;
        *__base.wake_mut_for_test() = __wake;
        __base
    });
    let reader = required_interest(&registry, reader_key, wake);
    let resources = WorthQueryInvalidationResources::new();
    let source_owner =
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner::new(
            resources, 1,
        );

    let context = reader
        .required_context_for_test("registry-fixture-family", Ok(()), &source_owner)
        .expect("the running reader issues its required context");
    // The upstream row this execution read is no longer indexed: a newer
    // settlement of its demand retired it.
    let (_lineage, retired) = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let selected = graph
        .integration_handle_for_test()
        .with_runtime(request_execution, |runtime| {
            let basis = runtime
                .admit_branch_basis(&runtime.main_branch_identity())
                .unwrap();
            let snapshot = runtime
                .snapshots()
                .snapshot_for_observation(&basis.observation())
                .unwrap();
            Arc::new(runtime.read_truth().positioned_snapshot(&snapshot).unwrap())
        });
    let consumed = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::ConsumedOutputEvidence::retained_for_test(retired, Arc::from([]), Vec::new(), None, selected);
    let stop = context
        .prepare_prerequisites_for_test(std::iter::once(&consumed), &source_owner)
        .err()
        .expect("a retired upstream cannot be claimed");
    assert_eq!(
        stop.kind(),
        WorthQueryOutputDemandDenialKind::PublicationStale
    );
    assert!(
        stop.clone().take_requested_output_for_test().is_none(),
        "a retired exact settlement cannot authorize managed readiness recovery"
    );
    assert_eq!(
        stop.recovery_posture(),
        WorthQueryOutputDemandRecoveryPosture::Retryable
    );
}

/// Model the publication boundary after DecisionReader has sealed an already
/// indexed upstream output. Directory policy comes from context issuance; the
/// registry must consult it before installing mandatory claims.
#[test]
fn prewarmed_output_cannot_be_promoted_without_durable_prerequisite_admission() {
    use worth_query_execution::facade::product::WorthQueryDurableStateAdmissionDenial as Durable;
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let registry = WorthQueryOutputDemandRegistry::default();
    let reader_key = key("ordinary-consumer", 71, 1);
    let upstream_key = Arc::new(key("prewarmed-upstream", 70, 2));
    let wake = Arc::new(DemandWake::new_for_test(Mutex::new(0), Condvar::new()));
    let (_lineage, identity) = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
    {
        let mut state = registry.state_for_test().lock().unwrap();
        let (posting, _cancellation) = state
            .reserve_settlement_vacancy_for_test(&identity)
            .unwrap();
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::settlement_index::SettlementIndex::fill_prepared_for_test(
            &posting, &identity, identity.clone(), upstream_key.clone(),
        );
        let mut upstream = record(identity.address_for_test().0,
            worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready(), 1);
        upstream.settlements_mut_for_test().push(identity.clone());
        state
            .records_mut_for_test()
            .insert_for_test(upstream_key.as_ref().clone(), upstream);
        let mut reader = record(occurrence(), DemandState::Running, 1);
        *reader.wake_mut_for_test() = wake.clone();
        state
            .records_mut_for_test()
            .insert_for_test(reader_key.clone(), reader);
        retain_owned_key(&mut state, &reader_key);
        retain_owned_key(&mut state, upstream_key.as_ref());
    }
    assert!(registry.posts_settlement_for_test(&identity).unwrap());
    let reader = interest(&registry, reader_key.clone(), wake);
    let source_owner = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner::new(
        worth_query_execution::facade::domain_computation::WorthQueryInvalidationResources::new(), 1,
    );
    let admission = Err(Durable::Unsupported {
        family: worth_foundational::facade::DurableStateFamily::OutputDemandRecords,
        supported_from_phase: 6,
    });
    // Ordinary publication with no consumed outputs does not promote custody.
    drop(
        reader
            .required_context_for_test("registry-fixture-family", admission, &source_owner)
            .unwrap()
            .prepare_prerequisites_for_test(std::iter::empty(), &source_owner)
            .unwrap(),
    );
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let selected = graph
        .integration_handle_for_test()
        .with_runtime(request_execution, |runtime| {
            let basis = runtime
                .admit_branch_basis(&runtime.main_branch_identity())
                .unwrap();
            let snapshot = runtime
                .snapshots()
                .snapshot_for_observation(&basis.observation())
                .unwrap();
            Arc::new(runtime.read_truth().positioned_snapshot(&snapshot).unwrap())
        });
    let consumed = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::ConsumedOutputEvidence::retained_for_test(identity.clone(), Arc::from([]), Vec::new(), None, selected);

    let context = reader
        .required_context_for_test("registry-fixture-family", admission, &source_owner)
        .unwrap();
    let stop = context
        .prepare_prerequisites_for_test(std::iter::once(&consumed), &source_owner)
        .err()
        .expect("prewarmed output promotion still requires admission");
    assert!(
        matches!(stop.kind(), WorthQueryOutputDemandDenialKind::DurableStateAdmission(cause) if cause == admission.unwrap_err())
    );
    let state = registry.state_for_test().lock().unwrap();
    assert_eq!(
        *state.records_for_test()[upstream_key.as_ref()].framework_required_count_for_test(),
        0
    );
    assert!(state.records_for_test()[&reader_key]
        .prerequisites_for_test()
        .is_empty());
    drop(state);
    // With the Memory policy, this same prewarmed settlement really can be
    // promoted. Its ticket releases the temporary mandatory claim on drop.
    let context = reader
        .required_context_for_test("registry-fixture-family", Ok(()), &source_owner)
        .unwrap();
    let claims = context
        .prepare_prerequisites_for_test(std::iter::once(&consumed), &source_owner)
        .unwrap();
    assert_eq!(
        *registry.state_for_test().lock().unwrap().records_for_test()[upstream_key.as_ref()]
            .framework_required_count_for_test(),
        1
    );
    drop(claims);
    assert_eq!(
        *registry.state_for_test().lock().unwrap().records_for_test()[upstream_key.as_ref()]
            .framework_required_count_for_test(),
        0
    );
}
