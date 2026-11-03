pub use crate::domain_computation::convergence_epoch::tests::fixture::direct_installed_world::direct_admission_fixture;
pub use crate::domain_computation::convergence_epoch::tests::fixture::direct_installed_world::DirectAdmissionFixture;
pub use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::WORKFLOW_STAGE;
pub use crate::domain_computation::convergence_epoch::tests::fixture::provider::disposition::FixtureDisposition;
pub use crate::domain_computation::convergence_epoch::tests::fixture::workflow_installed_world::workflow_admission_fixture;
pub use crate::domain_computation::convergence_epoch::tests::fixture::workflow_installed_world::workflow_yield_pending_admission_fixture;
pub use crate::domain_computation::convergence_epoch::tests::fixture::workflow_installed_world::WorkflowAdmissionFixture;
pub use crate::domain_computation::convergence_epoch::tests::yield_cleanup::pending_artifact::workflow_yield_with_pending_artifact;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceStepOutcome;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceYieldCleanupOutcome;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceYieldOutcome;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceStepOutcome;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceYieldCleanupOutcome;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceYieldOutcome;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryYieldedDirectConvergenceIteration;
pub use worth_query_execution::facade::convergence_epoch::WorthQueryYieldedWorkflowConvergenceIteration;
pub use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCallKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryManagedGraphCallRequest;
pub use worth_query_execution::facade::domain_computation::WorthQueryMoveOnlyArtifactHandle;

#[test]
fn direct_yield_cleanup_preserves_its_closed_epoch_receipt() {
    worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let yielded = direct_yield(execution, "direct-yield-cleanup", resource_request);
        let epoch_identity = yielded.epoch_identity().to_owned();
        let cleanup = match yielded.cleanup() {
            WorthQueryDirectConvergenceYieldCleanupOutcome::Complete(cleanup) => cleanup,
            WorthQueryDirectConvergenceYieldCleanupOutcome::RecoveryRequired(_) => {
                panic!("cooperative direct checkpoint cleanup must complete")
            }
        };

        assert_eq!(cleanup.identity(), epoch_identity);
        assert_eq!(cleanup.counters().yield_count(), 1);
        assert_eq!(cleanup.counters().readmission_count(), 0);
        assert_eq!(cleanup.counters().cleanup_attempt_count(), 1);
        assert_eq!(cleanup.counters().cleanup_completion_count(), 1);
        assert!(cleanup.incumbents().is_empty());
        assert!(cleanup.latest_report().is_none());
    });
}

#[test]
fn workflow_yield_cleanup_preserves_its_closed_epoch_receipt() {
    worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let yielded = workflow_yield(execution, "workflow-yield-cleanup", resource_request);
        let epoch_identity = yielded.epoch_identity().to_owned();
        let cleanup = match yielded.cleanup() {
            WorthQueryWorkflowConvergenceYieldCleanupOutcome::Complete(cleanup) => cleanup,
            WorthQueryWorkflowConvergenceYieldCleanupOutcome::Pending(_) => {
                panic!("artifact-free workflow checkpoint cleanup must not remain pending")
            }
            WorthQueryWorkflowConvergenceYieldCleanupOutcome::RecoveryRequired(_) => {
                panic!("cooperative workflow checkpoint cleanup must complete")
            }
        };

        assert_eq!(cleanup.identity(), epoch_identity);
        assert_eq!(cleanup.counters().yield_count(), 1);
        assert_eq!(cleanup.counters().readmission_count(), 0);
        assert_eq!(cleanup.counters().cleanup_attempt_count(), 1);
        assert_eq!(cleanup.counters().cleanup_completion_count(), 1);
        assert!(cleanup.incumbents().is_empty());
        assert!(cleanup.latest_report().is_none());
    });
}

#[test]
fn direct_yield_closed_recovery_counts_the_cleanup_as_complete() {
    worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let yielded = direct_yield_with_disposition(
            execution,
            FixtureDisposition::YieldThenCheckpointDropPanic,
            "direct-yield-closed-recovery",
            resource_request,
        );
        let receipt = match yielded.cleanup() {
            WorthQueryDirectConvergenceYieldCleanupOutcome::RecoveryRequired(receipt) => receipt,
            WorthQueryDirectConvergenceYieldCleanupOutcome::Complete(_) => {
                panic!("checkpoint drop panic claimed complete cleanup")
            }
        };

        assert_eq!(receipt.counters().cleanup_attempt_count(), 1);
        assert_eq!(receipt.counters().cleanup_completion_count(), 1);
        assert!(receipt.incumbents().is_empty());
        assert!(receipt.latest_report().is_none());
    });
}

#[test]
fn workflow_yield_closed_recovery_counts_the_cleanup_as_complete() {
    worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let yielded = workflow_yield_with_disposition(
            execution,
            FixtureDisposition::YieldThenCheckpointDropPanic,
            "workflow-yield-closed-recovery",
            resource_request,
        );
        let receipt = match yielded.cleanup() {
            WorthQueryWorkflowConvergenceYieldCleanupOutcome::RecoveryRequired(receipt) => receipt,
            WorthQueryWorkflowConvergenceYieldCleanupOutcome::Complete(_) => {
                panic!("checkpoint drop panic claimed complete cleanup")
            }
            WorthQueryWorkflowConvergenceYieldCleanupOutcome::Pending(_) => {
                panic!("artifact-free checkpoint drop panic claimed pending cleanup")
            }
        };

        assert_eq!(receipt.counters().cleanup_attempt_count(), 1);
        assert_eq!(receipt.counters().cleanup_completion_count(), 1);
        assert!(receipt.incumbents().is_empty());
        assert!(receipt.latest_report().is_none());
    });
}

#[test]
fn workflow_yield_pending_then_retry_counts_two_attempts_and_one_completion() {
    worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (yielded, artifact) = workflow_yield_with_pending_artifact(
            execution,
            "workflow-yield-pending",
            resource_request,
        );
        let borrowed = artifact
            .borrow("convergence cleanup pending proof")
            .expect("installed candidate contract must admit shared observation");
        let pending = match yielded.cleanup() {
            WorthQueryWorkflowConvergenceYieldCleanupOutcome::Pending(pending) => pending,
            _ => panic!("live artifact handle and borrow must keep workflow cleanup pending"),
        };
        assert_eq!(pending.counters().cleanup_attempt_count(), 1);
        assert_eq!(pending.counters().cleanup_completion_count(), 0);

        drop(borrowed);
        drop(artifact);
        let receipt = match pending.retry() {
            WorthQueryWorkflowConvergenceYieldCleanupOutcome::Complete(receipt) => receipt,
            _ => panic!("released artifact ownership must permit cleanup completion"),
        };
        assert_eq!(receipt.counters().cleanup_attempt_count(), 2);
        assert_eq!(receipt.counters().cleanup_completion_count(), 1);
    });
}

#[test]
fn same_scope_direct_cleanup_peers_keep_their_own_epochs() {
    worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let first = direct_yield(execution, "same-direct-cleanup-scope", resource_request);
        let second = direct_yield(execution, "same-direct-cleanup-scope", resource_request);
        let first_identity = first.epoch_identity().to_owned();
        let second_identity = second.epoch_identity().to_owned();
        assert_ne!(first_identity, second_identity);

        let second = match second.cleanup() {
            WorthQueryDirectConvergenceYieldCleanupOutcome::Complete(receipt) => receipt,
            WorthQueryDirectConvergenceYieldCleanupOutcome::RecoveryRequired(_) => {
                panic!("cooperative second direct peer must complete cleanup")
            }
        };
        let first = match first.cleanup() {
            WorthQueryDirectConvergenceYieldCleanupOutcome::Complete(receipt) => receipt,
            WorthQueryDirectConvergenceYieldCleanupOutcome::RecoveryRequired(_) => {
                panic!("cooperative first direct peer must complete cleanup")
            }
        };
        assert_eq!(first.identity(), first_identity);
        assert_eq!(second.identity(), second_identity);
        assert_eq!(first.counters().cleanup_attempt_count(), 1);
        assert_eq!(second.counters().cleanup_attempt_count(), 1);
    });
}

#[test]
fn same_stage_workflow_pending_and_complete_peers_do_not_cross_pair() {
    worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (first, artifact) = workflow_yield_with_pending_artifact(
            execution,
            "same-workflow-cleanup-scope",
            resource_request,
        );
        let borrowed = artifact
            .borrow("interleaved cleanup association proof")
            .expect("installed candidate contract must admit shared observation");
        let second = workflow_yield(execution, "same-workflow-cleanup-scope", resource_request);
        let first_identity = first.epoch_identity().to_owned();
        let second_identity = second.epoch_identity().to_owned();
        assert_ne!(first_identity, second_identity);

        let pending = match first.cleanup() {
            WorthQueryWorkflowConvergenceYieldCleanupOutcome::Pending(pending) => pending,
            _ => panic!("live first peer artifact must retain pending cleanup authority"),
        };
        let second = match second.cleanup() {
            WorthQueryWorkflowConvergenceYieldCleanupOutcome::Complete(receipt) => receipt,
            _ => panic!("artifact-free second workflow peer must complete cleanup"),
        };
        assert_eq!(pending.identity(), first_identity);
        assert_eq!(second.identity(), second_identity);
        assert_eq!(pending.counters().cleanup_attempt_count(), 1);
        assert_eq!(pending.counters().cleanup_completion_count(), 0);
        assert_eq!(second.counters().cleanup_attempt_count(), 1);
        assert_eq!(second.counters().cleanup_completion_count(), 1);

        drop(borrowed);
        drop(artifact);
        let first = match pending.retry() {
            WorthQueryWorkflowConvergenceYieldCleanupOutcome::Complete(receipt) => receipt,
            _ => panic!("rightful first peer must complete after its artifact owners close"),
        };
        assert_eq!(first.identity(), first_identity);
        assert_eq!(first.counters().cleanup_attempt_count(), 2);
        assert_eq!(first.counters().cleanup_completion_count(), 1);
    });
}

pub fn direct_yield(
    execution: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    call_identity: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryYieldedDirectConvergenceIteration {
    direct_yield_with_disposition(
        execution,
        FixtureDisposition::YieldThenConverged,
        call_identity,
        resource_request,
    )
}

pub fn direct_yield_with_disposition(
    execution: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    disposition: FixtureDisposition,
    call_identity: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryYieldedDirectConvergenceIteration {
    let DirectAdmissionFixture {
        runtime,
        operation,
        alternate_basis_operation: _,
        contract,
        managed,
        graph,
        bridge: _,
    } = direct_admission_fixture(disposition, resource_request);
    let epoch = runtime
        .admit_direct_convergence_epoch(&operation, contract, managed, graph)
        .unwrap_or_else(|_| panic!("direct convergence authorities must admit"))
        .start();
    let started = {
        let installed_fixture_phase = epoch.fixture_phase(execution);
        epoch.begin_iteration(&installed_fixture_phase, call(call_identity))
    }
    .unwrap_or_else(|_| panic!("direct convergence iteration must start"));
    let paused = {
        let fixture_phase_result = {
            let installed_fixture_phase = started.fixture_phase(execution);
            started.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryDirectConvergenceStepOutcome::Continue(paused) => paused,
            _ => panic!("direct yield fixture must reach a safe point"),
        }
    };
    match paused.yield_iteration() {
        WorthQueryDirectConvergenceYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("direct yield fixture must retain its exact epoch association"),
    }
}

pub fn workflow_yield(
    execution: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    call_identity: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryYieldedWorkflowConvergenceIteration {
    workflow_yield_with_disposition(
        execution,
        FixtureDisposition::YieldThenConverged,
        call_identity,
        resource_request,
    )
}

pub fn workflow_yield_with_disposition(
    execution: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    disposition: FixtureDisposition,
    call_identity: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryYieldedWorkflowConvergenceIteration {
    let WorkflowAdmissionFixture {
        runtime,
        operation,
        contract,
        managed,
        graph,
        bridge: _,
    } = workflow_admission_fixture(disposition, resource_request);
    let admitted = runtime
        .admit_workflow_convergence_epoch(&operation, contract, managed, graph)
        .unwrap_or_else(|_| panic!("workflow convergence authorities must admit"));
    let epoch = admitted
        .start()
        .unwrap_or_else(|_| panic!("workflow convergence epoch must start"));
    let started = {
        let installed_fixture_phase = epoch.fixture_phase(execution);
        epoch.begin_stage_iteration(
            &installed_fixture_phase,
            WORKFLOW_STAGE,
            call(call_identity),
        )
    }
    .unwrap_or_else(|_| panic!("workflow convergence iteration must start"));
    let paused = {
        let fixture_phase_result = {
            let installed_fixture_phase = started.fixture_phase(execution);
            started.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryWorkflowConvergenceStepOutcome::Continue(paused) => paused,
            _ => panic!("workflow yield fixture must reach a safe point"),
        }
    };
    match paused.yield_iteration() {
        WorthQueryWorkflowConvergenceYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("workflow yield fixture must retain its exact epoch association"),
    }
}

pub fn call(identity: &str) -> WorthQueryManagedGraphCallRequest {
    WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, identity)
}

#[path = "pending_artifact/mod.rs"]
pub mod pending_artifact;
