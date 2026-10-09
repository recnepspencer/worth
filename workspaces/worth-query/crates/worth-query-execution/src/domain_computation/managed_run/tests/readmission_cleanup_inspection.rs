use super::readmission_workflow::yielded_workflow_with_retained_artifact;
use super::readmission_workflow_association::shared_yielded_workflow_peers_with_provider;
use super::yield_fixture::YieldProvider;
use super::*;
use crate::domain_computation::{
    WorthQueryDirectReadmissionCleanupInspection, WorthQueryDirectReadmissionCleanupReceipt,
    WorthQueryWorkflowReadmissionCleanupInspection, WorthQueryWorkflowReadmissionCleanupReceipt,
};

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
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let (first, second, bridge, runtime) = shared_yielded_workflow_peers_with_provider(
            execution,
            YieldProvider::checkpoint_restore_panic(5),
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
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let (yielded, bridge, runtime, _producer, artifact) =
            yielded_workflow_with_retained_artifact(
                execution,
                YieldProvider::checkpoint_restore_panic(7),
            );
        let yielded_inspection = yielded.inspection().clone();
        let borrowed = artifact
            .borrow("readmission cleanup retained artifact")
            .expect("installed artifact contract must admit a surviving borrow");
        let pending =
            match workflow_recovery_cleanup(execution, yielded, &runtime, &bridge).finish() {
                crate::domain_computation::WorthQueryWorkflowReadmissionCleanupOutcome::Pending(
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

fn workflow_recovery_cleanup(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    yielded: crate::domain_computation::WorthQueryYieldedWorkflowRun,
    runtime: &WorthQueryExecutionRuntime,
    bridge: &RuntimeBridge,
) -> crate::domain_computation::WorthQueryWorkflowReadmissionCleanupRequired {
    let active_request = execution;

    match yielded.readmit_same_runtime(active_request, runtime, bridge) {
        crate::domain_computation::WorthQueryWorkflowReadmissionOutcome::RecoveryRequired(
            crate::domain_computation::WorthQueryWorkflowReadmissionRecoveryRequired::TerminalCleanup(
                recovery,
            ),
        ) => recovery.into_cleanup(),
        _ => panic!("restore panic must produce workflow terminal cleanup"),
    }
}

fn pending_workflow_cleanup(
    cleanup: crate::domain_computation::WorthQueryWorkflowReadmissionCleanupRequired,
) -> crate::domain_computation::WorthQueryWorkflowReadmissionCleanupPending {
    std::thread::spawn(move || match cleanup.finish() {
        crate::domain_computation::WorthQueryWorkflowReadmissionCleanupOutcome::Pending(
            pending,
        ) => pending,
        _ => panic!("foreign-thread workflow cleanup must retain Bridge retry authority"),
    })
    .join()
    .expect("workflow cleanup probe must return Pending")
}

fn complete_workflow_retry(
    pending: crate::domain_computation::WorthQueryWorkflowReadmissionCleanupPending,
) -> crate::domain_computation::WorthQueryWorkflowReadmissionCleanupReceipt {
    match pending.retry() {
        crate::domain_computation::WorthQueryWorkflowReadmissionCleanupOutcome::Complete(
            receipt,
        ) => receipt,
        _ => panic!("rightful workflow cleanup retry must complete"),
    }
}

fn assert_workflow_pending_association(
    cleanup: &crate::domain_computation::WorthQueryWorkflowReadmissionCleanupPendingInspection,
    yielded: &crate::domain_computation::WorthQueryYieldedWorkflowRunInspection,
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

fn assert_workflow_cleanup_association(
    cleanup: &crate::domain_computation::WorthQueryWorkflowReadmissionCleanupInspection,
    yielded: &crate::domain_computation::WorthQueryYieldedWorkflowRunInspection,
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
