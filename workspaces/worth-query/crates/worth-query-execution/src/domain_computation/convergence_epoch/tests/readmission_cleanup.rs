use super::fixture::{
    direct_admission_fixture, workflow_admission_fixture, DirectAdmissionFixture,
    FixtureDisposition, WorkflowAdmissionFixture, WORKFLOW_STAGE,
};
use crate::domain_computation::{
    WorthQueryDirectConvergenceReadmissionCleanupOutcome,
    WorthQueryDirectConvergenceReadmissionCleanupPending,
    WorthQueryDirectConvergenceReadmissionCleanupRequired,
    WorthQueryDirectConvergenceReadmissionOutcome,
    WorthQueryDirectConvergenceReadmissionRecoveryRequired, WorthQueryDirectConvergenceStepOutcome,
    WorthQueryDirectConvergenceYieldOutcome, WorthQueryGraphProviderCallKind,
    WorthQueryManagedGraphCallRequest, WorthQueryManagedRunCleanupDisposition,
    WorthQueryReadmissionEvidence, WorthQueryWorkflowConvergenceReadmissionCleanupOutcome,
    WorthQueryWorkflowConvergenceReadmissionCleanupPending,
    WorthQueryWorkflowConvergenceReadmissionCleanupRequired,
    WorthQueryWorkflowConvergenceReadmissionOutcome,
    WorthQueryWorkflowConvergenceReadmissionRecoveryRequired,
    WorthQueryWorkflowConvergenceStepOutcome, WorthQueryWorkflowConvergenceYieldOutcome,
};

struct RecoveryCleanup<Cleanup> {
    cleanup: Cleanup,
    epoch_identity: String,
    evidence: WorthQueryReadmissionEvidence,
}

struct DirectPendingRecovery {
    pending: WorthQueryDirectConvergenceReadmissionCleanupPending,
    epoch_identity: String,
    evidence: WorthQueryReadmissionEvidence,
}

struct WorkflowPendingRecovery {
    pending: WorthQueryWorkflowConvergenceReadmissionCleanupPending,
    epoch_identity: String,
    evidence: WorthQueryReadmissionEvidence,
}

#[test]
fn direct_readmission_cleanup_pending_retry_preserves_the_exact_epoch() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        complete_direct_pending(direct_pending(direct_cleanup_required(
            execution,
            resource_request,
        )));
    });
}

#[test]
fn workflow_readmission_cleanup_pending_retry_preserves_the_exact_epoch() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        complete_workflow_pending(workflow_pending(workflow_cleanup_required(
            execution,
            resource_request,
        )));
    });
}

#[test]
fn same_scope_and_stage_terminal_recovery_peers_keep_their_cleanup_owners() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let direct_a = direct_pending(direct_cleanup_required(execution, resource_request));
        let direct_b = direct_pending(direct_cleanup_required(execution, resource_request));
        assert_ne!(direct_a.epoch_identity, direct_b.epoch_identity);
        complete_direct_pending(direct_a);
        complete_direct_pending(direct_b);

        let workflow_a = workflow_pending(workflow_cleanup_required(execution, resource_request));
        let workflow_b = workflow_pending(workflow_cleanup_required(execution, resource_request));
        assert_ne!(workflow_a.epoch_identity, workflow_b.epoch_identity);
        complete_workflow_pending(workflow_a);
        complete_workflow_pending(workflow_b);
    });
}

fn direct_cleanup_required(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> RecoveryCleanup<WorthQueryDirectConvergenceReadmissionCleanupRequired> {
    let active_request = execution;

    let DirectAdmissionFixture {
        runtime,
        operation,
        alternate_basis_operation: _,
        contract,
        managed,
        graph,
        bridge,
    } = direct_admission_fixture(FixtureDisposition::YieldThenRestorePanic, resource_request);
    let epoch = runtime
        .admit_direct_convergence_epoch(&operation, contract, managed, graph)
        .unwrap_or_else(|_| panic!("direct cleanup fixture authorities must admit"))
        .start();
    let started = epoch
        .begin_iteration(execution, call("direct-readmission-cleanup"))
        .unwrap_or_else(|_| panic!("direct cleanup fixture iteration must start"));
    let paused = match started.advance(execution) {
        WorthQueryDirectConvergenceStepOutcome::Continue(paused) => paused,
        _ => panic!("direct cleanup fixture must reach a yield safe point"),
    };
    let yielded = match paused.yield_iteration() {
        WorthQueryDirectConvergenceYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("direct cleanup fixture must yield"),
    };
    let epoch_identity = yielded.epoch_identity().to_owned();
    match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
        WorthQueryDirectConvergenceReadmissionOutcome::RecoveryRequired(
            WorthQueryDirectConvergenceReadmissionRecoveryRequired::TerminalCleanup(recovery),
        ) => RecoveryCleanup {
            evidence: recovery.readmission_evidence(),
            cleanup: recovery.into_cleanup(),
            epoch_identity,
        },
        _ => panic!("direct restore panic must require terminal cleanup"),
    }
}

fn workflow_cleanup_required(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> RecoveryCleanup<WorthQueryWorkflowConvergenceReadmissionCleanupRequired> {
    let active_request = execution;

    let WorkflowAdmissionFixture {
        runtime,
        operation,
        contract,
        managed,
        graph,
        bridge,
    } = workflow_admission_fixture(FixtureDisposition::YieldThenRestorePanic, resource_request);
    let admitted = runtime
        .admit_workflow_convergence_epoch(&operation, contract, managed, graph)
        .unwrap_or_else(|_| panic!("workflow cleanup fixture authorities must admit"));
    let epoch = admitted
        .start()
        .unwrap_or_else(|_| panic!("workflow cleanup fixture must start"));
    let started = epoch
        .begin_stage_iteration(
            execution,
            WORKFLOW_STAGE,
            call("workflow-readmission-cleanup"),
        )
        .unwrap_or_else(|_| panic!("workflow cleanup fixture iteration must start"));
    let paused = match started.advance(execution) {
        WorthQueryWorkflowConvergenceStepOutcome::Continue(paused) => paused,
        _ => panic!("workflow cleanup fixture must reach a yield safe point"),
    };
    let yielded = match paused.yield_iteration() {
        WorthQueryWorkflowConvergenceYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("workflow cleanup fixture must yield"),
    };
    let epoch_identity = yielded.epoch_identity().to_owned();
    match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
        WorthQueryWorkflowConvergenceReadmissionOutcome::RecoveryRequired(
            WorthQueryWorkflowConvergenceReadmissionRecoveryRequired::TerminalCleanup(recovery),
        ) => RecoveryCleanup {
            evidence: recovery.readmission_evidence(),
            cleanup: recovery.into_cleanup(),
            epoch_identity,
        },
        _ => panic!("workflow restore panic must require terminal cleanup"),
    }
}

fn direct_pending(
    recovery: RecoveryCleanup<WorthQueryDirectConvergenceReadmissionCleanupRequired>,
) -> DirectPendingRecovery {
    let RecoveryCleanup {
        cleanup,
        epoch_identity,
        evidence,
    } = recovery;
    let pending = std::thread::spawn(move || match cleanup.finish() {
        WorthQueryDirectConvergenceReadmissionCleanupOutcome::Pending(pending) => pending,
        _ => panic!("foreign-thread direct cleanup must retain retry authority"),
    })
    .join()
    .expect("direct cleanup fault thread must return its pending authority");
    assert_eq!(pending.epoch_identity(), epoch_identity);
    let pending_evidence = pending.readmission_evidence();
    assert_cleanup_evidence_progression(evidence, pending_evidence);
    assert_pending_state(pending.counters(), pending_evidence);
    DirectPendingRecovery {
        pending,
        epoch_identity,
        evidence: pending_evidence,
    }
}

fn workflow_pending(
    recovery: RecoveryCleanup<WorthQueryWorkflowConvergenceReadmissionCleanupRequired>,
) -> WorkflowPendingRecovery {
    let RecoveryCleanup {
        cleanup,
        epoch_identity,
        evidence,
    } = recovery;
    let pending = std::thread::spawn(move || match cleanup.finish() {
        WorthQueryWorkflowConvergenceReadmissionCleanupOutcome::Pending(pending) => pending,
        _ => panic!("foreign-thread workflow cleanup must retain retry authority"),
    })
    .join()
    .expect("workflow cleanup fault thread must return its pending authority");
    assert_eq!(pending.epoch_identity(), epoch_identity);
    let pending_evidence = pending.readmission_evidence();
    assert_cleanup_evidence_progression(evidence, pending_evidence);
    assert_pending_state(pending.counters(), pending_evidence);
    WorkflowPendingRecovery {
        pending,
        epoch_identity,
        evidence: pending_evidence,
    }
}

fn assert_cleanup_evidence_progression(
    recovery: WorthQueryReadmissionEvidence,
    cleanup: WorthQueryReadmissionEvidence,
) {
    assert_eq!(cleanup.query_counters(), recovery.query_counters());
    let recovery_bridge = recovery
        .bridge_counters()
        .expect("terminal recovery must retain Bridge attempt evidence");
    let cleanup_bridge = cleanup
        .bridge_counters()
        .expect("cleanup must retain Bridge abort evidence");
    assert_eq!(recovery_bridge.abort_count(), 0);
    assert_eq!(cleanup_bridge.abort_count(), 1);
    assert_eq!(cleanup_bridge.commit_count(), 0);
}

fn assert_pending_state(
    counters: &crate::domain_computation::WorthQueryConvergenceEpochCounters,
    evidence: WorthQueryReadmissionEvidence,
) {
    assert_eq!(counters.yield_count(), 1);
    assert_eq!(counters.cleanup_attempt_count(), 1);
    assert_eq!(counters.cleanup_completion_count(), 0);
    assert_eq!(
        evidence.query_counters().provider_restore_attempt_count(),
        1
    );
    assert!(evidence.bridge_counters().is_some());
}

fn complete_direct_pending(recovery: DirectPendingRecovery) {
    let receipt = match recovery.pending.retry() {
        WorthQueryDirectConvergenceReadmissionCleanupOutcome::Complete(receipt) => receipt,
        _ => panic!("owner-thread direct retry must complete cleanup"),
    };
    assert_eq!(receipt.epoch_identity(), recovery.epoch_identity);
    assert_eq!(receipt.readmission_evidence(), recovery.evidence);
    assert_eq!(receipt.counters().cleanup_attempt_count(), 2);
    assert_eq!(receipt.counters().cleanup_completion_count(), 1);
    assert_eq!(
        receipt.disposition(),
        WorthQueryManagedRunCleanupDisposition::CleanupComplete
    );
}

fn complete_workflow_pending(recovery: WorkflowPendingRecovery) {
    let receipt = match recovery.pending.retry() {
        WorthQueryWorkflowConvergenceReadmissionCleanupOutcome::Complete(receipt) => receipt,
        _ => panic!("owner-thread workflow retry must complete cleanup"),
    };
    assert_eq!(receipt.epoch_identity(), recovery.epoch_identity);
    assert_eq!(receipt.readmission_evidence(), recovery.evidence);
    assert_eq!(receipt.counters().cleanup_attempt_count(), 2);
    assert_eq!(receipt.counters().cleanup_completion_count(), 1);
    assert_eq!(
        receipt.disposition(),
        WorthQueryManagedRunCleanupDisposition::CleanupComplete
    );
}

fn call(identity: &str) -> WorthQueryManagedGraphCallRequest {
    WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, identity)
}
