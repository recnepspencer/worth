use worth_runtime_bridge::facade::{
    BridgeExecutionBasisFinalizationFailureKind, BridgeManagedExecutionCancellationReason,
};

use super::yield_fixture::YieldProvider;
use super::*;

#[test]
fn bridge_terminalization_failure_preserves_the_paused_run_for_retry_or_cleanup() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (running, graph) = managed_graph_run_with_provider(
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
                    "yield-bridge-thread-affinity",
                ),
            )
            .unwrap();
        let paused = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("provider did not pause"),
        };
        let recovery = std::thread::spawn(move || match paused.yield_run() {
            crate::domain_computation::WorthQueryDirectYieldOutcome::RecoveryRequired(recovery) => {
                recovery
            }
            _ => panic!("foreign-thread bridge finalization did not require recovery"),
        })
        .join()
        .expect("foreign-thread yield probe should return recovery authority");
        assert_eq!(
            recovery.kind(),
            crate::domain_computation::WorthQueryYieldRecoveryKind::BridgeTerminalization(
                BridgeExecutionBasisFinalizationFailureKind::SignalRuntimeThreadAffinityViolation,
            )
        );
        assert!(recovery.running_attempt_recoverable());

        let paused = match recovery.into_paused() {
            Ok(paused) => paused,
            Err(_) => panic!("bridge failure consumed the still-running attempt"),
        };
        paused
            .active
            .request_cancellation(BridgeManagedExecutionCancellationReason::HostRequested)
            .unwrap();
        let terminal = match paused.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Cancelled(terminal) => terminal,
            _ => panic!("recovered paused run did not retain cancellation authority"),
        };
        assert!(terminal.cleanup().is_ok());
    });
}

#[test]
fn workflow_bridge_failure_aborts_artifact_freeze_and_restores_production() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let installer = WorthQueryExecutionRuntimeInstaller::new();
        let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            YieldProvider::installed(5),
        ),
    );
        let provider_support = provider_anchor.resource_support().clone();
        let graph = super::workflow_provider_steps::installed_graph(
            &installer,
            "workflow-yield-bridge-failure-graph",
            provider_anchor,
        );
        let runtime = super::workflow_provider_steps::installed_runtime(
            installer,
            "workflow yield bridge failure",
        );
        let operation_resources = crate::domain_computation::provider_session::admitted_yield_plan(
            "workflow-yield-bridge-failure",
            8,
        );
        let stage_resources = admitted_plan_with_graph_support(
            "workflow-yield-bridge-failure:producer",
            8,
            graph.role(),
            provider_support,
        );
        let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
            operation_resources,
            BTreeMap::from([("producer".to_owned(), stage_resources)]),
        );
        let output =
            crate::domain_computation::artifact_owner::installed_artifact_contract_for_managed_run(
            );
        let operation = workflow_authority_with_stage_graph_and_output_artifact(
            &runtime,
            &resources,
            "producer",
            &graph,
            WorthQueryOperationGraphAccess::Observe,
            output,
        );
        let running = super::workflow_provider_steps::admitted_workflow(
            &runtime,
            &operation,
            resources,
            resource_request,
        );
        let production = running
            .artifacts()
            .production_authority("producer")
            .unwrap()
            .expect("producer output artifact should install");
        let active = running
            .begin_stage_graph_execution(
                execution,
                "producer",
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "workflow-yield-bridge-failure",
                ),
            )
            .unwrap();
        let paused = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("workflow provider did not pause"),
        };

        let recovery = std::thread::spawn(move || match paused.yield_run() {
            crate::domain_computation::WorthQueryWorkflowYieldOutcome::RecoveryRequired(
                recovery,
            ) => recovery,
            _ => panic!("foreign-thread workflow yield did not require recovery"),
        })
        .join()
        .expect("foreign-thread workflow yield should preserve recovery authority");
        assert_eq!(
            recovery.kind(),
            crate::domain_computation::WorthQueryYieldRecoveryKind::BridgeTerminalization(
                BridgeExecutionBasisFinalizationFailureKind::SignalRuntimeThreadAffinityViolation,
            )
        );
        assert!(recovery.running_attempt_recoverable());
        let paused = recovery
            .into_paused()
            .unwrap_or_else(|_| panic!("Bridge failure consumed the paused workflow"));

        let disposals = Arc::new(AtomicUsize::new(0));
        let admission =
            crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::admit(
                &production,
                WorthQueryArtifactProductionEvidence::new(
                    "post-bridge-failure-provenance",
                    "post-bridge-failure-dependency",
                ),
            );
        let handle =
        crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::register_exact(
            &production,
            admission,
            BridgeFailureArtifact(Arc::clone(&disposals)),
        )
        .expect("Bridge failure must abort the provisional artifact freeze");
        drop(handle);
        assert_eq!(disposals.load(Ordering::Acquire), 1);

        paused
            .active
            .request_cancellation(BridgeManagedExecutionCancellationReason::HostRequested)
            .unwrap();
        let terminal = match paused.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Cancelled(terminal) => terminal,
            _ => panic!("recovered workflow did not retain cancellation authority"),
        };
        match terminal.cleanup() {
            WorthQueryWorkflowRunCleanupOutcome::Complete(_) => {}
            _ => panic!("recovered workflow did not clean every retained authority"),
        }
    });
}

struct BridgeFailureArtifact(Arc<AtomicUsize>);

impl WorthQueryArtifactProviderResource for BridgeFailureArtifact {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.affinity.provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        b"bridge-failure-artifact".to_vec()
    }

    fn retained_bytes(&self) -> usize {
        32
    }

    fn dispose(&mut self) {
        self.0.fetch_add(1, Ordering::AcqRel);
    }
}
