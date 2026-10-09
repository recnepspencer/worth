use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use worth_runtime_bridge::facade::BridgeExecutionBasisSignalTerminal;

use super::yield_fixture::YieldProvider;
use super::*;

#[test]
fn workflow_yield_retains_operation_and_stage_capacity_until_cleanup() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
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
            "workflow-yield-graph",
            provider_anchor,
        );
        let runtime =
            super::workflow_provider_steps::installed_runtime(installer, "workflow yield");
        let operation_resources =
            crate::domain_computation::provider_session::admitted_yield_plan("workflow-yield", 8);
        let stage_resources = admitted_plan_with_graph_support(
            "workflow-yield:stage",
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
        let running =
            super::workflow_provider_steps::admitted_workflow(&runtime, &operation, resources);
        let logical_run_identity = running.logical_run_identity().to_owned();
        let attempt_identity = running.identity().to_owned();
        let active = running
            .begin_stage_graph_execution(
                execution,
                "stage",
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "workflow-yield",
                ),
            )
            .unwrap();
        let paused = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("workflow provider did not pause"),
        };

        let yielded = match paused.yield_run() {
            crate::domain_computation::WorthQueryWorkflowYieldOutcome::Yielded(yielded) => yielded,
            _ => panic!("eligible workflow did not yield"),
        };
        assert_eq!(
            yielded.inspection().logical_run_identity(),
            logical_run_identity
        );
        assert_eq!(
            yielded.inspection().yielded_attempt_identity(),
            attempt_identity
        );
        assert_eq!(yielded.inspection().checkpoint().retained_bytes(), 7);
        assert_eq!(
            yielded.inspection().retained_capacity_reservation_count(),
            3
        );
        assert_eq!(
            yielded
                .inspection()
                .provider_work()
                .interrupted_call_count(),
            1
        );
        assert_eq!(
            yielded
                .inspection()
                .artifact_evidence()
                .retained_artifact_count(),
            0
        );

        let cleanup = match yielded.cleanup() {
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::Complete(receipt) => {
                receipt
            }
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::Pending(_) => {
                panic!("artifact-free yielded workflow reported pending cleanup")
            }
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::RecoveryRequired(
                _,
            ) => {
                panic!("ordinary checkpoint release unexpectedly required recovery")
            }
        };
        assert_eq!(
            cleanup.inspection().logical_run_identity(),
            logical_run_identity
        );
        assert!(cleanup.inspection().resources_released());
        assert_eq!(cleanup.inspection().released_reservation_count(), 3);
        assert_eq!(cleanup.inspection().checkpoint().retained_bytes(), 7);
    });
}

#[test]
fn workflow_suspension_failure_returns_terminalized_release_authority() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let installer = WorthQueryExecutionRuntimeInstaller::new();
        let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            YieldProvider::suspension_failure(),
        ),
    );
        let provider_support = provider_anchor.resource_support().clone();
        let graph = super::workflow_provider_steps::installed_graph(
            &installer,
            "workflow-yield-failure-graph",
            provider_anchor,
        );
        let runtime =
            super::workflow_provider_steps::installed_runtime(installer, "workflow yield failure");
        let operation_resources = crate::domain_computation::provider_session::admitted_yield_plan(
            "workflow-yield-failure",
            8,
        );
        let stage_resources = admitted_plan_with_graph_support(
            "workflow-yield-failure:stage",
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
        let running =
            super::workflow_provider_steps::admitted_workflow(&runtime, &operation, resources);
        let active = running
            .begin_stage_graph_execution(
                execution,
                "stage",
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "workflow-yield-failure",
                ),
            )
            .unwrap();
        let paused = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("workflow provider did not pause"),
        };
        let recovery = match paused.yield_run() {
            crate::domain_computation::WorthQueryWorkflowYieldOutcome::RecoveryRequired(
                recovery,
            ) => recovery,
            _ => panic!("workflow suspension failure did not preserve recovery authority"),
        };
        assert!(!recovery.running_attempt_recoverable());
        assert_eq!(
        recovery.kind(),
        crate::domain_computation::WorthQueryYieldRecoveryKind::ProviderCheckpointSuspension(
            crate::domain_computation::WorthQueryProviderCheckpointSuspensionFailureKind::
                ProviderRejected,
        )
    );
        let release = match recovery.release_terminalized() {
        Ok(crate::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::Complete(
            release,
        )) => release,
        Ok(crate::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::Pending(
            _,
        )) => {
            panic!("artifact-free workflow recovery reported pending cleanup")
        }
        Ok(
            crate::domain_computation::WorthQueryWorkflowYieldRecoveryReleaseOutcome::
                RecoveryRequired(_),
        ) => panic!("artifact-free workflow recovery gained artifact recovery"),
        Err(_) => panic!("artifact-free workflow recovery did not release"),
    };
        assert_eq!(
            release.inspection().bridge_signal_terminal(),
            BridgeExecutionBasisSignalTerminal::Cancelled
        );
        assert!(release.inspection().resources_released());
        assert_eq!(release.inspection().released_reservation_count(), 3);
    });
}

struct YieldArtifactResource(Arc<AtomicUsize>);

impl WorthQueryArtifactProviderResource for YieldArtifactResource {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.affinity.provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        b"yield-artifact".to_vec()
    }

    fn retained_bytes(&self) -> usize {
        64
    }

    fn dispose(&mut self) {
        self.0.fetch_add(1, Ordering::AcqRel);
    }
}

#[path = "yield_lifecycle_workflow/workflow_yield_cleanup_waits_for_retained_artifact_owners.rs"]
mod workflow_yield_cleanup_waits_for_retained_artifact_owners;
