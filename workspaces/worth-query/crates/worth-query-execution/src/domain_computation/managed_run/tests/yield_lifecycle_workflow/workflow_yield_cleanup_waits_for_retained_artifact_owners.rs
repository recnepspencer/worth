//! Workflow yield cleanup waits for retained artifact owners.

use super::*;

#[test]
fn workflow_yield_cleanup_waits_for_retained_artifact_owners() {
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
        let graph = super::super::workflow_provider_steps::installed_graph(
            &installer,
            "workflow-yield-artifact-graph",
            provider_anchor,
        );
        let runtime = super::super::workflow_provider_steps::installed_runtime(
            installer,
            "workflow yield artifact",
        );
        let operation_resources = crate::domain_computation::provider_session::admitted_yield_plan(
            "workflow-yield-artifact",
            8,
        );
        let stage_resources = admitted_plan_with_graph_support(
            "workflow-yield-artifact:producer",
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
        let running = super::super::workflow_provider_steps::admitted_workflow(
            &runtime, &operation, resources,
        );
        let production = running
            .artifacts()
            .production_authority("producer")
            .unwrap()
            .expect("output artifact contract should install");
        let admission =
            crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::admit(
                &production,
                WorthQueryArtifactProductionEvidence::new(
                    "yield-artifact-provenance",
                    "yield-artifact-dependency",
                ),
            );
        let disposed = Arc::new(AtomicUsize::new(0));
        let handle =
        crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::register_exact(
            &production,
            admission,
            YieldArtifactResource(Arc::clone(&disposed)),
        )
        .expect("exact artifact authority should register");
        let borrowed = handle
            .borrow("yield cleanup ownership probe")
            .expect("installed artifact contract should allow read borrow");
        let active = running
            .begin_stage_graph_execution(
                execution,
                "producer",
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "workflow-yield-artifact",
                ),
            )
            .unwrap();
        let paused = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("workflow provider did not pause"),
        };
        let yielded = match paused.yield_run() {
            crate::domain_computation::WorthQueryWorkflowYieldOutcome::Yielded(yielded) => yielded,
            _ => panic!("artifact-owning workflow did not yield"),
        };
        let yielded_attempt_identity = yielded.inspection().yielded_attempt_identity().to_owned();
        let provider_session_identity = yielded.inspection().provider_session_identity().to_owned();
        assert_eq!(
            yielded
                .inspection()
                .artifact_evidence()
                .retained_artifact_count(),
            1
        );
        let rejected_disposed = Arc::new(AtomicUsize::new(0));
        let rejected_admission =
            crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::admit(
                &production,
                WorthQueryArtifactProductionEvidence::new(
                    "post-yield-provenance",
                    "post-yield-dependency",
                ),
            );
        let denial =
        match crate::domain_computation::artifact_owner::WorthQueryArtifactProductionAuthority::register_exact(
            &production,
            rejected_admission,
            YieldArtifactResource(Arc::clone(&rejected_disposed)),
        ) {
            Ok(_) => panic!("yielded workflow retained live artifact production authority"),
            Err(denial) => denial,
        };
        assert_eq!(
            denial.kind(),
            crate::domain_computation::WorthQueryArtifactDenialKind::ProductionClosed
        );
        assert_eq!(rejected_disposed.load(Ordering::Acquire), 1);
        let pending = match yielded.cleanup() {
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::Pending(pending) => {
                pending
            }
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::Complete(_) => {
                panic!("live artifact owner allowed yielded cleanup to release lower authority")
            }
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::RecoveryRequired(
                _,
            ) => {
                panic!("ordinary checkpoint release unexpectedly required recovery")
            }
        };
        assert_eq!(disposed.load(Ordering::Acquire), 0);

        drop(borrowed);
        assert_eq!(disposed.load(Ordering::Acquire), 1);
        drop(handle);
        let cleanup = match pending.retry() {
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::Complete(cleanup) => {
                cleanup
            }
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::Pending(_) => {
                panic!("released artifact owners kept yielded cleanup pending")
            }
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::RecoveryRequired(
                _,
            ) => {
                panic!("ordinary checkpoint release unexpectedly required recovery")
            }
        };
        assert_eq!(
            cleanup
                .inspection()
                .artifact_evidence()
                .disposed_artifact_count(),
            1
        );
        assert_eq!(
            cleanup.inspection().yielded_attempt_identity(),
            yielded_attempt_identity
        );
        assert_eq!(
            cleanup.inspection().provider_session_identity(),
            provider_session_identity
        );
        assert!(cleanup.inspection().resources_released());
        assert_eq!(cleanup.inspection().released_reservation_count(), 3);
    });
}
