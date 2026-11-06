use crate::domain_computation::convergence_epoch::tests::fixture::direct_installed_world::direct_yield_denial_admission_fixture;
use crate::domain_computation::convergence_epoch::tests::fixture::direct_installed_world::DirectAdmissionFixture;
use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::WORKFLOW_STAGE;
use crate::domain_computation::convergence_epoch::tests::fixture::provider::yield_recovery::FixtureYieldRecoveryProbe;
use crate::domain_computation::convergence_epoch::tests::fixture::workflow_installed_world::workflow_yield_denial_admission_fixture;
use crate::domain_computation::convergence_epoch::tests::fixture::workflow_installed_world::WorkflowAdmissionFixture;
use worth_query_execution::facade::convergence_epoch::WorthQueryDeniedDirectConvergenceYield;
use worth_query_execution::facade::convergence_epoch::WorthQueryDeniedWorkflowConvergenceYield;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceIterationOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceStepOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceYieldOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceCleanupOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceIterationOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceStepOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceYieldOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCallKind;
use worth_query_execution::facade::domain_computation::WorthQueryManagedGraphCallRequest;

pub struct DirectDeniedPeer {
    pub denied: WorthQueryDeniedDirectConvergenceYield,
    pub epoch_identity: String,
    pub probe: FixtureYieldRecoveryProbe,
}

pub struct WorkflowDeniedPeer {
    pub denied: WorthQueryDeniedWorkflowConvergenceYield,
    pub epoch_identity: String,
    pub probe: FixtureYieldRecoveryProbe,
}

pub struct CompletedPeer {
    pub state_identity: String,
    pub occurrence_identity: String,
}

#[test]
fn same_scope_direct_denials_resume_their_exact_epochs() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
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
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
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

pub fn direct_denied_peer(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
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
    let started = {
        let installed_fixture_phase = epoch.fixture_phase_for_test(execution);
        epoch.begin_iteration(&installed_fixture_phase, call("same-scope-yield-denial"))
    }
    .unwrap_or_else(|_| panic!("direct yield-denial peer must start"));
    let epoch_identity = started.epoch_identity().to_owned();
    let paused = {
        let fixture_phase_result = {
            let installed_fixture_phase = started.fixture_phase_for_test(execution);
            started.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryDirectConvergenceStepOutcome::Continue(paused) => paused,
            _ => panic!("direct denial peer must reach its safe point"),
        }
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

pub fn workflow_denied_peer(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
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
    let started = {
        let installed_fixture_phase = epoch.fixture_phase_for_test(execution);
        epoch.begin_stage_iteration(
            &installed_fixture_phase,
            WORKFLOW_STAGE,
            call("same-stage-yield-denial"),
        )
    }
    .unwrap_or_else(|_| panic!("workflow yield-denial stage must start"));
    let epoch_identity = started.epoch_identity().to_owned();
    let paused = {
        let fixture_phase_result = {
            let installed_fixture_phase = started.fixture_phase_for_test(execution);
            started.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryWorkflowConvergenceStepOutcome::Continue(paused) => paused,
            _ => panic!("workflow denial peer must reach its safe point"),
        }
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

pub fn complete_direct_peer(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    peer: DirectDeniedPeer,
) -> CompletedPeer {
    let terminal = {
        let fixture_phase_result = {
            let fixture_state = peer.denied.retry();
            let installed_fixture_phase = fixture_state.fixture_phase_for_test(execution);
            fixture_state.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryDirectConvergenceStepOutcome::Completed(
                WorthQueryDirectConvergenceIterationOutcome::Converged(terminal),
            ) => terminal,
            _ => panic!("rightful direct denial owner must converge"),
        }
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

pub fn complete_workflow_peer(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    peer: WorkflowDeniedPeer,
) -> CompletedPeer {
    let terminal = {
        let fixture_phase_result = {
            let fixture_state = peer.denied.retry();
            let installed_fixture_phase = fixture_state.fixture_phase_for_test(execution);
            fixture_state.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryWorkflowConvergenceStepOutcome::Completed(
                WorthQueryWorkflowConvergenceIterationOutcome::Converged(terminal),
            ) => terminal,
            _ => panic!("rightful workflow denial owner must converge"),
        }
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

pub fn completed_peer(
    incumbents: &[worth_query_execution::facade::convergence_epoch::WorthQueryRetainedConvergenceCandidateEvidence],
    report: Option<
        &worth_query_execution::facade::convergence_epoch::WorthQueryBoundConvergenceReport,
    >,
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

pub fn assert_terminal_counters(
    counters: &worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceEpochCounters,
) {
    assert_eq!(counters.iteration_count(), 1);
    assert_eq!(counters.yield_count(), 0);
    assert_eq!(counters.readmission_count(), 0);
}

pub fn assert_cleanup_counters(
    counters: &worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceEpochCounters,
) {
    assert_eq!(counters.cleanup_attempt_count(), 1);
    assert_eq!(counters.cleanup_completion_count(), 1);
}

pub fn call(identity: &str) -> WorthQueryManagedGraphCallRequest {
    WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, identity)
}
