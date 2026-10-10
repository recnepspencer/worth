use worth_runtime_bridge::facade::{
    BridgeExecutionBasisSignalTerminal, BridgeManagedExecutionCancellationReason,
};

use super::yield_fixture::YieldProvider;
use super::*;

#[test]
fn workflow_signal_terminalized_after_safe_point_preserves_exact_recovery_evidence() {
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
            "workflow-yield-signal-race-graph",
            provider_anchor,
        );
        let runtime = super::workflow_provider_steps::installed_runtime(
            installer,
            "workflow yield signal race",
        );
        let operation_resources = crate::domain_computation::provider_session::admitted_yield_plan(
            "workflow-yield-signal-race",
            8,
        );
        let stage_resources = admitted_plan_with_graph_support(
            "workflow-yield-signal-race:stage",
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
        let logical_run_identity = running.logical_run_identity().to_owned();
        let attempt_identity = running.identity().to_owned();
        let active = running
            .begin_stage_graph_execution(
                execution,
                "stage",
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "workflow-yield-signal-race",
                ),
            )
            .unwrap();
        let paused = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("workflow provider did not pause"),
        };
        paused
            .active
            .request_cancellation(BridgeManagedExecutionCancellationReason::HostRequested)
            .expect("host should terminalize the exact workflow Signal attempt");
        let recovery = match paused.yield_run() {
            crate::domain_computation::WorthQueryWorkflowYieldOutcome::RecoveryRequired(
                recovery,
            ) => recovery,
            _ => panic!("pre-terminalized workflow Signal attempt minted yielded authority"),
        };
        assert_eq!(
            recovery.kind(),
            crate::domain_computation::WorthQueryYieldRecoveryKind::SignalAttemptAlreadyTerminal(
                BridgeExecutionBasisSignalTerminal::Cancelled,
            )
        );
        let release = match recovery.release_terminalized() {
        Ok(crate::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::Complete(
            release,
        )) => release,
        Ok(crate::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::Pending(
            _,
        )) => {
            panic!("artifact-free Signal recovery reported pending cleanup")
        }
        Ok(
            crate::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::
                RecoveryRequired(_),
        ) => panic!("artifact-free Signal recovery gained artifact recovery"),
        Err(_) => panic!("Signal-race workflow recovery did not release"),
    };
        let inspection = release.inspection();
        assert_eq!(inspection.logical_run_identity(), logical_run_identity);
        assert_eq!(inspection.yielded_attempt_identity(), attempt_identity);
        assert!(!inspection.bridge_signal_transition_performed());
        assert_eq!(inspection.provider_work().interrupted_call_count(), 1);
        assert_eq!(inspection.provider_work().completed_work_units(), 2);
    });
}
