//! Governed artifact retention survives a later provider call.
use super::*;

#[test]
fn governed_artifact_retention_survives_a_later_provider_call() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let installer = WorthQueryExecutionRuntimeInstaller::new();
        let retained = Arc::new(Mutex::new(None));
        let disposed = Arc::new(AtomicUsize::new(0));
        let provider_anchor = Arc::new(
        worth_query_execution::facade::integration::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            MultiCallArtifactProvider {
                begin_ordinal: Arc::new(AtomicUsize::new(0)),
                retained: Arc::clone(&retained),
                disposed: Arc::clone(&disposed),
            },
        ),
    );
        let provider_support = provider_anchor.resource_support().clone();
        let graph = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::installed_graph(
            &installer,
            "multi-call-artifact-graph",
            provider_anchor,
        );
        let runtime = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::installed_runtime(installer, "multi-call");
        let operation_resources = admitted_plan("multi-call-artifact", 8);
        let stage_resources = admitted_plan_with_graph_support(
            "multi-call-artifact:producer",
            8,
            graph.role(),
            provider_support,
        );
        let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
            operation_resources,
            BTreeMap::from([("producer".to_owned(), stage_resources)]),
        );
        let output =
            worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::artifact_owner::authority_affinity_tests::installed_artifact_contract_for_managed_run(
            );
        let operation = workflow_authority_with_stage_graph_and_output_artifact(
            &runtime,
            &resources,
            "producer",
            &graph,
            WorthQueryOperationGraphAccess::Observe,
            output,
        );
        let running = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::admitted_workflow(
            &runtime,
            &operation,
            resources,
            resource_request,
        );

        let first = complete_observe_call(execution, running, &graph, "multi-call-first");
        assert_eq!(first.receipt().work_report().produced_artifact_count(), 1);
        assert_eq!(first.receipt().work_report().retained_artifact_count(), 1);
        let second =
            complete_observe_call(execution, first.into_running(), &graph, "multi-call-second");
        assert_eq!(second.receipt().work_report().produced_artifact_count(), 0);
        assert_eq!(second.receipt().work_report().retained_artifact_count(), 0);

        let terminal = second.into_running().completed().unwrap();
        assert_eq!(terminal.provider_work().produced_artifact_count(), 1);
        assert_eq!(terminal.provider_work().retained_artifact_count(), 1);
        assert_eq!(terminal.provider_work().disposed_artifact_count(), 0);
        assert_eq!(terminal.provider_work().retained_bytes(), 37);
        assert_eq!(terminal.artifact_evidence().retained_artifact_count(), 1);

        drop(
            retained
                .lock()
                .expect("retained artifact lock should remain available")
                .take(),
        );
        assert_eq!(disposed.load(Ordering::Acquire), 1);
        match terminal.cleanup() {
            WorthQueryWorkflowRunCleanupOutcome::Complete(receipt) => {
                assert_eq!(
                    receipt
                        .inspection()
                        .artifact_evidence()
                        .disposed_artifact_count(),
                    1
                );
            }
            _ => panic!("released governed artifact should permit cleanup"),
        }
    });
}
