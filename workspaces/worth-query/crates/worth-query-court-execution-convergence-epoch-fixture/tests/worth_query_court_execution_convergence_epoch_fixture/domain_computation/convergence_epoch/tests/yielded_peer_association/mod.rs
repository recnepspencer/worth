pub use crate::domain_computation::convergence_epoch::tests::yielded_peer_association::owner_denial::deny_cross_direct_owners;
pub use crate::domain_computation::convergence_epoch::tests::yielded_peer_association::owner_denial::deny_cross_workflow_owners;

pub use crate::domain_computation::convergence_epoch::tests::fixture::direct_installed_world::direct_admission_fixture;
pub use crate::domain_computation::convergence_epoch::tests::fixture::workflow_installed_world::workflow_admission_fixture;
pub use crate::domain_computation::convergence_epoch::tests::fixture::direct_installed_world::DirectAdmissionFixture;
pub use crate::domain_computation::convergence_epoch::tests::fixture::provider::disposition::FixtureDisposition;
pub use crate::domain_computation::convergence_epoch::tests::fixture::workflow_installed_world::WorkflowAdmissionFixture;
pub use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::WORKFLOW_STAGE;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceIterationOutcome;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceReadmissionOutcome;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceStepOutcome;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceYieldOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime;
pub use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCallKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryManagedGraphCallRequest;
pub use worth_query_execution::facade::domain_computation::WorthQueryReadmissionEvidence;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceCleanupOutcome;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceIterationOutcome;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceReadmissionOutcome;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceStepOutcome;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceYieldOutcome;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryYieldedDirectConvergenceIteration;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryYieldedWorkflowConvergenceIteration;
pub use crate::domain_computation::convergence_epoch::tests::yielded_peer_association::completion_evidence::completed_peer;
pub use worth_runtime_bridge::facade::RuntimeBridge;

pub const PEER_CALL_SCOPE: &str = "same-yielded-peer-scope";

pub struct DirectYieldedPeer {
    pub runtime: WorthQueryExecutionRuntime,
    pub bridge: RuntimeBridge,
    pub yielded: WorthQueryYieldedDirectConvergenceIteration,
}

pub struct WorkflowYieldedPeer {
    pub runtime: WorthQueryExecutionRuntime,
    pub bridge: RuntimeBridge,
    pub yielded: WorthQueryYieldedWorkflowConvergenceIteration,
}

#[test]
fn same_scope_direct_peers_deny_cross_owners_then_complete_rightfully() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
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
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
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

pub struct CompletedPeer {
    pub state_identity: String,
    pub occurrence_identity: String,
}

pub fn complete_direct_peer(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    yielded: WorthQueryYieldedDirectConvergenceIteration,
    runtime: &WorthQueryExecutionRuntime,
    bridge: &RuntimeBridge,
) -> CompletedPeer {
    let active_request = execution;

    let epoch_identity = yielded.epoch_identity().to_owned();
    let started = {
        let fixture_phase_result = {
            let fixture_target = yielded;
            let installed_fixture_phase = fixture_target.fixture_phase_for_test(active_request);
            fixture_target.readmit_same_runtime(&installed_fixture_phase, runtime, bridge)
        };
        match fixture_phase_result {
            WorthQueryDirectConvergenceReadmissionOutcome::Readmitted(readmitted) => {
                let evidence = readmitted.readmission_evidence();
                assert_committed_owner_readmission(evidence, 0);
                readmitted.into_started()
            }
            _ => panic!("rightful direct peer must readmit"),
        }
    };
    let terminal = {
        let fixture_phase_result = {
            let fixture_target = started;
            let installed_fixture_phase = fixture_target.fixture_phase_for_test(execution);
            fixture_target.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryDirectConvergenceStepOutcome::Completed(
                WorthQueryDirectConvergenceIterationOutcome::Converged(terminal),
            ) => terminal,
            _ => panic!("rightfully readmitted direct peer must converge"),
        }
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

pub fn complete_workflow_peer(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    yielded: WorthQueryYieldedWorkflowConvergenceIteration,
    runtime: &WorthQueryExecutionRuntime,
    bridge: &RuntimeBridge,
) -> CompletedPeer {
    let active_request = execution;

    let epoch_identity = yielded.epoch_identity().to_owned();
    let started = {
        let fixture_phase_result = {
            let fixture_target = yielded;
            let installed_fixture_phase = fixture_target.fixture_phase_for_test(active_request);
            fixture_target.readmit_same_runtime(&installed_fixture_phase, runtime, bridge)
        };
        match fixture_phase_result {
            WorthQueryWorkflowConvergenceReadmissionOutcome::Readmitted(readmitted) => {
                let evidence = readmitted.readmission_evidence();
                assert_committed_owner_readmission(evidence, 1);
                readmitted.into_started()
            }
            _ => panic!("rightful workflow peer must readmit"),
        }
    };
    let terminal = {
        let fixture_phase_result = {
            let fixture_target = started;
            let installed_fixture_phase = fixture_target.fixture_phase_for_test(execution);
            fixture_target.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryWorkflowConvergenceStepOutcome::Completed(
                WorthQueryWorkflowConvergenceIterationOutcome::Converged(terminal),
            ) => terminal,
            _ => panic!("rightfully readmitted workflow peer must converge"),
        }
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

pub fn direct_yielded_peer(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
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
    let started = {
        let fixture_target = epoch;
        let installed_fixture_phase = fixture_target.fixture_phase_for_test(execution);
        fixture_target.begin_iteration(&installed_fixture_phase, call())
    }
    .unwrap_or_else(|_| panic!("direct peer iteration must start"));
    let paused = {
        let fixture_phase_result = {
            let fixture_target = started;
            let installed_fixture_phase = fixture_target.fixture_phase_for_test(execution);
            fixture_target.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryDirectConvergenceStepOutcome::Continue(paused) => paused,
            _ => panic!("direct peer must reach the yield safe point"),
        }
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

pub fn workflow_yielded_peer(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
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
    let started = {
        let fixture_target = epoch;
        let installed_fixture_phase = fixture_target.fixture_phase_for_test(execution);
        fixture_target.begin_stage_iteration(&installed_fixture_phase, WORKFLOW_STAGE, call())
    }
    .unwrap_or_else(|_| panic!("workflow peer iteration must start"));
    let paused = {
        let fixture_phase_result = {
            let fixture_target = started;
            let installed_fixture_phase = fixture_target.fixture_phase_for_test(execution);
            fixture_target.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryWorkflowConvergenceStepOutcome::Continue(paused) => paused,
            _ => panic!("workflow peer must reach the yield safe point"),
        }
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

pub fn call() -> WorthQueryManagedGraphCallRequest {
    WorthQueryManagedGraphCallRequest::new(
        WorthQueryGraphProviderCallKind::Observe,
        PEER_CALL_SCOPE,
    )
}

pub fn assert_foreign_query_denial(evidence: WorthQueryReadmissionEvidence) {
    assert_query_preflight_only(evidence);
    assert!(evidence.bridge_counters().is_none());
}

pub fn assert_foreign_bridge_denial(evidence: WorthQueryReadmissionEvidence) {
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

pub fn assert_query_preflight_only(evidence: WorthQueryReadmissionEvidence) {
    let query = evidence.query_counters();
    assert_eq!(query.preflight_check_count(), 1);
    assert_eq!(query.fresh_resource_attempt_count(), 0);
    assert_eq!(query.bridge_readmission_attempt_count(), 0);
    assert_eq!(query.provider_restore_attempt_count(), 0);
    assert_eq!(query.artifact_generation_attempt_count(), 0);
    assert_eq!(query.artifact_generation_commit_count(), 0);
    assert_eq!(query.committed_attempt_count(), 0);
}

pub fn assert_committed_owner_readmission(
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

#[path = "completion_evidence/mod.rs"]
pub mod completion_evidence;
#[path = "owner_denial/mod.rs"]
pub mod owner_denial;
