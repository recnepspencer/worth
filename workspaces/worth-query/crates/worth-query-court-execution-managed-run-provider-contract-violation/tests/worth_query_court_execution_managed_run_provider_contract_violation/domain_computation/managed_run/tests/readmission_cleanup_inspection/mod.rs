use worth_query_execution::court_doors_for_test::domain_computation::managed_run::readmission::WorthQueryWorkflowReadmissionCleanupOutcome;
use worth_query_execution::court_doors_for_test::domain_computation::managed_run::readmission::WorthQueryWorkflowReadmissionCleanupPending;
use worth_query_execution::court_doors_for_test::domain_computation::managed_run::readmission::WorthQueryWorkflowReadmissionCleanupPendingInspection;
use worth_query_execution::court_doors_for_test::domain_computation::managed_run::readmission::WorthQueryWorkflowReadmissionCleanupRequired;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::readmission_workflow::yielded_workflow_with_retained_artifact;
use crate::domain_computation::managed_run::tests::readmission_workflow_association::shared_yielded_workflow_peers_with_provider;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::yield_fixture::YieldProvider;
use crate::process::domain_computation::managed_run::tests::*;
use worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionCleanupInspection;
use worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionCleanupReceipt;
use worth_query_execution::court_doors_for_test::domain_computation::managed_run::readmission::WorthQueryWorkflowReadmissionCleanupInspection;
use worth_query_execution::court_doors_for_test::domain_computation::managed_run::readmission::WorthQueryWorkflowReadmissionCleanupReceipt;

#[test]
fn completed_readmission_cleanup_receipts_are_exact_inspection_newtypes() {
    assert_eq!(
        std::mem::size_of::<WorthQueryDirectReadmissionCleanupReceipt>(),
        std::mem::size_of::<WorthQueryDirectReadmissionCleanupInspection>()
    );
    assert_eq!(
        std::mem::size_of::<WorthQueryWorkflowReadmissionCleanupReceipt>(),
        std::mem::size_of::<WorthQueryWorkflowReadmissionCleanupInspection>()
    );
}

#[test]
fn workflow_cleanup_pending_peers_keep_exact_yielded_association_through_rightful_retry() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (first, second, bridge, runtime) = shared_yielded_workflow_peers_with_provider(
            execution,
            YieldProvider::checkpoint_restore_panic(5),
            resource_request,
        );
        let first_yielded = first.inspection().clone();
        let second_yielded = second.inspection().clone();
        let first = pending_workflow_cleanup(workflow_recovery_cleanup(
            execution, first, &runtime, &bridge,
        ));
        let second = pending_workflow_cleanup(workflow_recovery_cleanup(
            execution, second, &runtime, &bridge,
        ));
        assert_workflow_pending_association(first.inspection(), &first_yielded);
        assert_workflow_pending_association(second.inspection(), &second_yielded);
        assert_ne!(
            first.inspection().yielded_attempt_identity(),
            second.inspection().yielded_attempt_identity()
        );

        let second = complete_workflow_retry(second);
        let first = complete_workflow_retry(first);
        assert_workflow_cleanup_association(first.inspection(), &first_yielded);
        assert_workflow_cleanup_association(second.inspection(), &second_yielded);
    });
}

#[test]
fn retained_artifact_is_the_only_pending_axis_and_retry_observes_its_release() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (yielded, bridge, runtime, _producer, artifact) =
            yielded_workflow_with_retained_artifact(
                execution,
                YieldProvider::checkpoint_restore_panic(7),
                resource_request,
            );
        let yielded_inspection = yielded.inspection().clone();
        let borrowed = artifact
            .borrow("readmission cleanup retained artifact")
            .expect("installed artifact contract must admit a surviving borrow");
        let pending =
            match workflow_recovery_cleanup(execution, yielded, &runtime, &bridge).finish() {
                WorthQueryWorkflowReadmissionCleanupOutcome::Pending(
                    pending,
                ) => pending,
                _ => panic!("retained artifact must keep workflow cleanup pending"),
            };
        let inspection = pending.inspection();
        assert!(inspection.artifact_cleanup_pending());
        assert!(!inspection.bridge_cleanup_pending());
        assert_eq!(inspection.artifact_evidence().retained_artifact_count(), 1);
        assert_eq!(
            inspection.logical_run_identity(),
            yielded_inspection.logical_run_identity()
        );
        assert_eq!(
            inspection.checkpoint().identity(),
            yielded_inspection.checkpoint().identity()
        );

        drop(borrowed);
        let receipt = complete_workflow_retry(pending);
        let inspection = receipt.inspection();
        assert_eq!(inspection.artifact_evidence().retained_artifact_count(), 0);
        assert_eq!(inspection.artifact_evidence().disposed_artifact_count(), 1);
        assert_workflow_cleanup_association(inspection, &yielded_inspection);
    });
}

pub fn workflow_recovery_cleanup(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    yielded: worth_query_execution::facade::domain_computation::WorthQueryYieldedWorkflowRun,
    runtime: &WorthQueryExecutionRuntime,
    bridge: &RuntimeBridge,
) -> WorthQueryWorkflowReadmissionCleanupRequired {
    let active_request = execution;

    {
        let fixture_phase_result = {
            let installed_fixture_phase = yielded.fixture_phase_for_test(active_request);
            yielded.readmit_same_runtime(&installed_fixture_phase, runtime, bridge)
        };
        match fixture_phase_result {
        worth_query_execution::facade::domain_computation::WorthQueryWorkflowReadmissionOutcome::RecoveryRequired(
            worth_query_execution::facade::domain_computation::WorthQueryWorkflowReadmissionRecoveryRequired::TerminalCleanup(
                recovery,
            ),
        ) => recovery.into_cleanup(),
        _ => panic!("restore panic must produce workflow terminal cleanup"),
    }
    }
}

pub fn pending_workflow_cleanup(
    cleanup: WorthQueryWorkflowReadmissionCleanupRequired,
) -> WorthQueryWorkflowReadmissionCleanupPending {
    std::thread::spawn(move || match cleanup.finish() {
        WorthQueryWorkflowReadmissionCleanupOutcome::Pending(pending) => pending,
        _ => panic!("foreign-thread workflow cleanup must retain Bridge retry authority"),
    })
    .join()
    .expect("workflow cleanup probe must return Pending")
}

pub fn complete_workflow_retry(
    pending: WorthQueryWorkflowReadmissionCleanupPending,
) -> WorthQueryWorkflowReadmissionCleanupReceipt {
    match pending.retry() {
        WorthQueryWorkflowReadmissionCleanupOutcome::Complete(receipt) => receipt,
        _ => panic!("rightful workflow cleanup retry must complete"),
    }
}

pub fn assert_workflow_pending_association(
    cleanup: &WorthQueryWorkflowReadmissionCleanupPendingInspection,
    yielded: &worth_query_execution::facade::domain_computation::WorthQueryYieldedWorkflowRunInspection,
) {
    assert_eq!(
        cleanup.logical_run_identity(),
        yielded.logical_run_identity()
    );
    assert_eq!(
        cleanup.yielded_attempt_identity(),
        yielded.yielded_attempt_identity()
    );
    assert_eq!(
        cleanup.provider_session_identity(),
        yielded.provider_session_identity()
    );
    assert_eq!(
        cleanup.checkpoint().identity(),
        yielded.checkpoint().identity()
    );
    assert!(!cleanup.resource_plan_identity().is_empty());
    assert!(!cleanup.artifact_cleanup_pending());
    assert!(cleanup.bridge_cleanup_pending());
}

pub fn assert_workflow_cleanup_association(
    cleanup: &WorthQueryWorkflowReadmissionCleanupInspection,
    yielded: &worth_query_execution::facade::domain_computation::WorthQueryYieldedWorkflowRunInspection,
) {
    assert_eq!(
        cleanup.logical_run_identity(),
        yielded.logical_run_identity()
    );
    assert_eq!(
        cleanup.yielded_attempt_identity(),
        yielded.yielded_attempt_identity()
    );
    assert_eq!(
        cleanup.provider_session_identity(),
        yielded.provider_session_identity()
    );
    assert_eq!(
        cleanup.checkpoint().identity(),
        yielded.checkpoint().identity()
    );
    assert!(!cleanup.resource_plan_identity().is_empty());
    assert!(cleanup.resources_released());
    assert_eq!(cleanup.released_reservation_count(), 3);
}
