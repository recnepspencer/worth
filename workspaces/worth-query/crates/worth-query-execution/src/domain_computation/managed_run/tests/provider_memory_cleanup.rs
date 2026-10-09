use std::sync::Mutex;

use super::*;

struct EscapingMemoryProvider {
    retained: Arc<Mutex<Vec<WorthQueryGraphProviderRetainedMemory>>>,
}

struct EscapingMemoryExecution {
    retained: Arc<Mutex<Vec<WorthQueryGraphProviderRetainedMemory>>>,
}

impl WorthQueryGraphProviderExecution for EscapingMemoryExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        step.perform_work_unit(|| Ok(()))?;
        self.retained
            .lock()
            .expect("retained-memory fixture lock should remain available")
            .push(step.retain_bytes(32).map_err(step_failure)?);
        WorthQueryGraphProviderStepDisposition::complete("escaping-memory")
            .map_err(WorthQueryGraphProviderFailure::new)
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for EscapingMemoryProvider {
    type Execution = EscapingMemoryExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support(
            "escaping-provider-memory",
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
            EscapingMemoryExecution {
                retained: Arc::clone(&self.retained),
            },
        )
    }
}

struct EscapingStartProvider {
    retained: Arc<Mutex<Vec<WorthQueryGraphProviderRetainedMemory>>>,
}

struct NeverStartedExecution;

impl WorthQueryGraphProviderExecution for NeverStartedExecution {
    fn advance(
        &mut self,
        _step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        Err(WorthQueryGraphProviderFailure::new(
            "rejected provider must never advance",
        ))
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for EscapingStartProvider {
    type Execution = NeverStartedExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support(
            "escaping-start-memory",
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
        self.retained
            .lock()
            .expect("start-memory fixture lock should remain available")
            .push(start.retain_bytes(24).map_err(step_failure)?);
        Err(WorthQueryGraphProviderFailure::new(
            "provider rejected after escaping governed start memory",
        ))
    }
}

#[test]
fn multiple_provider_calls_retain_every_live_arena_until_cleanup() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let retained = Arc::new(Mutex::new(Vec::new()));
        let (running, graph) = managed_graph_run_with_provider(
            WorthQueryOperationGraphAccess::Observe,
            EscapingMemoryProvider {
                retained: Arc::clone(&retained),
            },
        );
        let running = complete_call(execution, running, &graph, "first-memory-call");
        let running = complete_call(execution, running, &graph, "second-memory-call");
        let terminal = running
            .completed()
            .expect("receipt-bound calls should complete logically");

        assert!(terminal.provider_work().provider_retained_bytes() >= 64);
        let first_pending = terminal
            .cleanup()
            .expect_err("two escaped arenas must keep cleanup pending");
        assert_eq!(
            first_pending.failure_kind(),
            WorthQueryManagedRunCleanupFailureKind::ProviderMemoryRetained
        );
        assert_eq!(
            first_pending.disposition(),
            WorthQueryManagedRunCleanupDisposition::CleanupPending
        );
        let first_bytes = first_pending.provider_retained_bytes();

        let first = retained
            .lock()
            .expect("retained-memory fixture lock should remain available")
            .remove(0);
        drop(first);
        let second_pending = first_pending
            .retry()
            .expect_err("the second escaped arena must remain pending");
        assert!(second_pending.provider_retained_bytes() < first_bytes);
        retained
            .lock()
            .expect("retained-memory fixture lock should remain available")
            .clear();

        let cleanup = second_pending
            .retry()
            .expect("released arenas should permit exact cleanup");
        assert_eq!(
            cleanup
                .inspection()
                .provider_work()
                .provider_retained_bytes(),
            0
        );
        assert!(cleanup.inspection().provider_work().peak_retained_bytes() >= first_bytes);
        assert_eq!(cleanup.inspection().released_reservation_count(), 2);
    });
}

#[test]
fn provider_start_failure_preserves_escaped_arena_recovery() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let retained = Arc::new(Mutex::new(Vec::new()));
        let (running, graph) = managed_graph_run_with_provider(
            WorthQueryOperationGraphAccess::Observe,
            EscapingStartProvider {
                retained: Arc::clone(&retained),
            },
        );
        let failure = match running.begin_graph_execution(
            execution,
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "escaping-start-memory",
            ),
        ) {
            Err(failure) => failure,
            Ok(_) => panic!("provider start should reject"),
        };
        assert_eq!(
        failure.kind(),
        crate::domain_computation::WorthQueryDirectGraphExecutionStartFailureKind::ProviderStartMemoryLeaked
    );
        assert!(failure.provider_retained_bytes() >= 24);
        let terminal = failure
            .into_running()
            .terminate_for_convergence(WorthQueryManagedRunTerminalKind::Failed);
        let pending = terminal
            .cleanup()
            .expect_err("escaped start memory must keep cleanup pending");
        assert_eq!(
            pending.failure_kind(),
            WorthQueryManagedRunCleanupFailureKind::ProviderMemoryRetained
        );

        retained
            .lock()
            .expect("start-memory fixture lock should remain available")
            .clear();
        let cleanup = pending
            .retry()
            .expect("released start memory should permit physical cleanup");
        assert_eq!(
            cleanup
                .inspection()
                .provider_work()
                .provider_retained_bytes(),
            0
        );
        assert_eq!(cleanup.inspection().released_reservation_count(), 2);
    });
}

#[test]
fn workflow_cleanup_retries_each_live_provider_arena_before_completion() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let retained = Arc::new(Mutex::new(Vec::new()));
        let installer = WorthQueryExecutionRuntimeInstaller::new();
        let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            EscapingMemoryProvider {
                retained: Arc::clone(&retained),
            },
        ),
    );
        let provider_support = provider_anchor.resource_support().clone();
        let graph = super::workflow_provider_steps::installed_graph(
            &installer,
            "workflow-cleanup-memory-graph",
            provider_anchor,
        );
        let runtime =
            super::workflow_provider_steps::installed_runtime(installer, "workflow cleanup memory");
        let operation_resources = crate::domain_computation::provider_session::admitted_yield_plan(
            "workflow-cleanup-memory",
            8,
        );
        let stage_resources = admitted_plan_with_graph_support(
            "workflow-cleanup-memory:stage",
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
        let running =
            complete_workflow_memory_call(execution, running, &graph, "first-workflow-memory");
        let running =
            complete_workflow_memory_call(execution, running, &graph, "second-workflow-memory");
        let terminal = running
            .completed()
            .expect("receipt-bound workflow should complete");
        let first_pending = match terminal.cleanup() {
            WorthQueryWorkflowRunCleanupOutcome::Pending(pending) => pending,
            _ => panic!("two escaped workflow arenas must keep cleanup pending"),
        };
        let first_bytes = first_pending.provider_retained_bytes();
        assert!(first_bytes >= 64);

        drop(
            retained
                .lock()
                .expect("workflow retained-memory lock should remain available")
                .remove(0),
        );
        let second_pending = match first_pending.retry() {
            WorthQueryWorkflowRunCleanupOutcome::Pending(pending) => pending,
            _ => panic!("second workflow arena must retain retry authority"),
        };
        assert!(second_pending.provider_retained_bytes() < first_bytes);
        retained
            .lock()
            .expect("workflow retained-memory lock should remain available")
            .clear();
        let cleanup = match second_pending.retry() {
            WorthQueryWorkflowRunCleanupOutcome::Complete(receipt) => receipt,
            _ => panic!("released workflow arenas must permit cleanup completion"),
        };
        assert_eq!(
            cleanup
                .inspection()
                .provider_work()
                .provider_retained_bytes(),
            0
        );
        assert!(cleanup.inspection().resources_released());
        assert_eq!(cleanup.inspection().released_reservation_count(), 3);
    });
}

fn complete_call(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    running: WorthQueryRunningDirectRun,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
    scope: &str,
) -> WorthQueryRunningDirectRun {
    let active = running
        .begin_graph_execution(
            execution,
            graph,
            WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, scope),
        )
        .expect("provider call should start");
    match active.advance(execution) {
        WorthQueryDirectGraphStepOutcome::Completed(completion) => completion.into_running(),
        _ => panic!("escaping-memory provider did not complete"),
    }
}

fn complete_workflow_memory_call(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    running: crate::domain_computation::WorthQueryRunningWorkflowRun,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
    scope: &str,
) -> crate::domain_computation::WorthQueryRunningWorkflowRun {
    let active = running
        .begin_stage_graph_execution(
            execution,
            "stage",
            graph,
            WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, scope),
        )
        .expect("workflow memory provider call should start");
    match active.advance(execution) {
        WorthQueryWorkflowGraphStepOutcome::Completed(completion) => completion.into_running(),
        _ => panic!("workflow memory provider did not complete"),
    }
}

fn step_failure(
    denial: crate::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
