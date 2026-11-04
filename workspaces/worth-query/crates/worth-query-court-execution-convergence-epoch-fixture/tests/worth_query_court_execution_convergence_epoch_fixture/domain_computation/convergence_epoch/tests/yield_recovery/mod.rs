pub use crate::domain_computation::convergence_epoch::tests::fixture::direct_installed_world::direct_admission_fixture;
pub use crate::domain_computation::convergence_epoch::tests::fixture::direct_installed_world::direct_yield_recovery_admission_fixture;
pub use crate::domain_computation::convergence_epoch::tests::fixture::direct_installed_world::DirectAdmissionFixture;
pub use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::WORKFLOW_STAGE;
pub use crate::domain_computation::convergence_epoch::tests::fixture::provider::disposition::FixtureDisposition;
pub use crate::domain_computation::convergence_epoch::tests::fixture::provider::yield_recovery::FixtureYieldRecoveryArtifact;
pub use crate::domain_computation::convergence_epoch::tests::fixture::workflow_installed_world::workflow_admission_fixture;
pub use crate::domain_computation::convergence_epoch::tests::fixture::workflow_installed_world::workflow_yield_recovery_admission_fixture;
pub use crate::domain_computation::convergence_epoch::tests::fixture::workflow_installed_world::workflow_yield_recovery_artifact_admission_fixture;
pub use crate::domain_computation::convergence_epoch::tests::fixture::workflow_installed_world::WorkflowAdmissionFixture;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceStepOutcome;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceYieldOutcome;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceYieldRecoveryRequired;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceYieldRunningRecovery;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceStepOutcome;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceYieldOutcome;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceYieldRecoveryCleanupOutcome;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceYieldRecoveryRequired;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceYieldRunningRecovery;
pub use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCallKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryManagedGraphCallRequest;

#[test]
fn direct_running_recovery_resumes_the_exact_epoch_on_the_rightful_thread() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (epoch_identity, paused) = direct_paused(
            execution,
            direct_admission_fixture(FixtureDisposition::YieldThenConverged, resource_request),
            "direct-running-yield-recovery",
        );
        let running = foreign_direct_running_recovery(paused);

        let paused = resume_direct(running);
        let yielded = match paused.yield_iteration() {
            WorthQueryDirectConvergenceYieldOutcome::Yielded(yielded) => yielded,
            _ => panic!("rightful-thread direct recovery must yield"),
        };
        assert_eq!(yielded.epoch_identity(), epoch_identity);
        let receipt = match yielded.cleanup() {
            worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceYieldCleanupOutcome::Complete(
                receipt,
            ) => receipt,
            _ => panic!("rightful direct recovery cleanup must complete"),
        };
        assert_eq!(receipt.identity(), epoch_identity);
        assert_eq!(receipt.counters().yield_count(), 1);
        assert_eq!(receipt.counters().cleanup_attempt_count(), 1);
        assert_eq!(receipt.counters().cleanup_completion_count(), 1);
    });
}

#[test]
fn workflow_running_recovery_resumes_the_exact_stage_on_the_rightful_thread() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (epoch_identity, paused) = workflow_paused(
            execution,
            workflow_admission_fixture(FixtureDisposition::YieldThenConverged, resource_request),
            "workflow-running-yield-recovery",
        );
        let running = foreign_workflow_running_recovery(paused);

        let paused = resume_workflow(running);
        let yielded = match paused.yield_iteration() {
            WorthQueryWorkflowConvergenceYieldOutcome::Yielded(yielded) => yielded,
            _ => panic!("rightful-thread workflow recovery must yield"),
        };
        assert_eq!(yielded.epoch_identity(), epoch_identity);
        let receipt = match yielded.cleanup() {
        worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceYieldCleanupOutcome::Complete(
            receipt,
        ) => receipt,
        _ => panic!("rightful workflow recovery cleanup must complete"),
    };
        assert_eq!(receipt.identity(), epoch_identity);
        assert_eq!(receipt.counters().yield_count(), 1);
        assert_eq!(receipt.counters().cleanup_attempt_count(), 1);
        assert_eq!(receipt.counters().cleanup_completion_count(), 1);
    });
}

#[test]
fn direct_terminal_recovery_closes_resources_once() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (fixture, probe) = direct_yield_recovery_admission_fixture(resource_request);
        let (epoch_identity, paused) =
            direct_paused(execution, fixture, "direct-terminal-yield-recovery");
        let cleanup = match paused.yield_iteration() {
            WorthQueryDirectConvergenceYieldOutcome::RecoveryRequired(
                WorthQueryDirectConvergenceYieldRecoveryRequired::TerminalCleanup(cleanup),
            ) => cleanup,
            _ => panic!("provider suspension rejection must require direct terminal cleanup"),
        };
        assert_eq!(probe.suspension_attempt_count(), 1);
        let receipt = match cleanup.finish() {
            Ok(receipt) => receipt,
            Err(_) => panic!("terminal direct recovery was misclassified as running"),
        };
        assert_eq!(receipt.identity(), epoch_identity);
        assert_eq!(receipt.counters().yield_count(), 0);
        assert_eq!(receipt.counters().cleanup_attempt_count(), 1);
        assert_eq!(receipt.counters().cleanup_completion_count(), 1);
        assert!(receipt.incumbents().is_empty());
        assert!(receipt.latest_report().is_none());
    });
}

#[test]
fn workflow_terminal_recovery_closes_without_artifacts() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (fixture, probe) = workflow_yield_recovery_admission_fixture(resource_request);
        let (epoch_identity, paused) =
            workflow_paused(execution, fixture, "workflow-terminal-yield-recovery");
        let cleanup = workflow_terminal_cleanup(paused);
        assert_eq!(probe.suspension_attempt_count(), 1);
        let receipt = match cleanup.finish() {
            Ok(WorthQueryWorkflowConvergenceYieldRecoveryCleanupOutcome::Complete(receipt)) => {
                receipt
            }
            Ok(_) => panic!("artifact-free workflow recovery did not complete"),
            Err(_) => panic!("terminal workflow recovery was misclassified as running"),
        };
        assert_closed_workflow_receipt(&receipt, &epoch_identity, 1);
    });
}

#[test]
fn workflow_terminal_recovery_pending_retry_preserves_authority() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (fixture, receiver, probe) = workflow_yield_recovery_artifact_admission_fixture(
            FixtureYieldRecoveryArtifact::Cooperative,
            resource_request,
        );
        let (epoch_identity, paused) =
            workflow_paused(execution, fixture, "workflow-yield-recovery-pending");
        let handle = receiver
            .recv()
            .expect("real provider step must issue its artifact handle");
        let borrowed = handle
            .borrow("convergence yield recovery pending proof")
            .expect("installed artifact contract must admit a real borrow");
        let cleanup = workflow_terminal_cleanup(paused);
        let pending = match cleanup.finish() {
            Ok(WorthQueryWorkflowConvergenceYieldRecoveryCleanupOutcome::Pending(pending)) => {
                pending
            }
            Ok(_) => panic!("live artifact borrow did not retain pending authority"),
            Err(_) => panic!("terminal workflow recovery was misclassified as running"),
        };
        assert_eq!(probe.suspension_attempt_count(), 1);
        assert_eq!(probe.disposal_attempt_count(), 0);
        assert_eq!(pending.identity(), epoch_identity);
        assert_eq!(pending.counters().cleanup_attempt_count(), 1);
        assert_eq!(pending.counters().cleanup_completion_count(), 0);

        drop(borrowed);
        drop(handle);
        let receipt = match pending.retry() {
            Ok(WorthQueryWorkflowConvergenceYieldRecoveryCleanupOutcome::Complete(receipt)) => {
                receipt
            }
            Ok(_) => panic!("released artifact owner did not complete rightful retry"),
            Err(_) => panic!("pending terminal recovery lost its cleanup posture"),
        };
        assert_closed_workflow_receipt(&receipt, &epoch_identity, 2);
    });
}

#[test]
fn workflow_terminal_recovery_types_double_artifact_panic_as_closed_recovery() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (fixture, receiver, probe) = workflow_yield_recovery_artifact_admission_fixture(
            FixtureYieldRecoveryArtifact::DoublePanicking,
            resource_request,
        );
        let (epoch_identity, paused) =
            workflow_paused(execution, fixture, "workflow-yield-recovery-double-panic");
        let _handle = receiver
            .recv()
            .expect("real provider step must issue its failing artifact handle");
        let cleanup = workflow_terminal_cleanup(paused);
        let receipt = match cleanup.finish() {
            Ok(WorthQueryWorkflowConvergenceYieldRecoveryCleanupOutcome::RecoveryRequired(
                receipt,
            )) => receipt,
            Ok(_) => panic!("double artifact panic was not typed as closed recovery"),
            Err(_) => panic!("terminal workflow recovery was misclassified as running"),
        };
        assert_eq!(probe.suspension_attempt_count(), 1);
        assert_eq!(probe.disposal_attempt_count(), 1);
        assert_eq!(probe.destructor_attempt_count(), 1);
        assert_closed_workflow_receipt(&receipt, &epoch_identity, 1);
    });
}

pub fn direct_paused(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    fixture: DirectAdmissionFixture,
    call_identity: &str,
) -> (
    String,
    worth_query_execution::facade::convergence_epoch::WorthQueryPausedDirectConvergenceIteration,
) {
    let epoch = fixture.admit();
    let identity = epoch.identity().to_owned();
    let started = {
        let installed_fixture_phase = epoch.fixture_phase_for_test(execution);
        epoch.begin_iteration(&installed_fixture_phase, call(call_identity))
    }
    .unwrap_or_else(|_| panic!("direct recovery iteration must start"));
    let paused = {
        let fixture_phase_result = {
            let installed_fixture_phase = started.fixture_phase_for_test(execution);
            started.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryDirectConvergenceStepOutcome::Continue(paused) => paused,
            _ => panic!("direct recovery fixture must reach a safe point"),
        }
    };
    (identity, paused)
}

pub fn workflow_paused(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    fixture: WorkflowAdmissionFixture,
    call_identity: &str,
) -> (
    String,
    worth_query_execution::facade::convergence_epoch::WorthQueryPausedWorkflowConvergenceIteration,
) {
    let epoch = fixture.admit();
    let identity = epoch.identity().to_owned();
    let started = {
        let installed_fixture_phase = epoch.fixture_phase_for_test(execution);
        epoch.begin_stage_iteration(
            &installed_fixture_phase,
            WORKFLOW_STAGE,
            call(call_identity),
        )
    }
    .unwrap_or_else(|_| panic!("workflow recovery iteration must start"));
    let paused = {
        let fixture_phase_result = {
            let installed_fixture_phase = started.fixture_phase_for_test(execution);
            started.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryWorkflowConvergenceStepOutcome::Continue(paused) => paused,
            _ => panic!("workflow recovery fixture must reach a safe point"),
        }
    };
    (identity, paused)
}

pub fn resume_direct(
    running: WorthQueryDirectConvergenceYieldRunningRecovery,
) -> worth_query_execution::facade::convergence_epoch::WorthQueryPausedDirectConvergenceIteration {
    match running.resume() {
        Ok(paused) => paused,
        Err(_) => panic!("running direct recovery changed posture"),
    }
}

pub fn resume_workflow(
    running: WorthQueryWorkflowConvergenceYieldRunningRecovery,
) -> worth_query_execution::facade::convergence_epoch::WorthQueryPausedWorkflowConvergenceIteration
{
    match running.resume() {
        Ok(paused) => paused,
        Err(_) => panic!("running workflow recovery changed posture"),
    }
}

pub fn foreign_direct_running_recovery(
    paused: worth_query_execution::facade::convergence_epoch::WorthQueryPausedDirectConvergenceIteration,
) -> WorthQueryDirectConvergenceYieldRunningRecovery {
    std::thread::spawn(move || match paused.yield_iteration() {
        WorthQueryDirectConvergenceYieldOutcome::RecoveryRequired(
            WorthQueryDirectConvergenceYieldRecoveryRequired::RunningAttempt(running),
        ) => running,
        _ => panic!("foreign-thread direct yield must retain running recovery authority"),
    })
    .join()
    .expect("direct recovery thread must return its exact owner")
}

pub fn foreign_workflow_running_recovery(
    paused: worth_query_execution::facade::convergence_epoch::WorthQueryPausedWorkflowConvergenceIteration,
) -> WorthQueryWorkflowConvergenceYieldRunningRecovery {
    std::thread::spawn(move || match paused.yield_iteration() {
        WorthQueryWorkflowConvergenceYieldOutcome::RecoveryRequired(
            WorthQueryWorkflowConvergenceYieldRecoveryRequired::RunningAttempt(running),
        ) => running,
        _ => panic!("foreign-thread workflow yield must retain running recovery authority"),
    })
    .join()
    .expect("workflow recovery thread must return its exact owner")
}

pub fn cleanup_direct_peer(
    yielded: worth_query_execution::facade::convergence_epoch::WorthQueryYieldedDirectConvergenceIteration,
    epoch_identity: &str,
) {
    let receipt = match yielded.cleanup() {
        worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceYieldCleanupOutcome::Complete(
            receipt,
        ) => receipt,
        _ => panic!("rightful direct peer cleanup did not complete"),
    };
    assert_eq!(receipt.identity(), epoch_identity);
    assert_eq!(receipt.counters().cleanup_completion_count(), 1);
}

pub fn cleanup_workflow_peer(
    yielded: worth_query_execution::facade::convergence_epoch::WorthQueryYieldedWorkflowConvergenceIteration,
    epoch_identity: &str,
) {
    let receipt = match yielded.cleanup() {
        worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceYieldCleanupOutcome::Complete(
            receipt,
        ) => receipt,
        _ => panic!("rightful workflow peer cleanup did not complete"),
    };
    assert_eq!(receipt.identity(), epoch_identity);
    assert_eq!(receipt.counters().cleanup_completion_count(), 1);
}

pub fn workflow_terminal_cleanup(
    paused: worth_query_execution::facade::convergence_epoch::WorthQueryPausedWorkflowConvergenceIteration,
) -> worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceYieldTerminalCleanupRequired{
    match paused.yield_iteration() {
        WorthQueryWorkflowConvergenceYieldOutcome::RecoveryRequired(
            WorthQueryWorkflowConvergenceYieldRecoveryRequired::TerminalCleanup(cleanup),
        ) => cleanup,
        _ => panic!("provider suspension rejection must require workflow terminal cleanup"),
    }
}

pub fn assert_closed_workflow_receipt(
    receipt: &worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceYieldRecoveryCleanupReceipt,
    epoch_identity: &str,
    attempt_count: usize,
) {
    assert_eq!(receipt.identity(), epoch_identity);
    assert_eq!(receipt.counters().yield_count(), 0);
    assert_eq!(receipt.counters().cleanup_attempt_count(), attempt_count);
    assert_eq!(receipt.counters().cleanup_completion_count(), 1);
    assert!(receipt.incumbents().is_empty());
    assert!(receipt.latest_report().is_none());
}

pub fn call(identity: &str) -> WorthQueryManagedGraphCallRequest {
    WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, identity)
}

#[path = "same_scope_and_stage_running_recovery_peers_keep_their_own_epochs/mod.rs"]
pub mod same_scope_and_stage_running_recovery_peers_keep_their_own_epochs;
