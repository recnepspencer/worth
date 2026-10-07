use super::installed_operation_fixture::{
    artifact_lease_workspace, artifact_move_workspace, bind_artifact_workflow, lease_intent,
    lease_intent_with_mode, move_intent,
};
use crate::workflow_request::workflow_request;
use worth_proof::TransitionOutcome;
use worth_query::facade::domain;
use worth_query::facade::runtime::ExecutionRequest;

#[test]
fn moved_artifact_borrows_once_and_disposes_exactly_once() {
    let (mut workspace, probe) = artifact_move_workspace("artifact-move").unwrap();
    let trace = bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(
            move_intent("produce"),
            &mut workspace,
            ExecutionRequest::serial(&workflow_request()),
        )
        .unwrap();

    assert_eq!(trace.stage_receipts().len(), 2);
    assert_eq!(probe.allocations(), 1);
    assert_eq!(probe.projection_calls(), 1);
    assert_eq!(probe.borrow_observations(), 1);
    assert_eq!(probe.disposals(), 1);
    let snapshots = probe.lifecycle_snapshots();
    assert_eq!(snapshots.len(), 3);
    assert_owner_counts(snapshots[0], 1, 0, 0);
    assert_owner_counts(snapshots[1], 1, 1, 0);
    assert_owner_counts(snapshots[2], 1, 0, 0);
    let counters = snapshots[2].counters();
    assert_eq!(counters.production_admissions, 1);
    assert_eq!(counters.owner_registrations, 1);
    assert_eq!(counters.transfer_admissions, 1);
    assert_eq!(counters.borrow_admissions, 1);
    assert_eq!(counters.lease_admissions, 0);
    assert_eq!(counters.provider_disposals, 0);
}

#[test]
fn distinct_retained_leases_borrow_and_release_the_same_owner_generation_safely() {
    let (mut workspace, probe) = artifact_lease_workspace("artifact-leases").unwrap();
    let trace = bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(
            lease_intent(),
            &mut workspace,
            ExecutionRequest::serial(&workflow_request()),
        )
        .unwrap();

    assert_eq!(trace.stage_receipts().len(), 3);
    assert_eq!(probe.allocations(), 1);
    assert_eq!(probe.projection_calls(), 1);
    assert_eq!(probe.borrow_observations(), 2);
    assert_eq!(probe.disposals(), 1);
    let snapshots = probe.lifecycle_snapshots();
    assert_eq!(snapshots.len(), 6);
    for window in snapshots.chunks_exact(3) {
        assert_owner_counts(window[0], 1, 0, 1);
        assert_owner_counts(window[1], 1, 1, 1);
        assert_owner_counts(window[2], 1, 0, 1);
    }
    let counters = snapshots[5].counters();
    assert_eq!(counters.production_admissions, 1);
    assert_eq!(counters.owner_registrations, 1);
    assert_eq!(counters.transfer_admissions, 0);
    assert_eq!(counters.borrow_admissions, 2);
    assert_eq!(counters.lease_admissions, 2);
    assert_eq!(counters.provider_disposals, 0);
}

#[test]
fn completed_consumer_denial_precedes_a_second_lease_mutation() {
    let (mut workspace, probe) =
        artifact_lease_workspace("artifact-completed-consumer-lease").unwrap();
    let run = bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
        .unwrap()
        .advance(
            "produce",
            domain::WorthQueryWorkflowValue::Text("retain-observer-lease".into()),
            &mut workspace,
            ExecutionRequest::serial(&workflow_request()),
        )
        .unwrap()
        .advance_with_artifact_lease(
            "observe-a",
            "produce",
            "observer-a",
            &mut workspace,
            ExecutionRequest::serial(&workflow_request()),
        )
        .unwrap();
    let retained = probe
        .take_escaped_lease()
        .expect("producer retained an independent observation lease");
    let before = retained.owner_snapshot();
    assert_eq!(before.counters().lease_admissions, 2);

    let denial = match run.advance_with_artifact_lease(
        "observe-a",
        "produce",
        "forbidden-retry",
        &mut workspace,
        ExecutionRequest::serial(&workflow_request()),
    ) {
        TransitionOutcome::Denied(denial) => denial,
        _ => panic!("completed artifact consumer did not deny"),
    };
    assert_eq!(
        denial.kind(),
        &domain::WorthQueryWorkflowAdvanceDenialKind::StageAlreadyCompleted
    );
    assert_eq!(denial.counters().stage_executor_contacts, 2);
    assert_eq!(probe.borrow_observations(), 1);
    let after = retained.owner_snapshot();
    assert_eq!(after.counters().lease_admissions, 2);
    assert_eq!(after.counters().transfer_admissions, 0);
}

#[test]
fn replacement_disposes_the_prior_owner_and_eventually_the_successor_once_each() {
    let (mut workspace, probe) = artifact_move_workspace("artifact-replacement").unwrap();
    bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(
            move_intent("replace"),
            &mut workspace,
            ExecutionRequest::serial(&workflow_request()),
        )
        .unwrap();

    assert_eq!(probe.allocations(), 2);
    assert_eq!(probe.projection_calls(), 2);
    assert_eq!(probe.replacements(), 1);
    assert_eq!(probe.disposals(), 2);
}

#[test]
fn explicit_cancellation_returns_a_cancelled_receipt_and_disposes_once() {
    let (mut workspace, probe) = artifact_move_workspace("artifact-cancel").unwrap();
    let outcome = bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(
            move_intent("cancel"),
            &mut workspace,
            ExecutionRequest::serial(&workflow_request()),
        );

    assert!(matches!(
        outcome,
        TransitionOutcome::Denied(_) | TransitionOutcome::Failed(_)
    ));
    assert_eq!(probe.cancellations(), 1);
    assert_eq!(probe.allocations(), 1);
    assert_eq!(probe.disposals(), 1);
}

#[test]
fn declared_failure_after_registration_releases_the_owned_resource() {
    let (mut workspace, probe) = artifact_move_workspace("artifact-failure-cleanup").unwrap();
    let outcome = bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(
            move_intent("fail-after-production"),
            &mut workspace,
            ExecutionRequest::serial(&workflow_request()),
        );

    assert!(matches!(
        outcome,
        TransitionOutcome::Denied(_) | TransitionOutcome::Failed(_)
    ));
    assert_eq!(probe.allocations(), 1);
    assert_eq!(probe.projection_calls(), 1);
    assert_eq!(probe.disposals(), 1);
}

#[test]
fn declared_failure_after_transfer_releases_the_owned_resource() {
    let (mut workspace, probe) = artifact_move_workspace("artifact-transfer-failure").unwrap();
    let outcome = bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(
            move_intent("fail-after-transfer"),
            &mut workspace,
            ExecutionRequest::serial(&workflow_request()),
        );

    assert!(matches!(
        outcome,
        TransitionOutcome::Denied(_) | TransitionOutcome::Failed(_)
    ));
    assert_eq!(probe.allocations(), 1);
    assert_eq!(probe.projection_calls(), 1);
    assert_eq!(probe.borrow_observations(), 1);
    assert_eq!(probe.disposals(), 1);
}

#[test]
fn declared_failure_after_lease_transfer_releases_lease_and_owner_exactly_once() {
    let (mut workspace, probe) =
        artifact_lease_workspace("artifact-lease-transfer-failure").unwrap();
    let outcome = bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(
            lease_intent_with_mode("fail-after-lease-transfer"),
            &mut workspace,
            ExecutionRequest::serial(&workflow_request()),
        );

    assert!(matches!(
        outcome,
        TransitionOutcome::Denied(_) | TransitionOutcome::Failed(_)
    ));
    assert_eq!(probe.allocations(), 1);
    assert_eq!(probe.projection_calls(), 1);
    assert_eq!(probe.borrow_observations(), 1);
    assert_eq!(probe.disposals(), 1);
    let snapshots = probe.lifecycle_snapshots();
    assert_eq!(snapshots.len(), 3);
    assert_owner_counts(snapshots[0], 1, 0, 1);
    assert_owner_counts(snapshots[1], 1, 1, 1);
    assert_owner_counts(snapshots[2], 1, 0, 1);
}

#[test]
fn escaped_transferred_handle_is_revoked_and_disposed_by_the_run_registry() {
    let (mut workspace, probe) = artifact_move_workspace("artifact-transfer-escape").unwrap();
    let outcome = bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(
            move_intent("escape-after-transfer"),
            &mut workspace,
            ExecutionRequest::serial(&workflow_request()),
        );

    assert!(matches!(
        outcome,
        TransitionOutcome::Denied(_) | TransitionOutcome::Failed(_)
    ));
    assert_eq!(probe.disposals(), 1);
    let escaped = probe
        .take_escaped_handle()
        .expect("consumer retained the transferred handle for sabotage");
    let snapshot = escaped.owner_snapshot();
    assert_owner_counts(snapshot, 0, 0, 0);
    assert!(snapshot.is_disposed());
    assert_eq!(snapshot.counters().provider_disposals, 1);
    let denial = match escaped.borrow("post-run-sabotage") {
        Err(denial) => denial,
        Ok(_) => panic!("run-closed escaped handle admitted a borrow"),
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryArtifactDenialKind::AlreadyDisposed
    );
    drop(escaped);
    assert_eq!(probe.disposals(), 1);
}

#[test]
fn preparation_producer_and_consumer_panics_each_dispose_exactly_once() {
    for (mode, expected_resources, expected_borrows) in [
        ("panic-during-projection", 1, 0),
        ("panic-after-production", 1, 0),
        ("panic-after-transfer", 1, 1),
        ("panic-during-replacement", 2, 0),
    ] {
        let (mut workspace, probe) =
            artifact_move_workspace(&format!("artifact-unwind-{mode}")).unwrap();
        let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = bind_artifact_workflow(&workspace)
                .admit_workflow_resources(
                    crate::suite::installed_operation_fixture::execution_resource_request(),
                    &workspace,
                )
                .unwrap()
                .reexecute(
                    move_intent(mode),
                    &mut workspace,
                    ExecutionRequest::serial(&workflow_request()),
                );
        }));

        assert!(unwind.is_err(), "{mode} did not unwind");
        assert_eq!(probe.allocations(), expected_resources, "{mode}");
        assert_eq!(probe.projection_calls(), expected_resources, "{mode}");
        assert_eq!(probe.borrow_observations(), expected_borrows, "{mode}");
        assert_eq!(probe.disposals(), expected_resources, "{mode}");
    }
}

fn assert_owner_counts(
    snapshot: domain::WorthQueryArtifactOwnerSnapshot,
    owners: usize,
    borrows: usize,
    leases: usize,
) {
    assert_eq!(snapshot.owner_count(), owners);
    assert_eq!(snapshot.borrow_count(), borrows);
    assert_eq!(snapshot.lease_count(), leases);
}
