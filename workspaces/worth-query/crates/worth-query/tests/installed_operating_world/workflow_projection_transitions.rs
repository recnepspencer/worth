use worth_query::facade::domain;

use super::installed_operation_fixture::{
    controlled_workflow_workspace, failing_controlled_workflow_workspace,
    failing_workflow_workspace, workflow_workspace, GeometryDomain,
};
use super::workflow_projection_lifecycle::{promoted, settle_workflow};

#[test]
fn workflow_terminal_paths_preserve_exact_close_semantics() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace =
        workflow_workspace(request_execution, "workflow-lifecycle-terminal").unwrap();
    let (settled, _) = settle_workflow(&mut workspace);
    let live = promoted(request_execution, settled, &mut workspace);
    let resource = live.resource_name().to_string();
    let cancelled = match live.cancel(&mut workspace) {
        domain::WorthQueryWorkflowProjectionCancellationOutcome::Cancelled(cancelled) => cancelled,
        domain::WorthQueryWorkflowProjectionCancellationOutcome::Stopped(_) => {
            panic!("workflow cancellation unexpectedly stopped")
        }
    };
    assert_eq!(
        cancelled.close_receipt().cause(),
        domain::WorthQueryProjectionLifecycleCloseCause::Cancellation
    );
    assert_eq!(
        cancelled
            .close_receipt()
            .ordinary()
            .closeout_kind()
            .as_str(),
        "consumer_terminated"
    );
    assert!(workspace.resolve_live_artifact_target(&resource).is_err());
    assert_eq!(
        cancelled
            .dispose()
            .close_receipt()
            .ordinary()
            .closeout_kind()
            .as_str(),
        "consumer_terminated"
    );

    let (settled, _) = settle_workflow(&mut workspace);
    let live = promoted(request_execution, settled, &mut workspace);
    let disposed = match live.dispose(&mut workspace) {
        domain::WorthQueryWorkflowProjectionDisposalOutcome::Disposed(disposed) => disposed,
        domain::WorthQueryWorkflowProjectionDisposalOutcome::Stopped(_) => {
            panic!("workflow disposal unexpectedly stopped")
        }
    };
    assert_eq!(
        disposed.close_receipt().cause(),
        domain::WorthQueryProjectionLifecycleCloseCause::Disposal
    );
}

#[test]
fn workflow_replacement_readmits_the_exact_pair_and_retains_transition_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace =
        workflow_workspace(request_execution, "workflow-lifecycle-replacement").unwrap();
    let (settled, _) = settle_workflow(&mut workspace);
    let live = promoted(request_execution, settled, &mut workspace);
    let (candidate, _) = settle_workflow(&mut workspace);
    let candidate = candidate.into_lifecycle();
    let witness = live.replacement_witness_for(&candidate).unwrap();
    let replaced = match live.replace_with(request_execution, candidate, witness, &mut workspace) {
        domain::WorthQueryWorkflowProjectionReplacementOutcome::Replaced(replaced) => replaced,
        _ => panic!("exact workflow replacement pair did not converge"),
    };
    assert_eq!(replaced.transition_work().compatibility_readmissions(), 1);
    assert_eq!(
        replaced.predecessor_close_receipt().cause(),
        domain::WorthQueryProjectionLifecycleCloseCause::Replacement
    );
    replaced.refresh(request_execution, &mut workspace).unwrap();
    let disposed = match replaced.dispose(&mut workspace) {
        domain::WorthQueryTransitionedWorkflowProjectionDisposalOutcome::Disposed(disposed) => {
            disposed
        }
        domain::WorthQueryTransitionedWorkflowProjectionDisposalOutcome::Stopped(_) => {
            panic!("replaced workflow disposal stopped")
        }
    };
    assert!(matches!(
        disposed.prior_transition(),
        Some(domain::WorthQueryProjectionPriorTransitionEvidence::Replacement { .. })
    ));

    let (settled, _) = settle_workflow(&mut workspace);
    let live = promoted(request_execution, settled, &mut workspace);
    let (candidate, _) = settle_workflow(&mut workspace);
    let candidate = candidate.into_lifecycle();
    let (wrong, _) = settle_workflow(&mut workspace);
    let wrong = wrong.into_lifecycle();
    let wrong_witness = live.replacement_witness_for(&wrong).unwrap();
    let stop = match live.replace_with(request_execution, candidate, wrong_witness, &mut workspace)
    {
        domain::WorthQueryWorkflowProjectionReplacementOutcome::Stopped(stop) => stop,
        _ => panic!("wrong-pair workflow replacement witness was accepted"),
    };
    assert_eq!(
        stop.kind(),
        domain::WorthQueryProjectionTransitionDenialKind::WrongCompatibilityPair
    );
    assert_eq!(stop.work().candidate().planning_attempts, 0);
    let (live, _) = stop.into_retry_parts();
    live.refresh(request_execution, &mut workspace).unwrap();
}

#[test]
fn workflow_cleanup_pending_owns_both_resources_and_supports_retry_and_rollback() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace =
        failing_workflow_workspace(request_execution, "workflow-cleanup-retry", 1).unwrap();
    let (settled, _) = settle_workflow(&mut workspace);
    let live = promoted(request_execution, settled, &mut workspace);
    let (candidate, _) = settle_workflow(&mut workspace);
    let candidate = candidate.into_lifecycle();
    let witness = live.replacement_witness_for(&candidate).unwrap();
    let pending = match live.replace_with(request_execution, candidate, witness, &mut workspace) {
        domain::WorthQueryWorkflowProjectionReplacementOutcome::CleanupPending(pending) => pending,
        _ => panic!("injected workflow close failure did not retain cleanup ownership"),
    };
    workspace
        .resolve_live_artifact_target(pending.predecessor_resource_name())
        .unwrap();
    workspace
        .resolve_live_artifact_target(pending.successor_resource_name())
        .unwrap();
    assert!(
        pending
            .replacement_witness()
            .counters()
            .canonical_comparisons
            > 0
    );
    let replaced = match pending.retry_cleanup(&mut workspace) {
        domain::WorthQueryWorkflowReplacementCleanupRetryOutcome::Replaced(replaced) => replaced,
        domain::WorthQueryWorkflowReplacementCleanupRetryOutcome::Pending(_) => {
            panic!("workflow cleanup retry did not recover")
        }
    };
    assert_eq!(replaced.cleanup_work().predecessor_close_attempts(), 2);

    let mut workspace =
        failing_workflow_workspace(request_execution, "workflow-cleanup-rollback", 1).unwrap();
    let (settled, _) = settle_workflow(&mut workspace);
    let live = promoted(request_execution, settled, &mut workspace);
    let (candidate, _) = settle_workflow(&mut workspace);
    let candidate = candidate.into_lifecycle();
    let witness = live.replacement_witness_for(&candidate).unwrap();
    let pending = match live.replace_with(request_execution, candidate, witness, &mut workspace) {
        domain::WorthQueryWorkflowProjectionReplacementOutcome::CleanupPending(pending) => pending,
        _ => panic!("workflow rollback fixture did not enter cleanup pending"),
    };
    let restored = match pending.rollback(&mut workspace) {
        domain::WorthQueryWorkflowReplacementRollbackOutcome::Restored {
            live,
            receipt,
            work,
        } => {
            assert_eq!(
                receipt.cause(),
                domain::WorthQueryProjectionLifecycleCloseCause::ReplacementRollback
            );
            assert_eq!(work.rollback_close_completions(), 1);
            live
        }
        domain::WorthQueryWorkflowReplacementRollbackOutcome::Pending(_) => {
            panic!("workflow rollback did not close the successor")
        }
    };
    restored.refresh(request_execution, &mut workspace).unwrap();
}

#[test]
fn workflow_rebind_requires_the_exact_stale_current_pair() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace =
        controlled_workflow_workspace(request_execution, "workflow-lifecycle-rebind").unwrap();
    let prior = workspace.domain(GeometryDomain).unwrap();
    let (settled, _) = settle_workflow(&mut workspace);
    let live = promoted(request_execution, settled, &mut workspace);
    workspace.advance_domain_installation_generation().unwrap();
    let (_, receipt) = workspace
        .rebind_domain(prior.rebind_request())
        .unwrap()
        .into_parts();
    let (candidate, _) = settle_workflow(&mut workspace);
    let candidate = candidate.into_lifecycle();
    let witness = live.rebind_witness_for(&candidate, receipt).unwrap();
    let rebound = match live.rebind_with(request_execution, candidate, witness, &mut workspace) {
        domain::WorthQueryWorkflowProjectionRebindOutcome::Rebound(rebound) => rebound,
        _ => panic!("exact workflow rebind pair did not converge"),
    };
    assert_eq!(rebound.transition_work().compatibility_readmissions(), 1);
    assert_eq!(
        rebound.predecessor_close_receipt().cause(),
        domain::WorthQueryProjectionLifecycleCloseCause::Rebind
    );
    rebound.refresh(request_execution, &mut workspace).unwrap();
    let cancelled = match rebound.cancel(&mut workspace) {
        domain::WorthQueryTransitionedWorkflowProjectionCancellationOutcome::Cancelled(
            cancelled,
        ) => cancelled,
        domain::WorthQueryTransitionedWorkflowProjectionCancellationOutcome::Stopped(_) => {
            panic!("rebound workflow cancellation stopped")
        }
    };
    assert!(matches!(
        cancelled.prior_transition(),
        Some(domain::WorthQueryProjectionPriorTransitionEvidence::Rebind { .. })
    ));
}

#[test]
fn workflow_rebind_cleanup_failure_retains_both_resources_until_retry() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = failing_controlled_workflow_workspace(
        request_execution,
        "workflow-rebind-cleanup-retry",
        1,
    )
    .unwrap();
    let prior = workspace.domain(GeometryDomain).unwrap();
    let (settled, _) = settle_workflow(&mut workspace);
    let live = promoted(request_execution, settled, &mut workspace);
    workspace.advance_domain_installation_generation().unwrap();
    let (_, receipt) = workspace
        .rebind_domain(prior.rebind_request())
        .unwrap()
        .into_parts();
    let (candidate, _) = settle_workflow(&mut workspace);
    let candidate = candidate.into_lifecycle();
    let witness = live.rebind_witness_for(&candidate, receipt).unwrap();
    let pending = match live.rebind_with(request_execution, candidate, witness, &mut workspace) {
        domain::WorthQueryWorkflowProjectionRebindOutcome::CleanupPending(pending) => pending,
        _ => panic!("injected rebind close failure did not retain both workflow resources"),
    };
    workspace
        .resolve_live_artifact_target(pending.predecessor_resource_name())
        .unwrap();
    workspace
        .resolve_live_artifact_target(pending.successor_resource_name())
        .unwrap();
    assert!(pending.rebind_witness().counters().canonical_comparisons > 0);
    let rebound = match pending.retry_cleanup(&mut workspace) {
        domain::WorthQueryWorkflowRebindCleanupRetryOutcome::Rebound(rebound) => rebound,
        domain::WorthQueryWorkflowRebindCleanupRetryOutcome::Pending(_) => {
            panic!("workflow rebind cleanup retry did not recover")
        }
    };
    assert_eq!(rebound.cleanup_work().predecessor_close_attempts(), 2);
    assert_eq!(
        rebound.predecessor_close_receipt().cause(),
        domain::WorthQueryProjectionLifecycleCloseCause::Rebind
    );
}
