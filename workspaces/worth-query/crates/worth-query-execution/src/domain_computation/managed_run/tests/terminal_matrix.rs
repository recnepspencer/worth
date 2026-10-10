use super::*;

#[derive(Clone, Copy)]
enum NonCompletionTerminal {
    Cancelled,
    TimedOut,
    Exhausted,
    Degraded,
    Failed,
}

impl NonCompletionTerminal {
    const ALL: [Self; 5] = [
        Self::Cancelled,
        Self::TimedOut,
        Self::Exhausted,
        Self::Degraded,
        Self::Failed,
    ];

    fn expected_kind(self) -> WorthQueryManagedRunTerminalKind {
        match self {
            Self::Cancelled => WorthQueryManagedRunTerminalKind::Cancelled,
            Self::TimedOut => WorthQueryManagedRunTerminalKind::TimedOut,
            Self::Exhausted => WorthQueryManagedRunTerminalKind::Exhausted,
            Self::Degraded => WorthQueryManagedRunTerminalKind::Degraded,
            Self::Failed => WorthQueryManagedRunTerminalKind::Failed,
        }
    }
}

#[test]
fn direct_noncompletion_terminals_cancel_signal_and_release_every_owner() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    for terminal in NonCompletionTerminal::ALL {
        let running = running_direct("direct-terminal-matrix", resource_request);
        let terminal_authority = running.terminal(terminal.expected_kind());
        assert_eq!(terminal_authority.kind(), terminal.expected_kind());
        let cleanup = terminal_authority
            .cleanup()
            .expect("owner-thread terminal cleanup should succeed");
        assert_eq!(cleanup.inspection().terminal(), terminal.expected_kind());
        assert_eq!(
            cleanup.inspection().disposition(),
            WorthQueryManagedRunCleanupDisposition::CleanupComplete
        );
        assert!(cleanup.inspection().resources_released());
        assert_eq!(cleanup.inspection().released_reservation_count(), 1);
    }
}

#[test]
fn workflow_noncompletion_terminals_cancel_signal_and_release_every_owner() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    for terminal in NonCompletionTerminal::ALL {
        let running = running_workflow("workflow-terminal-matrix", resource_request);
        let terminal_authority = running.terminal(terminal.expected_kind());
        assert_eq!(terminal_authority.kind(), terminal.expected_kind());
        let cleanup = match terminal_authority.cleanup() {
            WorthQueryWorkflowRunCleanupOutcome::Complete(receipt) => receipt,
            WorthQueryWorkflowRunCleanupOutcome::Pending(_) => {
                panic!("workflow without artifact production retained owners")
            }
            WorthQueryWorkflowRunCleanupOutcome::RecoveryRequired(failure) => {
                panic!("owner-thread workflow cleanup failed: {failure:?}")
            }
        };
        let inspection = cleanup.inspection();
        assert_eq!(inspection.terminal(), terminal.expected_kind());
        assert_eq!(
            inspection.disposition(),
            WorthQueryManagedRunCleanupDisposition::CleanupComplete
        );
        assert!(inspection.resources_released());
        assert_eq!(inspection.released_reservation_count(), 2);
    }
}

fn running_direct(
    label: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRunningDirectRun {
    let runtime = query_runtime();
    let plan = admitted_plan(label, 8);
    let operation = direct_authority(&runtime, &plan);
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("terminal-matrix direct resources should reserve");
    let lower = causal_fixture::managed_admission_context();
    runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request(), resource_request)
        .expect("terminal-matrix direct run should admit")
        .start()
}

fn running_workflow(
    label: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::domain_computation::WorthQueryRunningWorkflowRun {
    let runtime = query_runtime();
    let operation_resources = admitted_plan(label, 8);
    let stage_resources = admitted_plan(&format!("{label}:stage"), 4);
    let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
        operation_resources,
        BTreeMap::from([("stage".to_owned(), stage_resources)]),
    );
    let operation = workflow_authority(&runtime, &resources);
    let attempt = runtime
        .start_workflow_resource_attempt(&operation, resources)
        .expect("terminal-matrix workflow resources should reserve");
    let lower = causal_fixture::managed_admission_context();
    runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_workflow(&operation, attempt, lower.read_request(), resource_request)
        .expect("terminal-matrix workflow run should admit")
        .start()
        .expect("terminal-matrix workflow should start")
}
