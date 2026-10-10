use super::fixture::{
    direct_yield_denial_admission_fixture, workflow_yield_denial_admission_fixture,
    DirectAdmissionFixture, FixtureYieldRecoveryProbe, WorkflowAdmissionFixture, WORKFLOW_STAGE,
};
use crate::domain_computation::{
    WorthQueryDeniedDirectConvergenceYield, WorthQueryDeniedWorkflowConvergenceYield,
    WorthQueryDirectConvergenceIterationOutcome, WorthQueryDirectConvergenceStepOutcome,
    WorthQueryDirectConvergenceYieldOutcome, WorthQueryGraphProviderCallKind,
    WorthQueryManagedGraphCallRequest, WorthQueryWorkflowConvergenceCleanupOutcome,
    WorthQueryWorkflowConvergenceIterationOutcome, WorthQueryWorkflowConvergenceStepOutcome,
    WorthQueryWorkflowConvergenceYieldOutcome,
};

struct DirectDeniedPeer {
    denied: WorthQueryDeniedDirectConvergenceYield,
    epoch_identity: String,
    probe: FixtureYieldRecoveryProbe,
}

struct WorkflowDeniedPeer {
    denied: WorthQueryDeniedWorkflowConvergenceYield,
    epoch_identity: String,
    probe: FixtureYieldRecoveryProbe,
}

struct CompletedPeer {
    state_identity: String,
    occurrence_identity: String,
}

#[test]
fn same_scope_direct_denials_resume_their_exact_epochs() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let first = direct_denied_peer(execution, resource_request);
        let second = direct_denied_peer(execution, resource_request);
        assert_ne!(first.epoch_identity, second.epoch_identity);

        let second = complete_direct_peer(execution, second);
        let first = complete_direct_peer(execution, first);
        assert_eq!(first.state_identity, second.state_identity);
        assert_ne!(first.occurrence_identity, second.occurrence_identity);
    });
}

#[test]
fn same_stage_workflow_denials_resume_their_exact_epochs() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let first = workflow_denied_peer(execution, resource_request);
        let second = workflow_denied_peer(execution, resource_request);
        assert_ne!(first.epoch_identity, second.epoch_identity);

        let second = complete_workflow_peer(execution, second);
        let first = complete_workflow_peer(execution, first);
        assert_eq!(first.state_identity, second.state_identity);
        assert_ne!(first.occurrence_identity, second.occurrence_identity);
    });
}

fn direct_denied_peer(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> DirectDeniedPeer {
    let (fixture, probe) = direct_yield_denial_admission_fixture(resource_request);
    let DirectAdmissionFixture {
        runtime,
        operation,
        alternate_basis_operation: _,
        contract,
        managed,
        graph,
        bridge: _,
    } = fixture;
    let epoch = runtime
        .admit_direct_convergence_epoch(&operation, contract, managed, graph)
        .unwrap_or_else(|_| panic!("direct yield-denial peer must admit"))
        .start();
    let started = epoch
        .begin_iteration(execution, call("same-scope-yield-denial"))
        .unwrap_or_else(|_| panic!("direct yield-denial peer must start"));
    let epoch_identity = started.epoch_identity().to_owned();
    let paused = match started.advance(execution) {
        WorthQueryDirectConvergenceStepOutcome::Continue(paused) => paused,
        _ => panic!("direct denial peer must reach its safe point"),
    };
    let denied = match paused.yield_iteration() {
        WorthQueryDirectConvergenceYieldOutcome::Denied(denied) => denied,
        _ => panic!("missing direct checkpoint must deny before suspension"),
    };
    assert_eq!(probe.suspension_attempt_count(), 0);
    DirectDeniedPeer {
        denied,
        epoch_identity,
        probe,
    }
}

fn workflow_denied_peer(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorkflowDeniedPeer {
    let (fixture, probe) = workflow_yield_denial_admission_fixture(resource_request);
    let WorkflowAdmissionFixture {
        runtime,
        operation,
        contract,
        managed,
        graph,
        bridge: _,
    } = fixture;
    let admitted = runtime
        .admit_workflow_convergence_epoch(&operation, contract, managed, graph)
        .unwrap_or_else(|_| panic!("workflow yield-denial peer must admit"));
    let epoch = admitted
        .start()
        .unwrap_or_else(|_| panic!("workflow yield-denial peer must start"));
    let started = epoch
        .begin_stage_iteration(execution, WORKFLOW_STAGE, call("same-stage-yield-denial"))
        .unwrap_or_else(|_| panic!("workflow yield-denial stage must start"));
    let epoch_identity = started.epoch_identity().to_owned();
    let paused = match started.advance(execution) {
        WorthQueryWorkflowConvergenceStepOutcome::Continue(paused) => paused,
        _ => panic!("workflow denial peer must reach its safe point"),
    };
    let denied = match paused.yield_iteration() {
        WorthQueryWorkflowConvergenceYieldOutcome::Denied(denied) => denied,
        _ => panic!("missing workflow checkpoint must deny before suspension"),
    };
    assert_eq!(probe.suspension_attempt_count(), 0);
    WorkflowDeniedPeer {
        denied,
        epoch_identity,
        probe,
    }
}

fn complete_direct_peer(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    peer: DirectDeniedPeer,
) -> CompletedPeer {
    let terminal = match peer.denied.retry().advance(execution) {
        WorthQueryDirectConvergenceStepOutcome::Completed(
            WorthQueryDirectConvergenceIterationOutcome::Converged(terminal),
        ) => terminal,
        _ => panic!("rightful direct denial owner must converge"),
    };
    assert_eq!(terminal.identity(), peer.epoch_identity);
    assert_eq!(peer.probe.suspension_attempt_count(), 0);
    assert_terminal_counters(terminal.counters());
    let completed = completed_peer(terminal.incumbents(), terminal.latest_report());
    let receipt = terminal
        .cleanup()
        .unwrap_or_else(|_| panic!("direct denial owner must clean up"));
    assert_eq!(receipt.identity(), peer.epoch_identity);
    assert_cleanup_counters(receipt.counters());
    completed
}

fn complete_workflow_peer(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    peer: WorkflowDeniedPeer,
) -> CompletedPeer {
    let terminal = match peer.denied.retry().advance(execution) {
        WorthQueryWorkflowConvergenceStepOutcome::Completed(
            WorthQueryWorkflowConvergenceIterationOutcome::Converged(terminal),
        ) => terminal,
        _ => panic!("rightful workflow denial owner must converge"),
    };
    assert_eq!(terminal.identity(), peer.epoch_identity);
    assert_eq!(peer.probe.suspension_attempt_count(), 0);
    assert_terminal_counters(terminal.counters());
    let completed = completed_peer(terminal.incumbents(), terminal.latest_report());
    let receipt = match terminal.cleanup() {
        WorthQueryWorkflowConvergenceCleanupOutcome::Complete(receipt) => receipt,
        _ => panic!("workflow denial owner must clean up"),
    };
    assert_eq!(receipt.identity(), peer.epoch_identity);
    assert_cleanup_counters(receipt.counters());
    completed
}

fn completed_peer(
    incumbents: &[crate::domain_computation::WorthQueryRetainedConvergenceCandidateEvidence],
    report: Option<&crate::domain_computation::WorthQueryBoundConvergenceReport>,
) -> CompletedPeer {
    let report = report.expect("denial owner completion must retain its report");
    assert_eq!(incumbents.len(), 1);
    assert_eq!(
        incumbents[0].report_evidence_identity(),
        report.evidence_identity()
    );
    CompletedPeer {
        state_identity: incumbents[0].state_identity().to_owned(),
        occurrence_identity: incumbents[0].occurrence_identity().to_owned(),
    }
}

fn assert_terminal_counters(
    counters: &crate::domain_computation::WorthQueryConvergenceEpochCounters,
) {
    assert_eq!(counters.iteration_count(), 1);
    assert_eq!(counters.yield_count(), 0);
    assert_eq!(counters.readmission_count(), 0);
}

fn assert_cleanup_counters(
    counters: &crate::domain_computation::WorthQueryConvergenceEpochCounters,
) {
    assert_eq!(counters.cleanup_attempt_count(), 1);
    assert_eq!(counters.cleanup_completion_count(), 1);
}

fn call(identity: &str) -> WorthQueryManagedGraphCallRequest {
    WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, identity)
}
