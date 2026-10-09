use super::yield_fixture::YieldProvider;
use super::*;
use worth_runtime_bridge::facade::{
    BridgeExecutionBasisSignalTerminal, BridgeManagedExecutionCancellationReason,
};

#[test]
fn signal_terminalized_after_safe_point_cannot_be_relabelled_as_yielded() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let (running, graph) = managed_graph_run_with_provider(
            WorthQueryOperationGraphAccess::Observe,
            YieldProvider::installed(5),
        );
        let active = running
            .begin_graph_execution(
                execution,
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "direct-yield-signal-race",
                ),
            )
            .unwrap();
        let paused = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("provider did not pause"),
        };
        paused
            .active
            .request_cancellation(BridgeManagedExecutionCancellationReason::HostRequested)
            .expect("host should terminalize the exact Signal attempt");
        let recovery = match paused.yield_run() {
            crate::domain_computation::WorthQueryDirectYieldOutcome::RecoveryRequired(recovery) => {
                recovery
            }
            _ => panic!("pre-terminalized Signal attempt minted yielded authority"),
        };
        assert!(!recovery.running_attempt_recoverable());
        assert_eq!(
            recovery.kind(),
            crate::domain_computation::WorthQueryYieldRecoveryKind::SignalAttemptAlreadyTerminal(
                BridgeExecutionBasisSignalTerminal::Cancelled,
            )
        );
        let cleanup = match recovery.cleanup_terminalized() {
            Ok(cleanup) => cleanup,
            Err(_) => panic!("terminalized Signal race did not preserve cleanup authority"),
        };
        assert!(!cleanup.inspection().bridge_signal_transition_performed());
        assert_eq!(
            cleanup.inspection().bridge_signal_terminal(),
            BridgeExecutionBasisSignalTerminal::Cancelled
        );
        assert_eq!(
            cleanup
                .inspection()
                .provider_work()
                .interrupted_call_count(),
            1
        );
        assert_eq!(
            cleanup.inspection().provider_work().abandoned_call_count(),
            0
        );
    });
}

#[test]
fn timeout_and_rejection_after_safe_point_cannot_mint_yielded_authority() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let (timed_out, timeout_bridge) =
            paused_direct_yield_target(execution, "direct-yield-timeout-race");
        timeout_bridge
            .advance_managed_execution_clock(1)
            .expect("host clock should advance");
        timed_out
            .active
            .admit_ready_timeout()
            .expect("ready timeout should terminalize the exact Signal attempt");
        assert_preterminalized_yield_recovery(
            timed_out,
            BridgeExecutionBasisSignalTerminal::TimedOut,
        );

        let (rejected, _) = paused_direct_yield_target(execution, "direct-yield-rejection-race");
        rejected
        .active
        .reject_execution(
            worth_runtime_bridge::facade::BridgeManagedExecutionRejectionReason::SemanticFailure,
        )
        .expect("rejection should terminalize the exact Signal attempt");
        assert_preterminalized_yield_recovery(
            rejected,
            BridgeExecutionBasisSignalTerminal::Rejected,
        );
    });
}

fn paused_direct_yield_target(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    scope: &str,
) -> (
    crate::domain_computation::WorthQueryPausedDirectGraphExecution,
    RuntimeBridge,
) {
    let (running, graph, bridge) = managed_graph_run_with_provider_and_bridge(
        WorthQueryOperationGraphAccess::Observe,
        YieldProvider::installed(5),
    );
    let active = running
        .begin_graph_execution(
            execution,
            &graph,
            WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, scope),
        )
        .unwrap();
    let paused = match active.advance(execution) {
        WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
        _ => panic!("provider did not pause"),
    };
    (paused, bridge)
}

fn assert_preterminalized_yield_recovery(
    paused: crate::domain_computation::WorthQueryPausedDirectGraphExecution,
    terminal: BridgeExecutionBasisSignalTerminal,
) {
    let recovery = match paused.yield_run() {
        crate::domain_computation::WorthQueryDirectYieldOutcome::RecoveryRequired(recovery) => {
            recovery
        }
        _ => panic!("pre-terminalized Signal attempt minted yielded authority"),
    };
    assert_eq!(
        recovery.kind(),
        crate::domain_computation::WorthQueryYieldRecoveryKind::SignalAttemptAlreadyTerminal(
            terminal,
        )
    );
    let cleanup = match recovery.cleanup_terminalized() {
        Ok(cleanup) => cleanup,
        Err(_) => panic!("pre-terminalized Signal attempt lost cleanup authority"),
    };
    assert!(!cleanup.inspection().bridge_signal_transition_performed());
    assert_eq!(cleanup.inspection().bridge_signal_terminal(), terminal);
    assert_eq!(
        cleanup
            .inspection()
            .provider_work()
            .interrupted_call_count(),
        1
    );
}
