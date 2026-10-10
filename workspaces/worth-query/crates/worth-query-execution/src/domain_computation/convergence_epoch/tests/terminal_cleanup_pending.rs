use super::fixture::{
    workflow_yield_pending_admission_fixture, WorkflowAdmissionFixture, WORKFLOW_STAGE,
};
use crate::domain_computation::{
    WorthQueryConverged, WorthQueryGraphProviderCallKind, WorthQueryManagedGraphCallRequest,
    WorthQueryMoveOnlyArtifactHandle, WorthQueryWorkflowConvergenceCleanupOutcome,
    WorthQueryWorkflowConvergenceCleanupPending, WorthQueryWorkflowConvergenceCleanupReceipt,
    WorthQueryWorkflowConvergenceIterationOutcome, WorthQueryWorkflowConvergenceStepOutcome,
    WorthQueryWorkflowConvergenceTerminal,
};

#[test]
fn workflow_terminal_cleanup_pending_retries_the_same_epoch_after_artifact_release() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (terminal, artifact_receiver) =
            workflow_terminal_with_live_cleanup_artifact(execution, resource_request);
        let artifact = artifact_receiver
            .recv()
            .expect("production provider step must issue the move-only artifact handle");
        let borrowed = artifact
            .borrow("terminal cleanup pending proof")
            .expect("installed candidate contract must admit shared observation");
        let identity = terminal.identity().to_owned();
        let report_id = terminal
            .latest_report()
            .unwrap()
            .evidence_identity()
            .to_owned();
        let occurrence_id = terminal.incumbents()[0].occurrence_identity().to_owned();

        let pending = match terminal.cleanup() {
            WorthQueryWorkflowConvergenceCleanupOutcome::Pending(pending) => pending,
            _ => panic!("live artifact handle and borrow must keep terminal cleanup pending"),
        };
        assert_pending_epoch(&pending, &identity, &report_id, &occurrence_id);

        drop(borrowed);
        drop(artifact);
        let receipt = match pending.retry() {
            WorthQueryWorkflowConvergenceCleanupOutcome::Complete(receipt) => receipt,
            _ => panic!("released artifact ownership must permit terminal cleanup completion"),
        };
        assert_completed_epoch(&receipt, &identity, &report_id, &occurrence_id);
    });
}

fn workflow_terminal_with_live_cleanup_artifact(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    WorthQueryWorkflowConvergenceTerminal<WorthQueryConverged>,
    Receiver<WorthQueryMoveOnlyArtifactHandle>,
) {
    let (fixture, artifact_receiver) = workflow_yield_pending_admission_fixture(resource_request);
    let WorkflowAdmissionFixture {
        runtime,
        operation,
        contract,
        managed,
        graph,
        bridge: _,
    } = fixture;
    let epoch = runtime
        .admit_workflow_convergence_epoch(&operation, contract, managed, graph)
        .unwrap_or_else(|_| panic!("artifact workflow authorities must admit"))
        .start()
        .unwrap_or_else(|_| panic!("artifact workflow convergence must start"));
    let started = epoch
        .begin_stage_iteration(
            execution,
            WORKFLOW_STAGE,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "workflow-terminal-cleanup-pending",
            ),
        )
        .unwrap_or_else(|_| panic!("artifact workflow iteration must start"));
    let paused = match started.advance(execution) {
        WorthQueryWorkflowConvergenceStepOutcome::Continue(paused) => paused,
        _ => panic!("artifact workflow must expose its retained safe point"),
    };
    let terminal = match paused.advance(execution) {
        WorthQueryWorkflowConvergenceStepOutcome::Completed(
            WorthQueryWorkflowConvergenceIterationOutcome::Converged(terminal),
        ) => terminal,
        _ => panic!("second artifact workflow step must converge"),
    };
    (terminal, artifact_receiver)
}

fn assert_pending_epoch(
    pending: &WorthQueryWorkflowConvergenceCleanupPending<WorthQueryConverged>,
    identity: &str,
    report_identity: &str,
    occurrence_identity: &str,
) {
    assert_eq!(pending.identity(), identity);
    assert_eq!(pending.counters().cleanup_attempt_count(), 1);
    assert_eq!(pending.counters().cleanup_completion_count(), 0);
    assert_eq!(
        pending.latest_report().unwrap().evidence_identity(),
        report_identity
    );
    assert_eq!(
        pending.incumbents()[0].occurrence_identity(),
        occurrence_identity
    );
}

fn assert_completed_epoch(
    receipt: &WorthQueryWorkflowConvergenceCleanupReceipt<WorthQueryConverged>,
    identity: &str,
    report_identity: &str,
    occurrence_identity: &str,
) {
    assert_eq!(receipt.identity(), identity);
    assert_eq!(receipt.counters().cleanup_attempt_count(), 2);
    assert_eq!(receipt.counters().cleanup_completion_count(), 1);
    assert_eq!(
        receipt.latest_report().unwrap().evidence_identity(),
        report_identity
    );
    assert_eq!(
        receipt.incumbents()[0].occurrence_identity(),
        occurrence_identity
    );
}
use std::sync::mpsc::Receiver;
