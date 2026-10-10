use super::yield_fixture::YieldProvider;
use super::*;
use worth_runtime_bridge::facade::BridgeExecutionBasisSignalTerminal;

#[test]
fn direct_yield_retains_exact_authorities_and_releases_them_explicitly() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (running, graph, _bridge) = managed_graph_run_with_provider_and_bridge(
            WorthQueryOperationGraphAccess::Observe,
            YieldProvider::installed(5),
            resource_request,
        );
        let logical_run_identity = running.logical_run_identity().to_owned();
        let attempt_identity = running.identity().to_owned();
        let active = running
            .begin_graph_execution(
                execution,
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "direct-yield",
                ),
            )
            .expect("yield provider should begin");
        let paused = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("yield provider did not reach its declared safe point"),
        };

        let yielded = match paused.yield_run() {
            crate::domain_computation::WorthQueryDirectYieldOutcome::Yielded(yielded) => yielded,
            _ => panic!("eligible direct run did not yield"),
        };
        assert_eq!(
            yielded.inspection().logical_run_identity(),
            logical_run_identity
        );
        assert_eq!(
            yielded.inspection().yielded_attempt_identity(),
            attempt_identity
        );
        assert_ne!(
            yielded.inspection().logical_run_identity(),
            yielded.inspection().yielded_attempt_identity()
        );
        assert_eq!(yielded.inspection().checkpoint().retained_bytes(), 5);
        assert!(yielded.inspection().checkpoint().provider_generation() > 0);
        assert_eq!(
            yielded.inspection().retained_capacity_reservation_count(),
            2
        );
        assert_eq!(
            yielded
                .inspection()
                .provider_work()
                .interrupted_call_count(),
            1
        );
        assert_eq!(
            yielded.inspection().provider_work().completed_work_units(),
            2
        );

        let cleanup = complete_direct_yield_cleanup(yielded);
        assert_eq!(cleanup.logical_run_identity(), logical_run_identity);
        assert_eq!(cleanup.yielded_attempt_identity(), attempt_identity);
        assert!(cleanup.resources_released());
        assert_eq!(cleanup.released_reservation_count(), 2);
        assert_eq!(cleanup.checkpoint().unwrap().retained_bytes(), 5);
    });
}

#[test]
fn direct_yield_denials_preserve_the_paused_execution_authority() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (running, graph) = managed_graph_run_with_provider(
            WorthQueryOperationGraphAccess::Observe,
            YieldProvider::without_installed_yield(),
            resource_request,
        );
        let active = running
            .begin_graph_execution(
                execution,
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "direct-yield-denied",
                ),
            )
            .unwrap();
        let paused = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("provider did not pause"),
        };
        let denied = match paused.yield_run() {
            crate::domain_computation::WorthQueryDirectYieldOutcome::Denied(denied) => denied,
            _ => panic!("non-yieldable contract minted yielded authority"),
        };
        assert_eq!(
            denied.kind(),
            crate::domain_computation::WorthQueryDirectYieldDenialKind::YieldNotInstalled
        );
        let completion = match denied.into_paused().advance(execution) {
            WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
            _ => panic!("yield denial consumed the paused execution"),
        };
        let terminal = completion.into_running().completed().unwrap();
        assert!(terminal.cleanup().is_ok());
    });
}

#[test]
fn checkpoint_claim_is_required_even_when_yield_is_installed() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (running, graph) = managed_graph_run_with_provider(
            WorthQueryOperationGraphAccess::Observe,
            YieldProvider::without_checkpoint_evidence(),
            resource_request,
        );
        let active = running
            .begin_graph_execution(
                execution,
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "direct-no-checkpoint",
                ),
            )
            .unwrap();
        let paused = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("provider did not pause"),
        };
        let denied = match paused.yield_run() {
            crate::domain_computation::WorthQueryDirectYieldOutcome::Denied(denied) => denied,
            _ => panic!("missing checkpoint evidence permitted yield"),
        };
        assert_eq!(
            denied.kind(),
            crate::domain_computation::WorthQueryDirectYieldDenialKind::CheckpointUnavailable
        );
        let completion = match denied.into_paused().advance(execution) {
            WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
            _ => panic!("checkpoint denial consumed the execution"),
        };
        assert!(completion
            .into_running()
            .completed()
            .unwrap()
            .cleanup()
            .is_ok());
    });
}

#[test]
fn suspension_failure_terminalizes_signal_but_preserves_cleanup_authority() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (running, graph) = managed_graph_run_with_provider(
            WorthQueryOperationGraphAccess::Observe,
            YieldProvider::suspension_failure(),
            resource_request,
        );
        let active = running
            .begin_graph_execution(
                execution,
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "direct-suspend-failure",
                ),
            )
            .unwrap();
        let paused = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("provider did not pause"),
        };
        let recovery = match paused.yield_run() {
            crate::domain_computation::WorthQueryDirectYieldOutcome::RecoveryRequired(recovery) => {
                recovery
            }
            _ => panic!("suspension failure did not return recovery authority"),
        };
        assert!(!recovery.running_attempt_recoverable());
        assert_eq!(
        recovery.kind(),
        crate::domain_computation::WorthQueryYieldRecoveryKind::ProviderCheckpointSuspension(
            crate::domain_computation::WorthQueryProviderCheckpointSuspensionFailureKind::
                ProviderRejected,
        )
    );
        let cleanup = match recovery.cleanup_terminalized() {
            Ok(cleanup) => cleanup,
            Err(_) => panic!("terminalized direct recovery did not release"),
        };
        assert_eq!(
            cleanup.inspection().bridge_signal_terminal(),
            BridgeExecutionBasisSignalTerminal::Cancelled
        );
        assert!(cleanup.inspection().resources_released());
        assert_eq!(cleanup.inspection().released_reservation_count(), 2);
        assert_eq!(
            cleanup.inspection().provider_work().abandoned_call_count(),
            1
        );
    });
}

#[test]
fn suspension_panic_and_oversized_checkpoint_follow_the_same_recovery_lane() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        for (provider, expected_kind) in [
        (
            YieldProvider::suspension_panic(),
            crate::domain_computation::WorthQueryYieldRecoveryKind::ProviderCheckpointSuspension(
                crate::domain_computation::WorthQueryProviderCheckpointSuspensionFailureKind::
                    ProviderPanicked,
            ),
        ),
        (
            YieldProvider::checkpoint_probe_panic(),
            crate::domain_computation::WorthQueryYieldRecoveryKind::ProviderCheckpointSuspension(
                crate::domain_computation::WorthQueryProviderCheckpointSuspensionFailureKind::
                    CheckpointRetention(
                        crate::domain_computation::
                            WorthQueryProviderCheckpointRetentionFailureKind::
                                RetainedByteProbePanicked,
                    ),
            ),
        ),
        (
            YieldProvider::checkpoint_probe_and_drop_panic(),
            crate::domain_computation::WorthQueryYieldRecoveryKind::ProviderCheckpointSuspension(
                crate::domain_computation::WorthQueryProviderCheckpointSuspensionFailureKind::
                    CheckpointRetention(
                        crate::domain_computation::
                            WorthQueryProviderCheckpointRetentionFailureKind::
                                RetainedByteProbePanicked,
                    ),
            ),
        ),
        (
            YieldProvider::checkpoint_memory_mismatch(3_000, 4_097, false),
            crate::domain_computation::WorthQueryYieldRecoveryKind::ProviderCheckpointSuspension(
                crate::domain_computation::WorthQueryProviderCheckpointSuspensionFailureKind::
                    CheckpointMemoryMismatch,
            ),
        ),
    ] {
        let (running, graph) =
            managed_graph_run_with_provider(WorthQueryOperationGraphAccess::Observe, provider, resource_request);
        let active = running
            .begin_graph_execution(execution,
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "direct-yield-adversarial",
                ),
            )
            .unwrap();
        let paused = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("provider did not pause"),
        };
        let recovery = match paused.yield_run() {
            crate::domain_computation::WorthQueryDirectYieldOutcome::RecoveryRequired(recovery) => {
                recovery
            }
            _ => panic!("invalid checkpoint transition did not require recovery"),
        };
        assert!(!recovery.running_attempt_recoverable());
        assert_eq!(recovery.kind(), expected_kind);
        match recovery.cleanup_terminalized() {
            Ok(_) => {}
            Err(_) => panic!("terminalized adversarial recovery did not release"),
        }
    }
    });
}

#[test]
fn direct_yield_preserves_exact_applied_effect_evidence() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (running, graph) = managed_graph_effect_run_with_provider(
            YieldProvider::installed_with_partial_effect(5),
            resource_request,
        );
        let active = running
            .begin_graph_execution(
                execution,
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::TouchEffect,
                    "direct-yield-partial-effect",
                ),
            )
            .expect("installed effect provider should begin");
        let paused = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("effect provider did not reach its yield safe point"),
        };
        let yielded = match paused.yield_run() {
            crate::domain_computation::WorthQueryDirectYieldOutcome::Yielded(yielded) => yielded,
            _ => panic!("installed partial-effect posture did not admit yield"),
        };
        assert_eq!(
            yielded.inspection().provider_work().applied_effect_count(),
            1
        );
        assert_eq!(
            yielded.inspection().provider_work().completed_work_units(),
            3
        );
        let cleanup = complete_direct_yield_cleanup(yielded);
        assert_eq!(cleanup.provider_work().applied_effect_count(), 1);
        assert_eq!(cleanup.provider_work().abandoned_call_count(), 0);
        assert_eq!(cleanup.provider_work().interrupted_call_count(), 1);
    });
}
