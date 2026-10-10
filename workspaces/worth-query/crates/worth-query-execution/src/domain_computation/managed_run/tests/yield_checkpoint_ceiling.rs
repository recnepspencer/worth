use super::yield_fixture::YieldProvider;
use super::*;

#[test]
fn checkpoint_memory_mismatch_preserves_exact_release_evidence() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        for (provider, expected_release, label) in [
            (
                YieldProvider::checkpoint_memory_mismatch(3_000, 4_097, false),
                crate::domain_computation::WorthQueryProviderCheckpointReleaseDisposition::Released,
                "workflow-checkpoint-memory-mismatch",
            ),
            (
                YieldProvider::checkpoint_memory_mismatch(3_000, 4_097, true),
                crate::domain_computation::WorthQueryProviderCheckpointReleaseDisposition::Panicked,
                "workflow-checkpoint-memory-mismatch-drop-panic",
            ),
        ] {
            let paused =
                paused_workflow_checkpoint_target(execution, provider, label, resource_request);
            let recovery = match paused.yield_run() {
                crate::domain_computation::WorthQueryWorkflowYieldOutcome::RecoveryRequired(
                    recovery,
                ) => recovery,
                _ => panic!("checkpoint memory mismatch minted yielded authority"),
            };
            assert_eq!(
            recovery.kind(),
            crate::domain_computation::WorthQueryYieldRecoveryKind::ProviderCheckpointSuspension(
                crate::domain_computation::WorthQueryProviderCheckpointSuspensionFailureKind::
                    CheckpointMemoryMismatch,
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
                crate::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::Complete(
                    release,
                ),
            ) => release,
            Ok(
                crate::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::Pending(
                    _,
                ),
            ) => panic!("artifact-free checkpoint mismatch recovery reported pending cleanup"),
            Ok(
                crate::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::
                    RecoveryRequired(_),
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

fn paused_workflow_checkpoint_target(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    provider: YieldProvider,
    label: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::domain_computation::WorthQueryPausedWorkflowGraphExecution {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            provider,
        ),
    );
    let provider_support = provider_anchor.resource_support().clone();
    let graph = super::workflow_provider_steps::installed_graph(
        &installer,
        &format!("{label}-graph"),
        provider_anchor,
    );
    let runtime =
        super::workflow_provider_steps::installed_runtime(installer, "workflow checkpoint ceiling");
    let operation_resources =
        crate::domain_computation::provider_session::admitted_yield_plan(label, 8);
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
    let running = super::workflow_provider_steps::admitted_workflow(
        &runtime,
        &operation,
        resources,
        resource_request,
    );
    let active = running
        .begin_stage_graph_execution(
            execution,
            "stage",
            &graph,
            WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, label),
        )
        .expect("checkpoint ceiling provider should begin");
    match active.advance(execution) {
        WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
        _ => panic!("checkpoint ceiling provider did not reach its safe point"),
    }
}
