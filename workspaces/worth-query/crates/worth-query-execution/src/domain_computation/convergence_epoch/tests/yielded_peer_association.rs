mod completion_evidence;
use super::fixture::{
    direct_admission_fixture, workflow_admission_fixture, DirectAdmissionFixture,
    FixtureDisposition, WorkflowAdmissionFixture, WORKFLOW_STAGE,
};
use crate::domain_computation::{
    WorthQueryDirectConvergenceIterationOutcome, WorthQueryDirectConvergenceReadmissionOutcome,
    WorthQueryDirectConvergenceStepOutcome, WorthQueryDirectConvergenceYieldOutcome,
    WorthQueryExecutionRuntime, WorthQueryGraphProviderCallKind, WorthQueryManagedGraphCallRequest,
    WorthQueryReadmissionEvidence, WorthQueryWorkflowConvergenceCleanupOutcome,
    WorthQueryWorkflowConvergenceIterationOutcome, WorthQueryWorkflowConvergenceReadmissionOutcome,
    WorthQueryWorkflowConvergenceStepOutcome, WorthQueryWorkflowConvergenceYieldOutcome,
    WorthQueryYieldedDirectConvergenceIteration, WorthQueryYieldedWorkflowConvergenceIteration,
};
use completion_evidence::completed_peer;
use worth_runtime_bridge::facade::RuntimeBridge;

const PEER_CALL_SCOPE: &str = "same-yielded-peer-scope";

struct DirectYieldedPeer {
    runtime: WorthQueryExecutionRuntime,
    bridge: RuntimeBridge,
    yielded: WorthQueryYieldedDirectConvergenceIteration,
}

struct WorkflowYieldedPeer {
    runtime: WorthQueryExecutionRuntime,
    bridge: RuntimeBridge,
    yielded: WorthQueryYieldedWorkflowConvergenceIteration,
}

#[test]
fn same_scope_direct_peers_deny_cross_owners_then_complete_rightfully() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let DirectYieldedPeer {
            runtime: runtime_a,
            bridge: bridge_a,
            yielded: yielded_a,
        } = direct_yielded_peer(execution, resource_request);
        let DirectYieldedPeer {
            runtime: runtime_b,
            bridge: bridge_b,
            yielded: yielded_b,
        } = direct_yielded_peer(execution, resource_request);

        assert_ne!(yielded_a.epoch_identity(), yielded_b.epoch_identity());
        assert_ne!(
            yielded_a.graph_authority_identity(),
            yielded_b.graph_authority_identity()
        );
        let yielded_a =
            deny_cross_direct_owners(execution, yielded_a, &runtime_a, &runtime_b, &bridge_b);
        let yielded_b =
            deny_cross_direct_owners(execution, yielded_b, &runtime_b, &runtime_a, &bridge_a);

        let completed_a = complete_direct_peer(execution, yielded_a, &runtime_a, &bridge_a);
        let completed_b = complete_direct_peer(execution, yielded_b, &runtime_b, &bridge_b);
        assert_eq!(completed_a.state_identity, completed_b.state_identity);
        assert_ne!(
            completed_a.occurrence_identity,
            completed_b.occurrence_identity
        );
    });
}

#[test]
fn same_stage_workflow_peers_deny_cross_owners_then_complete_rightfully() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let WorkflowYieldedPeer {
            runtime: runtime_a,
            bridge: bridge_a,
            yielded: yielded_a,
        } = workflow_yielded_peer(execution, resource_request);
        let WorkflowYieldedPeer {
            runtime: runtime_b,
            bridge: bridge_b,
            yielded: yielded_b,
        } = workflow_yielded_peer(execution, resource_request);

        assert_ne!(yielded_a.epoch_identity(), yielded_b.epoch_identity());
        assert_ne!(
            yielded_a.graph_authority_identity(),
            yielded_b.graph_authority_identity()
        );
        let yielded_a =
            deny_cross_workflow_owners(execution, yielded_a, &runtime_a, &runtime_b, &bridge_b);
        let yielded_b =
            deny_cross_workflow_owners(execution, yielded_b, &runtime_b, &runtime_a, &bridge_a);

        let completed_a = complete_workflow_peer(execution, yielded_a, &runtime_a, &bridge_a);
        let completed_b = complete_workflow_peer(execution, yielded_b, &runtime_b, &bridge_b);
        assert_eq!(completed_a.state_identity, completed_b.state_identity);
        assert_ne!(
            completed_a.occurrence_identity,
            completed_b.occurrence_identity
        );
    });
}

struct CompletedPeer {
    state_identity: String,
    occurrence_identity: String,
}

fn deny_cross_direct_owners(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    yielded: WorthQueryYieldedDirectConvergenceIteration,
    owner_runtime: &WorthQueryExecutionRuntime,
    foreign_runtime: &WorthQueryExecutionRuntime,
    foreign_bridge: &RuntimeBridge,
) -> WorthQueryYieldedDirectConvergenceIteration {
    let active_request = execution;

    let epoch_identity = yielded.epoch_identity().to_owned();
    let graph_identity = yielded.graph_authority_identity().to_owned();
    let denied = match yielded.readmit_same_runtime(active_request, foreign_runtime, foreign_bridge)
    {
        WorthQueryDirectConvergenceReadmissionOutcome::Denied(denied) => denied,
        _ => panic!("foreign direct peer must not readmit the yielded owner"),
    };
    assert_foreign_query_denial(denied.readmission_evidence());
    let yielded = denied.into_yielded();
    assert_eq!(yielded.epoch_identity(), epoch_identity);
    assert_eq!(yielded.graph_authority_identity(), graph_identity);

    let denied = match yielded.readmit_same_runtime(active_request, owner_runtime, foreign_bridge) {
        WorthQueryDirectConvergenceReadmissionOutcome::Denied(denied) => denied,
        _ => panic!("foreign Bridge peer must not readmit the direct yielded owner"),
    };
    assert_foreign_bridge_denial(denied.readmission_evidence());
    let yielded = denied.into_yielded();
    assert_eq!(yielded.epoch_identity(), epoch_identity);
    assert_eq!(yielded.graph_authority_identity(), graph_identity);
    yielded
}

fn deny_cross_workflow_owners(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    yielded: WorthQueryYieldedWorkflowConvergenceIteration,
    owner_runtime: &WorthQueryExecutionRuntime,
    foreign_runtime: &WorthQueryExecutionRuntime,
    foreign_bridge: &RuntimeBridge,
) -> WorthQueryYieldedWorkflowConvergenceIteration {
    let active_request = execution;

    let epoch_identity = yielded.epoch_identity().to_owned();
    let graph_identity = yielded.graph_authority_identity().to_owned();
    let denied = match yielded.readmit_same_runtime(active_request, foreign_runtime, foreign_bridge)
    {
        WorthQueryWorkflowConvergenceReadmissionOutcome::Denied(denied) => denied,
        _ => panic!("foreign workflow peer must not readmit the yielded owner"),
    };
    assert_foreign_query_denial(denied.readmission_evidence());
    let yielded = denied.into_yielded();
    assert_eq!(yielded.epoch_identity(), epoch_identity);
    assert_eq!(yielded.graph_authority_identity(), graph_identity);

    let denied = match yielded.readmit_same_runtime(active_request, owner_runtime, foreign_bridge) {
        WorthQueryWorkflowConvergenceReadmissionOutcome::Denied(denied) => denied,
        _ => panic!("foreign Bridge peer must not readmit the workflow yielded owner"),
    };
    assert_foreign_bridge_denial(denied.readmission_evidence());
    let yielded = denied.into_yielded();
    assert_eq!(yielded.epoch_identity(), epoch_identity);
    assert_eq!(yielded.graph_authority_identity(), graph_identity);
    yielded
}

fn complete_direct_peer(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    yielded: WorthQueryYieldedDirectConvergenceIteration,
    runtime: &WorthQueryExecutionRuntime,
    bridge: &RuntimeBridge,
) -> CompletedPeer {
    let active_request = execution;

    let epoch_identity = yielded.epoch_identity().to_owned();
    let started = match yielded.readmit_same_runtime(active_request, runtime, bridge) {
        WorthQueryDirectConvergenceReadmissionOutcome::Readmitted(readmitted) => {
            let evidence = readmitted.readmission_evidence();
            assert_committed_owner_readmission(evidence, 0);
            readmitted.into_started()
        }
        _ => panic!("rightful direct peer must readmit"),
    };
    let terminal = match started.advance(execution) {
        WorthQueryDirectConvergenceStepOutcome::Completed(
            WorthQueryDirectConvergenceIterationOutcome::Converged(terminal),
        ) => terminal,
        _ => panic!("rightfully readmitted direct peer must converge"),
    };
    assert_eq!(terminal.identity(), epoch_identity);
    assert_eq!(terminal.counters().yield_count(), 1);
    assert_eq!(terminal.counters().readmission_count(), 1);
    let completed = completed_peer(terminal.incumbents(), terminal.latest_report());
    let cleanup = terminal
        .cleanup()
        .unwrap_or_else(|_| panic!("rightful direct peer must release its terminal"));
    assert_eq!(cleanup.identity(), epoch_identity);
    assert_eq!(cleanup.counters().cleanup_attempt_count(), 1);
    assert_eq!(cleanup.counters().cleanup_completion_count(), 1);
    completed
}

fn complete_workflow_peer(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    yielded: WorthQueryYieldedWorkflowConvergenceIteration,
    runtime: &WorthQueryExecutionRuntime,
    bridge: &RuntimeBridge,
) -> CompletedPeer {
    let active_request = execution;

    let epoch_identity = yielded.epoch_identity().to_owned();
    let started = match yielded.readmit_same_runtime(active_request, runtime, bridge) {
        WorthQueryWorkflowConvergenceReadmissionOutcome::Readmitted(readmitted) => {
            let evidence = readmitted.readmission_evidence();
            assert_committed_owner_readmission(evidence, 1);
            readmitted.into_started()
        }
        _ => panic!("rightful workflow peer must readmit"),
    };
    let terminal = match started.advance(execution) {
        WorthQueryWorkflowConvergenceStepOutcome::Completed(
            WorthQueryWorkflowConvergenceIterationOutcome::Converged(terminal),
        ) => terminal,
        _ => panic!("rightfully readmitted workflow peer must converge"),
    };
    assert_eq!(terminal.identity(), epoch_identity);
    assert_eq!(terminal.counters().yield_count(), 1);
    assert_eq!(terminal.counters().readmission_count(), 1);
    let completed = completed_peer(terminal.incumbents(), terminal.latest_report());
    let cleanup = match terminal.cleanup() {
        WorthQueryWorkflowConvergenceCleanupOutcome::Complete(cleanup) => cleanup,
        _ => panic!("rightful workflow peer must release its terminal"),
    };
    assert_eq!(cleanup.identity(), epoch_identity);
    assert_eq!(cleanup.counters().cleanup_attempt_count(), 1);
    assert_eq!(cleanup.counters().cleanup_completion_count(), 1);
    completed
}

fn direct_yielded_peer(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> DirectYieldedPeer {
    let DirectAdmissionFixture {
        runtime,
        operation,
        alternate_basis_operation: _,
        contract,
        managed,
        graph,
        bridge,
    } = direct_admission_fixture(FixtureDisposition::YieldThenConverged, resource_request);
    let epoch = runtime
        .admit_direct_convergence_epoch(&operation, contract, managed, graph)
        .unwrap_or_else(|_| panic!("direct peer must admit"))
        .start();
    let started = epoch
        .begin_iteration(execution, call())
        .unwrap_or_else(|_| panic!("direct peer iteration must start"));
    let paused = match started.advance(execution) {
        WorthQueryDirectConvergenceStepOutcome::Continue(paused) => paused,
        _ => panic!("direct peer must reach the yield safe point"),
    };
    let yielded = match paused.yield_iteration() {
        WorthQueryDirectConvergenceYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("direct peer must yield"),
    };
    DirectYieldedPeer {
        runtime,
        bridge,
        yielded,
    }
}

fn workflow_yielded_peer(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorkflowYieldedPeer {
    let WorkflowAdmissionFixture {
        runtime,
        operation,
        contract,
        managed,
        graph,
        bridge,
    } = workflow_admission_fixture(FixtureDisposition::YieldThenConverged, resource_request);
    let admitted = runtime
        .admit_workflow_convergence_epoch(&operation, contract, managed, graph)
        .unwrap_or_else(|_| panic!("workflow peer must admit"));
    let epoch = admitted
        .start()
        .unwrap_or_else(|_| panic!("workflow peer epoch must start"));
    let started = epoch
        .begin_stage_iteration(execution, WORKFLOW_STAGE, call())
        .unwrap_or_else(|_| panic!("workflow peer iteration must start"));
    let paused = match started.advance(execution) {
        WorthQueryWorkflowConvergenceStepOutcome::Continue(paused) => paused,
        _ => panic!("workflow peer must reach the yield safe point"),
    };
    let yielded = match paused.yield_iteration() {
        WorthQueryWorkflowConvergenceYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("workflow peer must yield"),
    };
    WorkflowYieldedPeer {
        runtime,
        bridge,
        yielded,
    }
}

fn call() -> WorthQueryManagedGraphCallRequest {
    WorthQueryManagedGraphCallRequest::new(
        WorthQueryGraphProviderCallKind::Observe,
        PEER_CALL_SCOPE,
    )
}

fn assert_foreign_query_denial(evidence: WorthQueryReadmissionEvidence) {
    assert_query_preflight_only(evidence);
    assert!(evidence.bridge_counters().is_none());
}

fn assert_foreign_bridge_denial(evidence: WorthQueryReadmissionEvidence) {
    assert_query_preflight_only(evidence);
    let bridge = evidence
        .bridge_counters()
        .expect("foreign Bridge denial must retain exact preflight evidence");
    assert_eq!(bridge.preflight_check_count(), 1);
    assert_eq!(bridge.reservation_check_count(), 0);
    assert_eq!(bridge.signal_attempt_admission_count(), 0);
    assert_eq!(bridge.signal_attempt_check_count(), 0);
    assert_eq!(bridge.signal_queue_binding_count(), 0);
    assert_eq!(bridge.abort_count(), 0);
    assert_eq!(bridge.commit_count(), 0);
}

fn assert_query_preflight_only(evidence: WorthQueryReadmissionEvidence) {
    let query = evidence.query_counters();
    assert_eq!(query.preflight_check_count(), 1);
    assert_eq!(query.fresh_resource_attempt_count(), 0);
    assert_eq!(query.bridge_readmission_attempt_count(), 0);
    assert_eq!(query.provider_restore_attempt_count(), 0);
    assert_eq!(query.artifact_generation_attempt_count(), 0);
    assert_eq!(query.artifact_generation_commit_count(), 0);
    assert_eq!(query.committed_attempt_count(), 0);
}

fn assert_committed_owner_readmission(
    evidence: WorthQueryReadmissionEvidence,
    artifact_attempts: usize,
) {
    let query = evidence.query_counters();
    assert_eq!(query.preflight_check_count(), 1);
    assert_eq!(query.fresh_resource_attempt_count(), 1);
    assert_eq!(query.bridge_readmission_attempt_count(), 1);
    assert_eq!(query.provider_restore_attempt_count(), 1);
    assert_eq!(query.artifact_generation_attempt_count(), artifact_attempts);
    assert_eq!(query.artifact_generation_commit_count(), artifact_attempts);
    assert_eq!(query.committed_attempt_count(), 1);
    let bridge = evidence
        .bridge_counters()
        .expect("rightful readmission must retain committed Bridge evidence");
    assert_eq!(bridge.preflight_check_count(), 1);
    assert_eq!(bridge.reservation_check_count(), 1);
    assert_eq!(bridge.signal_attempt_admission_count(), 1);
    assert_eq!(bridge.signal_attempt_check_count(), 1);
    assert_eq!(bridge.signal_queue_binding_count(), 1);
    assert_eq!(bridge.abort_count(), 0);
    assert_eq!(bridge.commit_count(), 1);
}
