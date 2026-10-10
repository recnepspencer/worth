use std::sync::atomic::{AtomicUsize, Ordering};

use worth_runtime_bridge::facade::BridgeManagedExecutionCancellationReason;

use super::*;

struct CountingProvider {
    advances: Arc<AtomicUsize>,
    work_units: Arc<AtomicUsize>,
    exceed_work_budget: bool,
    reject_work: bool,
}

struct CountingExecution {
    advances: Arc<AtomicUsize>,
    work_units: Arc<AtomicUsize>,
    exceed_work_budget: bool,
    reject_work: bool,
}

impl WorthQueryGraphProviderExecution for CountingExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        self.advances.fetch_add(1, Ordering::Relaxed);
        let work = if self.exceed_work_budget { 9 } else { 1 };
        for _ in 0..work {
            if self.reject_work {
                let _ = step.perform_work_unit(|| -> Result<(), WorthQueryGraphProviderFailure> {
                    Err(WorthQueryGraphProviderFailure::new(
                        "provider work unit rejected",
                    ))
                });
                let _ = step.perform_work_unit(|| {
                    self.work_units.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                });
            } else {
                step.perform_work_unit(|| {
                    self.work_units.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                })?;
            }
        }
        WorthQueryGraphProviderStepDisposition::complete("counting-provider")
            .map_err(WorthQueryGraphProviderFailure::new)
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for CountingProvider {
    type Execution = CountingExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support(
            "managed-interruption",
            8,
        )
    }

    fn begin(
        &self,
        _call: &WorthQueryGraphProviderCall,
        start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        WorthQueryGraphProviderFailure,
    > {
        admit_provider_execution(
            start,
            CountingExecution {
                advances: Arc::clone(&self.advances),
                work_units: Arc::clone(&self.work_units),
                exceed_work_budget: self.exceed_work_budget,
                reject_work: self.reject_work,
            },
        )
    }
}

#[test]
fn signal_cancellation_stops_before_the_next_provider_step() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let advances = Arc::new(AtomicUsize::new(0));
        let work_units = Arc::new(AtomicUsize::new(0));
        let (running, graph) = managed_graph_run_with_provider(
            WorthQueryOperationGraphAccess::Observe,
            CountingProvider {
                advances: Arc::clone(&advances),
                work_units,
                exceed_work_budget: false,
                reject_work: false,
            },
            resource_request,
        );
        let active = start_observe(execution, running, &graph, "cancel-before-step");
        active
            .request_cancellation(BridgeManagedExecutionCancellationReason::HostRequested)
            .expect("Bridge should cancel the exact Signal request");
        let terminal = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Cancelled(terminal) => terminal,
            _ => panic!("Signal cancellation did not derive a cancelled terminal"),
        };
        assert_eq!(advances.load(Ordering::Relaxed), 0);
        assert_eq!(
            terminal.provider_work().session_disposition(),
            WorthQueryManagedProviderSessionDisposition::Interrupted
        );
        assert_eq!(terminal.provider_work().interrupted_call_count(), 1);
        let cleanup = terminal.cleanup().expect("cancelled step should clean up");
        assert!(cleanup.inspection().resources_released());
    });
}

#[test]
fn signal_timeout_stops_before_the_next_provider_step() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let advances = Arc::new(AtomicUsize::new(0));
        let work_units = Arc::new(AtomicUsize::new(0));
        let (running, graph, bridge) = managed_graph_run_with_provider_and_bridge(
            WorthQueryOperationGraphAccess::Observe,
            CountingProvider {
                advances: Arc::clone(&advances),
                work_units,
                exceed_work_budget: false,
                reject_work: false,
            },
            resource_request,
        );
        let active = start_observe(execution, running, &graph, "timeout-before-step");
        bridge
            .advance_managed_execution_clock(1)
            .expect("host clock authority should advance Signal time");
        let timeout = active
            .admit_ready_timeout()
            .expect("Bridge should admit the ready exact-request timeout");
        let terminal = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::TimedOut(terminal) => terminal,
            _ => panic!("Signal timeout did not derive a timed-out terminal"),
        };
        assert_eq!(advances.load(Ordering::Relaxed), 0);
        assert_eq!(terminal.provider_work().interrupted_call_count(), 1);
        assert_eq!(
            terminal
                .provider_work()
                .last_safe_point()
                .expect("timeout terminal must retain the consumed Signal safe point")
                .bridge_evidence()
                .timeout_wake_identity(),
            Some(timeout.timeout_wake_identity())
        );
        let cleanup = terminal.cleanup().expect("timed-out step should clean up");
        assert!(cleanup.inspection().resources_released());
    });
}

#[test]
fn signal_rejection_degrades_before_the_next_provider_step() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let advances = Arc::new(AtomicUsize::new(0));
        let work_units = Arc::new(AtomicUsize::new(0));
        let (running, graph) = managed_graph_run_with_provider(
            WorthQueryOperationGraphAccess::Observe,
            CountingProvider {
                advances: Arc::clone(&advances),
                work_units,
                exceed_work_budget: false,
                reject_work: false,
            },
            resource_request,
        );
        let active = start_observe(execution, running, &graph, "reject-before-step");
        active
        .reject_execution(
            worth_runtime_bridge::facade::BridgeManagedExecutionRejectionReason::SemanticFailure,
        )
        .expect("Bridge should reject the exact Signal request");
        let terminal = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Degraded(terminal) => terminal,
            _ => panic!("Signal rejection did not derive a degraded terminal"),
        };
        assert_eq!(advances.load(Ordering::Relaxed), 0);
        assert_eq!(terminal.provider_work().interrupted_call_count(), 1);
        let cleanup = terminal.cleanup().expect("degraded step should clean up");
        assert!(cleanup.inspection().resources_released());
    });
}

#[test]
fn provider_cannot_advance_after_exceeding_the_governed_work_port() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let advances = Arc::new(AtomicUsize::new(0));
        let work_units = Arc::new(AtomicUsize::new(0));
        let (running, graph) = managed_graph_run_with_provider(
            WorthQueryOperationGraphAccess::Observe,
            CountingProvider {
                advances: Arc::clone(&advances),
                work_units: Arc::clone(&work_units),
                exceed_work_budget: true,
                reject_work: false,
            },
            resource_request,
        );
        let active = start_observe(execution, running, &graph, "over-budget-step");
        let terminal = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Failed(terminal) => terminal,
            _ => panic!("over-budget provider advanced the managed run"),
        };
        assert_eq!(advances.load(Ordering::Relaxed), 1);
        assert_eq!(work_units.load(Ordering::Relaxed), 8);
        assert_eq!(terminal.provider_work().completed_work_units(), 8);
        assert_eq!(terminal.provider_work().abandoned_call_count(), 1);
        let cleanup = terminal
            .cleanup()
            .expect("failed step should retain cleanup");
        assert_eq!(
            cleanup.inspection().disposition(),
            WorthQueryManagedRunCleanupDisposition::RecoveryRequired
        );
    });
}

#[test]
fn rejected_work_closure_cannot_claim_a_completed_unit() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let advances = Arc::new(AtomicUsize::new(0));
        let work_units = Arc::new(AtomicUsize::new(0));
        let (running, graph) = managed_graph_run_with_provider(
            WorthQueryOperationGraphAccess::Observe,
            CountingProvider {
                advances: Arc::clone(&advances),
                work_units: Arc::clone(&work_units),
                exceed_work_budget: false,
                reject_work: true,
            },
            resource_request,
        );
        let terminal =
            match start_observe(execution, running, &graph, "rejected-work").advance(execution) {
                WorthQueryDirectGraphStepOutcome::Failed(terminal) => terminal,
                _ => panic!("rejected provider work advanced the managed run"),
            };
        assert_eq!(advances.load(Ordering::Relaxed), 1);
        assert_eq!(work_units.load(Ordering::Relaxed), 0);
        assert_eq!(terminal.provider_work().completed_work_units(), 0);
        let failure = terminal.provider_work().last_step_failure().unwrap();
        assert_eq!(
            failure.invocation(),
            WorthQueryGraphProviderStepInvocationDisposition::Returned
        );
        assert_eq!(failure.invocation_failure_detail(), None);
        assert_eq!(
            failure.latched_provider_failure_detail(),
            Some("provider work unit rejected")
        );
        assert_eq!(failure.governed_denial_kind(), None);
        terminal.cleanup().expect("rejected work should clean up");
    });
}

fn start_observe(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    running: WorthQueryRunningDirectRun,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
    scope: &str,
) -> crate::domain_computation::WorthQueryActiveDirectGraphExecution {
    running
        .begin_graph_execution(
            execution,
            graph,
            WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, scope),
        )
        .expect("exact installed provider should start")
}
