use super::fixture::{direct_epoch_fixture, FixtureDisposition};
use super::terminal_fixture::{converged_terminal, stable_without_proof_terminal};
use crate::domain_computation::{
    WorthQueryDirectConvergenceIterationOutcome, WorthQueryDirectConvergenceStepOutcome,
    WorthQueryGraphProviderCallKind, WorthQueryManagedGraphCallRequest,
};
use worth_runtime_bridge::facade::BridgeManagedExecutionCancellationReason;

#[test]
fn epoch_counters_are_exact_and_isolated_from_unrelated_epochs() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let terminal = converged_terminal(execution);
        let counters = terminal.counters();
        assert_eq!(counters.operation_authority_check_count(), 1);
        assert_eq!(counters.contract_authority_check_count(), 1);
        assert_eq!(counters.managed_run_authority_check_count(), 1);
        assert_eq!(counters.graph_authority_check_count(), 1);
        assert_eq!(counters.iteration_count(), 1);
        assert_eq!(counters.provider_work_unit_count(), 1);
        assert_eq!(counters.comparator_call_count(), 1);
        assert_eq!(counters.progress_check_count(), 1);
        assert_eq!(counters.repeated_state_probe_count(), 1);
        assert_eq!(counters.incumbent_retention_count(), 0);
        assert_eq!(counters.incumbent_replacement_count(), 1);
        assert_eq!(counters.yield_count(), 0);
        assert_eq!(counters.readmission_count(), 0);
        assert_eq!(counters.cleanup_attempt_count(), 0);
        assert_eq!(counters.cleanup_completion_count(), 0);
        let domain_work = terminal
            .latest_report()
            .expect("exact-cost terminal must retain its report")
            .domain_work();
        assert_eq!(domain_work.comparator_call_count(), 1);
        assert_eq!(domain_work.progress_check_count(), 1);
        assert_eq!(domain_work.repeated_state_probe_count(), 1);

        let unrelated = stable_without_proof_terminal(execution);
        assert_eq!(terminal.counters(), counters);
        assert_eq!(unrelated.counters().iteration_count(), 1);

        let cleaned = match terminal.cleanup() {
            Ok(cleanup) => cleanup,
            Err(_) => panic!("exact-cost terminal must retain cleanup authority"),
        };
        assert_eq!(cleaned.counters().cleanup_attempt_count(), 1);
        assert_eq!(cleaned.counters().cleanup_completion_count(), 1);
        if unrelated.cleanup().is_err() {
            panic!("unrelated terminal must retain its own cleanup authority");
        }
    });
}

#[test]
fn late_terminal_reconciles_cumulative_convergence_work_without_double_counting() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let epoch = direct_epoch_fixture(FixtureDisposition::Continue);
        let started = match epoch.begin_iteration(execution, call("completed-before-cancellation"))
        {
            Ok(started) => started,
            Err(_) => panic!("first convergence iteration must start"),
        };
        let epoch = match started.advance(execution) {
            WorthQueryDirectConvergenceStepOutcome::Completed(
                WorthQueryDirectConvergenceIterationOutcome::Continue(epoch),
            ) => epoch,
            _ => panic!("first convergence iteration must remain active"),
        };
        assert_eq!(epoch.counters().provider_work_unit_count(), 1);

        let started = match epoch.begin_iteration(execution, call("cancelled-after-completion")) {
            Ok(started) => started,
            Err(_) => panic!("second convergence iteration must start"),
        };
        started
            .request_cancellation(BridgeManagedExecutionCancellationReason::HostRequested)
            .expect("late cancellation must admit");
        let terminal = match started.advance(execution) {
            WorthQueryDirectConvergenceStepOutcome::Terminal(
                WorthQueryDirectConvergenceIterationOutcome::Cancelled(terminal),
            ) => terminal,
            _ => panic!("late managed cancellation must remain a cancellation terminal"),
        };
        assert_eq!(terminal.counters().iteration_count(), 2);
        assert_eq!(terminal.counters().provider_work_unit_count(), 1);
        assert_eq!(terminal.incumbents().len(), 1);
        assert!(terminal.latest_report().is_some());
        let cleanup = terminal
            .cleanup()
            .unwrap_or_else(|_| panic!("late cancellation must retain cleanup authority"));
        assert_eq!(cleanup.counters().cleanup_attempt_count(), 1);
        assert_eq!(cleanup.counters().cleanup_completion_count(), 1);
    });
}

fn call(identity: &str) -> WorthQueryManagedGraphCallRequest {
    WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, identity)
}
