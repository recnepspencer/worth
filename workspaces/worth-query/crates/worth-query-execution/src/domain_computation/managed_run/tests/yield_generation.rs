use std::collections::BTreeMap;
use std::sync::Arc;

use worth_runtime_bridge::facade::BridgeManagedExecutionCancellationReason;

use super::yield_fixture::YieldProvider;
use super::*;

#[test]
fn successor_installation_after_direct_safe_point_denies_yield() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (running, graph, _bridge, mut runtime) = managed_graph_run_with_provider_and_runtime(
            WorthQueryOperationGraphAccess::Observe,
            YieldProvider::installed(5),
            resource_request,
        );
        let active = running
            .begin_graph_execution(
                execution,
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "direct-stale-yield-generation",
                ),
            )
            .unwrap();
        let paused = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("direct provider did not pause"),
        };
        commit_successor(&mut runtime);

        let denied = match paused.yield_run() {
            crate::domain_computation::WorthQueryDirectYieldOutcome::Denied(denied) => denied,
            _ => panic!("stale direct installation minted yielded authority"),
        };
        assert_eq!(
            denied.kind(),
            crate::domain_computation::WorthQueryDirectYieldDenialKind::InstallationGenerationStale
        );
        let paused = denied.into_paused();
        paused
            .active
            .request_cancellation(BridgeManagedExecutionCancellationReason::HostRequested)
            .unwrap();
        let terminal = match paused.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Cancelled(terminal) => terminal,
            _ => panic!("stale direct run did not preserve cancellation authority"),
        };
        assert!(terminal.cleanup().is_ok());
    });
}

#[test]
fn successor_installation_after_workflow_safe_point_denies_yield() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let installer = WorthQueryExecutionRuntimeInstaller::new();
        let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            YieldProvider::installed(7),
        ),
    );
        let provider_support = provider_anchor.resource_support().clone();
        let graph = super::workflow_provider_steps::installed_graph(
            &installer,
            "workflow-stale-yield-generation-graph",
            provider_anchor,
        );
        let mut runtime =
            super::workflow_provider_steps::installed_runtime(installer, "workflow stale yield");
        let operation_resources = crate::domain_computation::provider_session::admitted_yield_plan(
            "workflow-stale-yield-generation",
            8,
        );
        let stage_resources = admitted_plan_with_graph_support(
            "workflow-stale-yield-generation:stage",
            8,
            graph.role(),
            provider_support,
        );
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
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "workflow-stale-yield-generation",
                ),
            )
            .unwrap();
        let paused = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("workflow provider did not pause"),
        };
        commit_successor(&mut runtime);

        let denied = match paused.yield_run() {
            crate::domain_computation::WorthQueryWorkflowYieldOutcome::Denied(denied) => denied,
            _ => panic!("stale workflow installation minted yielded authority"),
        };
        assert_eq!(
        denied.kind(),
        crate::domain_computation::WorthQueryWorkflowYieldDenialKind::InstallationGenerationStale
    );
        let paused = denied.into_paused();
        paused
            .active
            .request_cancellation(BridgeManagedExecutionCancellationReason::HostRequested)
            .unwrap();
        let terminal = match paused.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Cancelled(terminal) => terminal,
            _ => panic!("stale workflow run did not preserve cancellation authority"),
        };
        match terminal.cleanup() {
            WorthQueryWorkflowRunCleanupOutcome::Complete(_) => {}
            _ => panic!("stale workflow cancellation did not clean up"),
        }
    });
}

fn commit_successor(runtime: &mut WorthQueryExecutionRuntime) {
    let successor = Arc::new(runtime.installed_packages().successor_generation());
    runtime
        .commit_successor_installation(successor)
        .expect("test runtime should admit its exact successor generation");
}
