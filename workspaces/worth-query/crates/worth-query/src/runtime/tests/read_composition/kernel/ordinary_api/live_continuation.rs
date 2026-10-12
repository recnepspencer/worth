use super::live::{closed, one_shot_task_result, open_task_resource, task_workspace};
use crate::ordinary::live::{
    WorthQueryManagedLiveCheckpointOutcome, WorthQueryManagedLiveContinuationDurability,
    WorthQueryManagedLiveDeliveryCauseKind, WorthQueryManagedLiveResumeNextAction,
    WorthQueryManagedLiveResumeOutcome, WorthQueryManagedLiveResumeStopKind,
};
use crate::ordinary_outcome::{
    WorthQueryOrdinaryRuntimeAsyncPostureKind, WorthQueryOrdinaryRuntimeBasisPostureKind,
    WorthQueryOrdinaryRuntimeCausePostureKind, WorthQueryOrdinaryRuntimePostureKind,
};
use crate::runtime::async_result_state::runtime_async_checkpoint_label_identity;
use crate::runtime::tests::support::*;
use crate::runtime::{WorthQueryAuthorityLane, WorthQueryWorkspace};

#[test]
fn checkpoint_resume_replays_queued_delivery_and_preserves_one_shot_meaning() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_workspace(request_execution, "managed-live-resume-replay");
    let handle = open_task_resource(request_execution, &mut workspace, "tasks.resume-replay");
    let checkpoint = checkpointed(handle.checkpoint(&mut workspace));

    workspace
        .write(request_execution, task_insert("Arrived while suspended"))
        .expect("write should remain Query-routed while the continuation owns the resource");

    let resumed = resumed(checkpoint.resume(&mut workspace));
    assert_eq!(resumed.receipt().pending_delivery_batch_count(), 1);
    assert_eq!(resumed.receipt().resumed_delivery_sequence(), Some(1));
    let handle = resumed.into_handle();
    let live = handle
        .read(request_execution, &mut workspace)
        .expect("resumed managed resource should remain readable");
    let one_shot = one_shot_task_result(request_execution, &mut workspace);
    assert_eq!(live.rows(), one_shot.rows());

    let delivery = handle
        .drain(&mut workspace)
        .expect("queued delivery should drain after resume");
    assert_eq!(delivery.batches().len(), 1);
    assert_eq!(
        delivery.batches()[0].cause_kind(),
        WorthQueryManagedLiveDeliveryCauseKind::RelationalChange
    );
    assert!(closed(handle.close(&mut workspace)).lane_terminal());
}

#[test]
fn checkpoint_and_resume_receipts_preserve_delivery_already_waiting_at_checkpoint() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_workspace(
        request_execution,
        "managed-live-checkpoint-pending-delivery",
    );
    let handle = open_task_resource(
        request_execution,
        &mut workspace,
        "tasks.checkpoint-pending-delivery",
    );
    workspace
        .write(request_execution, task_insert("Queued before checkpoint"))
        .expect("write should queue a delivery before checkpoint");

    let continuation = checkpointed(handle.checkpoint(&mut workspace));
    assert_eq!(continuation.checkpoint().pending_delivery_batch_count(), 1);
    assert_eq!(continuation.checkpoint().last_delivery_sequence(), Some(1));
    assert_eq!(
        continuation.checkpoint().durability(),
        WorthQueryManagedLiveContinuationDurability::RuntimeBound
    );

    let resumed = resumed(continuation.resume(&mut workspace));
    assert_eq!(resumed.receipt().pending_delivery_batch_count(), 1);
    assert_eq!(resumed.receipt().resumed_delivery_sequence(), Some(1));
    let handle = resumed.into_handle();
    assert_eq!(
        handle
            .drain(&mut workspace)
            .expect("pre-checkpoint delivery should remain drainable")
            .batches()
            .len(),
        1
    );
    assert!(closed(handle.close(&mut workspace)).lane_terminal());
}

#[test]
fn foreign_workspace_resume_stops_without_consuming_recovery_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut owner = task_workspace(request_execution, "managed-live-resume-owner");
    let mut foreign = task_workspace(request_execution, "managed-live-resume-foreign");
    let continuation = checkpointed(
        open_task_resource(request_execution, &mut owner, "tasks.resume-owner")
            .checkpoint(&mut owner),
    );

    let stop = stopped(continuation.resume(&mut foreign));
    assert_eq!(
        stop.kind(),
        WorthQueryManagedLiveResumeStopKind::ForeignWorkspace
    );
    assert_eq!(
        stop.next_action(),
        WorthQueryManagedLiveResumeNextAction::UseOwningWorkspace
    );
    let resumed = resumed(stop.into_continuation().resume(&mut owner));
    assert!(closed(resumed.into_handle().close(&mut owner)).lane_terminal());
}

#[test]
fn policy_drift_requires_authority_rebind_before_resume() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = policy_drift_workspace(request_execution);
    let continuation = checkpointed(
        open_task_resource(
            request_execution,
            &mut workspace,
            "tasks.resume-policy-drift",
        )
        .checkpoint(&mut workspace),
    );

    let stop = stopped(continuation.resume(&mut workspace));
    assert_eq!(
        stop.kind(),
        WorthQueryManagedLiveResumeStopKind::AuthorityRebindRequired
    );
    assert_eq!(
        stop.next_action(),
        WorthQueryManagedLiveResumeNextAction::RebindAuthority
    );
    assert!(matches!(
        stop.close(&mut workspace),
        crate::ordinary::live::WorthQueryManagedLiveCloseOutcome::Closed(_)
    ));
}

#[test]
fn stale_basis_cannot_cross_a_managed_resume_boundary() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_workspace(request_execution, "managed-live-resume-stale-basis");
    let continuation = checkpointed(
        open_task_resource(
            request_execution,
            &mut workspace,
            "tasks.resume-stale-basis",
        )
        .checkpoint(&mut workspace),
    );
    let name = continuation.checkpoint().resource_name().to_string();
    let (_, generation) = live_subscription_async_identity(&workspace.runtime, &name);
    let drifted_basis = runtime_async_checkpoint_label_identity("basis:drifted-after-checkpoint");
    workspace
        .runtime
        .project_async_result_state(
            &name,
            &WorthQueryRuntimeAsyncResultProjection::completion_state(
                BridgeAsyncCompletionState::Denied(
                    BridgeAsyncCompletionDenialClass::SignalLifecycleDenied,
                ),
                "async:stale-resume",
            ),
            &drifted_basis,
            &generation,
        )
        .expect("hostile async basis drift should project");

    let stop = stopped(continuation.resume(&mut workspace));
    assert_eq!(stop.kind(), WorthQueryManagedLiveResumeStopKind::StaleBasis);
    assert_eq!(
        stop.next_action(),
        WorthQueryManagedLiveResumeNextAction::SupplyFreshBasis
    );
    assert!(matches!(
        stop.close(&mut workspace),
        crate::ordinary::live::WorthQueryManagedLiveCloseOutcome::Closed(_)
    ));
}

#[test]
fn managed_observation_keeps_temporal_and_async_causes_distinct() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_workspace(request_execution, "managed-live-causal-observation");
    let handle = open_task_resource(
        request_execution,
        &mut workspace,
        "tasks.causal-observation",
    );
    workspace
        .runtime
        .emit_time_only_delivery(
            handle.name(),
            crate::subscription::QuerySubscriptionDeliveryCauseKind::WindowEntry,
            "tick:managed-window-entry",
            false,
            true,
        )
        .expect("temporal delivery should emit");
    let (basis, generation) = live_subscription_async_identity(&workspace.runtime, handle.name());
    workspace
        .runtime
        .project_async_result_state(
            handle.name(),
            &WorthQueryRuntimeAsyncResultProjection::completion_state(
                BridgeAsyncCompletionState::Admitted(BridgeAsyncCompletionClass::Fulfilled),
                "async:managed-current",
            ),
            &basis,
            &generation,
        )
        .expect("async state should project");

    let observation = handle
        .observe(&mut workspace)
        .expect("managed causal posture should be observable");
    assert_eq!(
        observation.runtime_posture().kind(),
        WorthQueryOrdinaryRuntimePostureKind::Current
    );
    assert_eq!(
        observation.runtime_posture().cause_posture(),
        WorthQueryOrdinaryRuntimeCausePostureKind::TimeOnly
    );
    assert_eq!(
        observation.runtime_posture().async_posture(),
        Some(WorthQueryOrdinaryRuntimeAsyncPostureKind::Current)
    );
    assert_eq!(
        observation.runtime_posture().basis_posture(),
        WorthQueryOrdinaryRuntimeBasisPostureKind::Stable
    );
    let delivery = handle
        .drain(&mut workspace)
        .expect("typed temporal delivery should drain");
    assert_eq!(
        delivery.batches()[0].cause_kind(),
        WorthQueryManagedLiveDeliveryCauseKind::Temporal
    );
    assert!(closed(handle.close(&mut workspace)).lane_terminal());
}

#[test]
fn preview_binding_never_moves_managed_authority_or_delivery_into_preview_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_workspace(request_execution, "managed-live-preview-isolation");
    let handle = open_task_resource(request_execution, &mut workspace, "tasks.preview-isolated");
    let binding = {
        let mut preview = workspace
            .preview(
                request_execution,
                test_session_label("managed live preview isolation"),
            )
            .expect("preview should open");
        let binding = preview.use_view(handle.view());
        assert!(preview.discard().discarded());
        binding
    };

    assert_eq!(
        binding.source_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert_eq!(
        binding.preview_lane(),
        WorthQueryAuthorityLane::PreviewTruth
    );
    let observation = handle
        .observe(&mut workspace)
        .expect("authoritative managed resource should survive preview discard");
    assert_eq!(
        observation.authority_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert!(handle
        .drain(&mut workspace)
        .expect("preview discard must not leak delivery")
        .is_empty());
    assert!(closed(handle.close(&mut workspace)).lane_terminal());
}

#[test]
fn abandoned_continuation_is_reaped_before_subsequent_runtime_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_workspace(request_execution, "managed-live-abandoned-continuation");
    let continuation = checkpointed(
        open_task_resource(
            request_execution,
            &mut workspace,
            "tasks.abandoned-continuation",
        )
        .checkpoint(&mut workspace),
    );
    drop(continuation);

    assert!(workspace
        .resolve_live_artifact_target("tasks.abandoned-continuation")
        .is_err());
    let receipt = workspace
        .write(
            request_execution,
            task_insert("After abandoned continuation"),
        )
        .expect("subsequent work should reap an abandoned continuation first");
    assert!(receipt
        .terminal_affected_live_view_ids_projection()
        .is_empty());
}

fn checkpointed(
    outcome: WorthQueryManagedLiveCheckpointOutcome,
) -> crate::ordinary::live::WorthQueryManagedLiveContinuation {
    match outcome {
        WorthQueryManagedLiveCheckpointOutcome::Checkpointed(completion) => {
            completion.into_continuation()
        }
        WorthQueryManagedLiveCheckpointOutcome::Stopped(stop) => {
            panic!(
                "managed checkpoint unexpectedly stopped: {:?}",
                stop.error()
            )
        }
    }
}

fn resumed(
    outcome: WorthQueryManagedLiveResumeOutcome,
) -> crate::ordinary::live::WorthQueryManagedLiveResumeCompletion {
    match outcome {
        WorthQueryManagedLiveResumeOutcome::Resumed(completion) => completion,
        WorthQueryManagedLiveResumeOutcome::Stopped(stop) => {
            panic!("managed resume unexpectedly stopped: {:?}", stop.kind())
        }
    }
}

fn stopped(
    outcome: WorthQueryManagedLiveResumeOutcome,
) -> crate::ordinary::live::WorthQueryManagedLiveResumeStop {
    match outcome {
        WorthQueryManagedLiveResumeOutcome::Stopped(stop) => stop,
        WorthQueryManagedLiveResumeOutcome::Resumed(_) => {
            panic!("hostile managed resume unexpectedly succeeded")
        }
    }
}

fn task_insert(title: &str) -> crate::runtime::WorthQueryWriteCommand {
    insert_command(
        "Task",
        [
            ("identity.id", test_string_aspect_value("")),
            ("title.value", test_string_aspect_value(title)),
        ],
    )
}

fn policy_drift_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryWorkspace {
    complete_backend_from_parts_builder()
        .subscription_activation(RemaskingSubscriptionActivation {
            projection: WorthQueryRuntimeRemaskProjection::remasked(
                WorthQueryRuntimeRemaskReasonKind::PolicyDrift,
                "policy:drifted",
                "tenant-truth:stable",
                "tenant-schema:stable",
                "relationship-proof:stable",
                "schema-context:stable",
            ),
        })
        .build_backend_from_parts()
        .build(request_execution)
        .expect("policy-drift runtime should build")
        .workspace("managed-live-policy-drift")
        .expect("policy-drift workspace should open")
}
