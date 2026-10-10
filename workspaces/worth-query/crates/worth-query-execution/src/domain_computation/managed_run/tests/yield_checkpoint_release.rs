use std::collections::BTreeMap;
use std::sync::Arc;

use super::yield_fixture::YieldProvider;
use super::*;

#[test]
fn suspension_and_execution_destructor_panics_are_independently_contained() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        for (provider, expected_kind, label) in [
        (
            YieldProvider::suspension_failure_and_execution_drop_panic(),
            crate::domain_computation::WorthQueryProviderCheckpointSuspensionFailureKind::
                ProviderRejected,
            "direct-suspension-rejection-and-execution-drop-panic",
        ),
        (
            YieldProvider::suspension_and_execution_drop_panic(),
            crate::domain_computation::WorthQueryProviderCheckpointSuspensionFailureKind::
                ProviderPanicked,
            "direct-suspension-and-execution-drop-panic",
        ),
    ] {
        let (running, graph) =
            managed_graph_run_with_provider(WorthQueryOperationGraphAccess::Observe, provider, resource_request);
        let active = running
            .begin_graph_execution(execution,
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    label,
                ),
            )
            .unwrap();
        let paused = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("double-panic provider did not pause"),
        };
        let recovery = match paused.yield_run() {
            crate::domain_computation::WorthQueryDirectYieldOutcome::RecoveryRequired(recovery) => {
                recovery
            }
            _ => panic!("double-panic suspension did not require recovery"),
        };
        assert_eq!(
            recovery.kind(),
            crate::domain_computation::WorthQueryYieldRecoveryKind::ProviderCheckpointSuspension(
                expected_kind,
            )
        );
        let failure = recovery
            .resource_evidence()
            .provider_checkpoint_failure()
            .expect("suspension failure must remain inspectable");
        assert_eq!(
            failure.provider_execution_release().disposal(),
            crate::domain_computation::WorthQueryProviderExecutionDisposalDisposition::Completed
        );
        assert_eq!(
            failure.provider_execution_release().destructor(),
            crate::domain_computation::WorthQueryProviderExecutionDestructorDisposition::Panicked
        );
        assert!(failure.checkpoint_release().is_none());
        let cleanup = match recovery.cleanup_terminalized() {
            Ok(cleanup) => cleanup,
            Err(_) => panic!("terminalized double-panic recovery was not cleanable"),
        };
        let release = cleanup.inspection().provider_work().provider_execution_release();
        assert_eq!(release.release_count(), 1);
        assert_eq!(
            release.panicked_destructor_count(),
            1
        );
        assert!(cleanup.inspection().resources_released());
        assert_eq!(cleanup.inspection().released_reservation_count(), 2);
    }
    });
}

#[test]
fn execution_destructor_panic_releases_returned_checkpoint_with_exact_disposition() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        for (checkpoint_drop_panics, expected_release, label) in [
            (
                false,
                crate::domain_computation::WorthQueryProviderCheckpointReleaseDisposition::Released,
                "direct-checkpoint-release-after-execution-drop-panic",
            ),
            (
                true,
                crate::domain_computation::WorthQueryProviderCheckpointReleaseDisposition::Panicked,
                "direct-checkpoint-drop-panic-after-execution-drop-panic",
            ),
        ] {
            let (running, graph) = managed_graph_run_with_provider(
                WorthQueryOperationGraphAccess::Observe,
                YieldProvider::checkpoint_and_execution_drop_panic(checkpoint_drop_panics),
                resource_request,
            );
            let active = running
                .begin_graph_execution(
                    execution,
                    &graph,
                    WorthQueryManagedGraphCallRequest::new(
                        WorthQueryGraphProviderCallKind::Observe,
                        label,
                    ),
                )
                .unwrap();
            let paused = match active.advance(execution) {
                WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
                _ => panic!("execution-drop-panic provider did not pause"),
            };
            let recovery = match paused.yield_run() {
                crate::domain_computation::WorthQueryDirectYieldOutcome::RecoveryRequired(
                    recovery,
                ) => recovery,
                _ => panic!("execution destructor panic minted a yielded capability"),
            };
            assert_eq!(
            recovery.kind(),
            crate::domain_computation::WorthQueryYieldRecoveryKind::ProviderCheckpointSuspension(
                crate::domain_computation::WorthQueryProviderCheckpointSuspensionFailureKind::
                    ProviderExecutionReleaseRecoveryRequired,
            )
        );
            let failure = recovery
                .resource_evidence()
                .provider_checkpoint_failure()
                .expect("suspension failure must remain inspectable");
            assert_eq!(
            failure.provider_execution_release().disposal(),
            crate::domain_computation::WorthQueryProviderExecutionDisposalDisposition::Completed
        );
            assert_eq!(
            failure.provider_execution_release().destructor(),
            crate::domain_computation::WorthQueryProviderExecutionDestructorDisposition::Panicked
        );
            assert_eq!(
                failure
                    .checkpoint_release()
                    .expect("checkpoint must be released after execution release fails")
                    .disposition(),
                expected_release
            );
            assert_eq!(failure.checkpoint_retained_byte_probe_count(), 1);
            let cleanup = match recovery.cleanup_terminalized() {
                Ok(cleanup) => cleanup,
                Err(_) => panic!("terminalized execution-drop recovery was not cleanable"),
            };
            assert!(cleanup.inspection().resources_released());
            assert_eq!(cleanup.inspection().released_reservation_count(), 2);
        }
    });
}

#[test]
fn checkpoint_probe_failure_preserves_its_own_release_disposition() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        for (provider, expected_release) in [
            (
                YieldProvider::checkpoint_probe_panic(),
                crate::domain_computation::WorthQueryProviderCheckpointReleaseDisposition::Released,
            ),
            (
                YieldProvider::checkpoint_probe_and_drop_panic(),
                crate::domain_computation::WorthQueryProviderCheckpointReleaseDisposition::Panicked,
            ),
        ] {
            let (running, graph) = managed_graph_run_with_provider(
                WorthQueryOperationGraphAccess::Observe,
                provider,
                resource_request,
            );
            let active = running
                .begin_graph_execution(
                    execution,
                    &graph,
                    WorthQueryManagedGraphCallRequest::new(
                        WorthQueryGraphProviderCallKind::Observe,
                        "direct-checkpoint-probe-release-evidence",
                    ),
                )
                .unwrap();
            let paused = match active.advance(execution) {
                WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
                _ => panic!("checkpoint probe provider did not pause"),
            };
            let recovery = match paused.yield_run() {
                crate::domain_computation::WorthQueryDirectYieldOutcome::RecoveryRequired(
                    recovery,
                ) => recovery,
                _ => panic!("checkpoint probe panic minted a yielded capability"),
            };
            let failure = recovery
                .resource_evidence()
                .provider_checkpoint_failure()
                .expect("checkpoint retention failure must remain inspectable");
            assert_eq!(
                failure
                    .checkpoint_retention_failure()
                    .expect("retained-byte probe failure carries checkpoint evidence")
                    .release_disposition(),
                expected_release
            );
            match recovery.cleanup_terminalized() {
                Ok(_) => {}
                Err(_) => panic!("checkpoint probe recovery was not cleanable"),
            }
        }
    });
}

#[test]
fn direct_cleanup_contains_and_reports_checkpoint_destructor_panic() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (running, graph) = managed_graph_run_with_provider(
            WorthQueryOperationGraphAccess::Observe,
            YieldProvider::checkpoint_drop_panic(),
            resource_request,
        );
        let active = running
            .begin_graph_execution(
                execution,
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "direct-checkpoint-drop-panic",
                ),
            )
            .unwrap();
        let paused = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("drop-panic provider did not pause"),
        };
        let yielded = match paused.yield_run() {
            crate::domain_computation::WorthQueryDirectYieldOutcome::Yielded(yielded) => yielded,
            _ => panic!("drop-panic checkpoint did not yield before cleanup"),
        };
        let cleanup = match yielded.cleanup() {
            crate::domain_computation::WorthQueryDirectYieldCleanupOutcome::RecoveryRequired(
                recovery,
            ) => recovery,
            crate::domain_computation::WorthQueryDirectYieldCleanupOutcome::Complete(_) => {
                panic!("checkpoint destructor panic claimed complete direct cleanup")
            }
        };
        assert_eq!(
            cleanup
                .inspection()
                .checkpoint()
                .expect("yielded cleanup carries checkpoint release")
                .release_disposition(),
            crate::domain_computation::WorthQueryProviderCheckpointReleaseDisposition::Panicked
        );
        assert!(cleanup.inspection().resources_released());
        assert_eq!(cleanup.inspection().released_reservation_count(), 2);
    });
}

#[test]
fn workflow_cleanup_returns_recovery_required_after_checkpoint_destructor_panic() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let installer = WorthQueryExecutionRuntimeInstaller::new();
        let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            YieldProvider::checkpoint_drop_panic(),
        ),
    );
        let provider_support = provider_anchor.resource_support().clone();
        let graph = super::workflow_provider_steps::installed_graph(
            &installer,
            "workflow-checkpoint-drop-panic-graph",
            provider_anchor,
        );
        let runtime = super::workflow_provider_steps::installed_runtime(
            installer,
            "workflow checkpoint drop panic",
        );
        let operation_resources = crate::domain_computation::provider_session::admitted_yield_plan(
            "workflow-checkpoint-drop-panic",
            8,
        );
        let stage_resources = admitted_plan_with_graph_support(
            "workflow-checkpoint-drop-panic:stage",
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
                    "workflow-checkpoint-drop-panic",
                ),
            )
            .unwrap();
        let paused = match active.advance(execution) {
            WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("workflow drop-panic provider did not pause"),
        };
        let yielded = match paused.yield_run() {
            crate::domain_computation::WorthQueryWorkflowYieldOutcome::Yielded(yielded) => yielded,
            _ => panic!("workflow drop-panic checkpoint did not yield"),
        };
        let recovery = match yielded.cleanup() {
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::RecoveryRequired(
                recovery,
            ) => recovery,
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::Complete(_) => {
                panic!("checkpoint destructor panic claimed complete cleanup")
            }
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::Pending(_) => {
                panic!("artifact-free checkpoint destructor panic claimed pending artifacts")
            }
        };
        assert_eq!(
            recovery.inspection().checkpoint().release_disposition(),
            crate::domain_computation::WorthQueryProviderCheckpointReleaseDisposition::Panicked
        );
        assert!(recovery.inspection().resources_released());
        assert_eq!(recovery.inspection().released_reservation_count(), 3);
    });
}
