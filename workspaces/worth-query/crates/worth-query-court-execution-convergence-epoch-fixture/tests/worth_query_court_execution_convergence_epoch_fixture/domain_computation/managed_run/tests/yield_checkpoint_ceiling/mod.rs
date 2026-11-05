use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::yield_fixture::YieldProvider;
use crate::domain_computation::managed_run::tests::*;

#[test]
fn checkpoint_memory_mismatch_preserves_exact_release_evidence() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        for (provider, expected_release, label) in [
            (
                YieldProvider::checkpoint_memory_mismatch(3_000, 4_097, false),
                worth_query_execution::facade::domain_computation::WorthQueryProviderCheckpointReleaseDisposition::Released,
                "workflow-checkpoint-memory-mismatch",
            ),
            (
                YieldProvider::checkpoint_memory_mismatch(3_000, 4_097, true),
                worth_query_execution::facade::domain_computation::WorthQueryProviderCheckpointReleaseDisposition::Panicked,
                "workflow-checkpoint-memory-mismatch-drop-panic",
            ),
        ] {
            let paused =
                paused_workflow_checkpoint_target(execution, provider, label, resource_request);
            let recovery = match paused.yield_run() {
                worth_query_execution::facade::domain_computation::WorthQueryWorkflowYieldOutcome::RecoveryRequired(
                    recovery,
                ) => recovery,
                _ => panic!("checkpoint memory mismatch minted yielded authority"),
            };
            assert_eq!(
            recovery.kind(),
            worth_query_execution::facade::domain_computation::WorthQueryYieldRecoveryKind::ProviderCheckpointSuspension(
                worth_query_execution::facade::domain_computation::WorthQueryProviderCheckpointSuspensionFailureKind::CheckpointMemoryMismatch,
            )
        );
            let checkpoint_release = recovery
                .resource_evidence()
                .provider_checkpoint_failure()
                .expect("memory mismatch carries suspension failure evidence")
                .checkpoint_release()
                .expect("memory mismatch carries the rejected checkpoint");
            assert_eq!(checkpoint_release.checkpoint().retained_bytes(), 4_097);
            assert_eq!(checkpoint_release.disposition(), expected_release);
            let release = match recovery.release_terminalized() {
            Ok(
                worth_query_execution::facade::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::Complete(
                    release,
                ),
            ) => release,
            Ok(
                worth_query_execution::facade::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::Pending(
                    _,
                ),
            ) => panic!("artifact-free checkpoint mismatch recovery reported pending cleanup"),
            Ok(
                worth_query_execution::facade::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::RecoveryRequired(_),
            ) => panic!("artifact-free checkpoint mismatch gained artifact recovery"),
            Err(_) => panic!("checkpoint mismatch recovery lost terminalized release authority"),
        };
            assert_eq!(
                release
                    .inspection()
                    .checkpoint()
                    .expect("release preserves checkpoint mismatch evidence")
                    .release_disposition(),
                expected_release
            );
            assert!(release.inspection().resources_released());
            assert_eq!(release.inspection().released_reservation_count(), 3);
        }
    });
}

pub fn paused_workflow_checkpoint_target(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    provider: YieldProvider,
    label: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> worth_query_execution::facade::domain_computation::WorthQueryPausedWorkflowGraphExecution {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        worth_query_execution::facade::integration::WorthQueryGraphProviderAnchor::install::<
            ManagedGraph,
            _,
        >(provider),
    );
    let provider_support = provider_anchor.resource_support().clone();
    let graph = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::installed_graph(
        &installer,
        &format!("{label}-graph"),
        provider_anchor,
    );
    let runtime =
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::installed_runtime(installer, "workflow checkpoint ceiling");
    let operation_resources =
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::admitted_yield_plan(label, 8);
    let stage_resource_label = format!("{label}:stage");
    let stage_resources =
        admitted_plan_with_graph_support(&stage_resource_label, 8, graph.role(), provider_support);
    let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
        operation_resources,
        BTreeMap::from([("stage".to_owned(), stage_resources)]),
    );
    let operation = workflow_authority_with_stage_graph(
        &runtime,
        &resources,
        "stage",
        &graph,
        WorthQueryOperationGraphAccess::Observe,
    );
    let running = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::workflow_provider_steps::admitted_workflow(
        &runtime,
        &operation,
        resources,
        resource_request,
    );
    let installed_running_phase = execution.phase_for_workflow_for_test(&running);

    let active = running
        .begin_stage_graph_execution(
            &installed_running_phase,
            "stage",
            &graph,
            WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, label),
        )
        .expect("checkpoint ceiling provider should begin");
    match active.advance(&installed_running_phase) {
        WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
        _ => panic!("checkpoint ceiling provider did not reach its safe point"),
    }
}
