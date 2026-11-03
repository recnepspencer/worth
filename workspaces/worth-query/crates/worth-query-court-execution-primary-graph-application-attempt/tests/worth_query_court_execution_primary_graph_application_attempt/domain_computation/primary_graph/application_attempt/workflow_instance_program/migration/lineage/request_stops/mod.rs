use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::workflow_instance_program::migration::lineage::successor_identity;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
#[test]
fn successor_read_keeps_cancellation_at_the_native_frontier() {
    let _observation =
            worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    world.application.with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let execution = host.borrowed_request_for_test();
            let selected = world.selected_product(&host);
            let graph = world.application.runtime.primary_graph().unwrap();
            let request = worth_execution::CancellationSource::new();
            let serial = worth_execution::SerialRequest::from_memory(worth_execution::SerialMemoryBudget::new(None), request.token(), None);
            let cancelled = worth_execution::ExecutionRequest::serial(&serial);
            let anchor = selected.resolve_entity(&phase, worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus::reference(), "open".to_owned(), &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope(), worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode::Ordinary).unwrap().entity_id();
            graph.integration_handle().with_runtime(&execution, |runtime| {
                let snapshot = selected.application_basis().snapshot_handle();
                let _probe = worth_execution::ExecutionCheckpointObservationForTest::install_on_this_thread_for_test(move || request.cancel());
                let denial = successor_identity(&cancelled, runtime, snapshot, graph.layout.workflow(), anchor).unwrap_err();
                assert_eq!(denial.kind(), worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind::Cancelled);
            });
        }).unwrap();
}
