use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandRecoveryPosture;

#[test]
fn pending_required_prerequisite_preserves_its_retryable_operation() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let demand_key = key("dependent", 70, 1);
    let wake = Arc::new(DemandWake {
        generation: Mutex::new(0),
        changed: Condvar::new(),
    });
    (*(*registry.state_for_test()).lock().unwrap().records_for_test()).insert_for_test(
        demand_key.clone(),
        DemandRecord {
            required_interests: 1,
            wake: Arc::clone(&wake),
            ..record(occurrence(), DemandState::Running, 1)
        },
    );
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
        (*(*(*registry.state_for_test()).lock().unwrap().records_for_test())[&demand_key].state_for_test()),
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
    let wake = Arc::new(DemandWake {
        generation: Mutex::new(0),
        changed: Condvar::new(),
    });
    (*(*registry.state_for_test()).lock().unwrap().records_for_test()).insert_for_test(
        reader_key.clone(),
        DemandRecord {
            required_interests: 1,
            wake: Arc::clone(&wake),
            ..record(occurrence(), DemandState::Running, 1)
        },
    );
    let reader = required_interest(&registry, reader_key, wake);
    let resources = WorthQueryInvalidationResources::new();
    let source_owner =
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner::new(
            resources, 1,
        );

    let context = reader
        .required_context_for_test("registry-fixture-family", &source_owner)
        .expect("the running reader issues its required context");
    // The upstream row this execution read is no longer indexed: a newer
    // settlement of its demand retired it.
    let (_lineage, retired) = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    let graph = (*world.application.runtime_for_test()).primary_graph().unwrap();
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
