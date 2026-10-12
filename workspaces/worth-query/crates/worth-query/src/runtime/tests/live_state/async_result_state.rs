use super::super::support::*;
use crate::runtime::async_result_state::runtime_async_checkpoint_label_identity;

fn bridge_projection_for(
    kind: WorthQueryRuntimeAsyncResultStateKind,
) -> WorthQueryRuntimeAsyncResultProjection {
    match kind {
        WorthQueryRuntimeAsyncResultStateKind::Pending => {
            WorthQueryRuntimeAsyncResultProjection::pending("async:pending")
        }
        WorthQueryRuntimeAsyncResultStateKind::Current => {
            WorthQueryRuntimeAsyncResultProjection::completion_state(
                BridgeAsyncCompletionState::Admitted(BridgeAsyncCompletionClass::Fulfilled),
                "async:current",
            )
        }
        WorthQueryRuntimeAsyncResultStateKind::Failed => {
            WorthQueryRuntimeAsyncResultProjection::completion_state(
                BridgeAsyncCompletionState::Denied(BridgeAsyncCompletionDenialClass::Rejected),
                "async:failed",
            )
        }
        WorthQueryRuntimeAsyncResultStateKind::Stale => {
            WorthQueryRuntimeAsyncResultProjection::completion_state(
                BridgeAsyncCompletionState::Denied(BridgeAsyncCompletionDenialClass::StaleDenied),
                "async:stale",
            )
        }
        WorthQueryRuntimeAsyncResultStateKind::Cancelled => {
            WorthQueryRuntimeAsyncResultProjection::completion_state(
                BridgeAsyncCompletionState::Denied(BridgeAsyncCompletionDenialClass::Cancelled),
                "async:cancelled",
            )
        }
        WorthQueryRuntimeAsyncResultStateKind::Retried => {
            WorthQueryRuntimeAsyncResultProjection::forward_causality(
                BridgeAsyncForwardCausalityClass::RetryAfterTimeout,
                "async:retried",
            )
        }
        WorthQueryRuntimeAsyncResultStateKind::Revalidating => {
            WorthQueryRuntimeAsyncResultProjection::forward_causality(
                BridgeAsyncForwardCausalityClass::RevalidationAfterTruthBasisDrift,
                "async:revalidating",
            )
        }
        WorthQueryRuntimeAsyncResultStateKind::Superseded => {
            WorthQueryRuntimeAsyncResultProjection::supersession("async:superseded")
        }
        WorthQueryRuntimeAsyncResultStateKind::Denied => {
            WorthQueryRuntimeAsyncResultProjection::completion_state(
                BridgeAsyncCompletionState::Denied(
                    BridgeAsyncCompletionDenialClass::SignalLifecycleDenied,
                ),
                "async:denied",
            )
        }
        WorthQueryRuntimeAsyncResultStateKind::Unresolved => {
            WorthQueryRuntimeAsyncResultProjection::completion_state(
                BridgeAsyncCompletionState::Admitted(
                    BridgeAsyncCompletionClass::EffectsIndeterminate,
                ),
                "async:unresolved",
            )
        }
    }
}

#[test]
fn runtime_state_and_inspection_project_async_result_state_parity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.async-state",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    let (basis_identity, checkpoint_identity) =
        live_subscription_async_identity(&runtime, view.name());
    let cases = [
        (
            WorthQueryRuntimeAsyncResultStateKind::Pending,
            WorthQueryRuntimeStateKind::Pending,
        ),
        (
            WorthQueryRuntimeAsyncResultStateKind::Current,
            WorthQueryRuntimeStateKind::Ready,
        ),
        (
            WorthQueryRuntimeAsyncResultStateKind::Failed,
            WorthQueryRuntimeStateKind::Failed,
        ),
        (
            WorthQueryRuntimeAsyncResultStateKind::Stale,
            WorthQueryRuntimeStateKind::Stale,
        ),
        (
            WorthQueryRuntimeAsyncResultStateKind::Cancelled,
            WorthQueryRuntimeStateKind::Cancelled,
        ),
        (
            WorthQueryRuntimeAsyncResultStateKind::Retried,
            WorthQueryRuntimeStateKind::Retried,
        ),
        (
            WorthQueryRuntimeAsyncResultStateKind::Revalidating,
            WorthQueryRuntimeStateKind::Revalidating,
        ),
        (
            WorthQueryRuntimeAsyncResultStateKind::Superseded,
            WorthQueryRuntimeStateKind::Superseded,
        ),
        (
            WorthQueryRuntimeAsyncResultStateKind::Denied,
            WorthQueryRuntimeStateKind::Denied,
        ),
        (
            WorthQueryRuntimeAsyncResultStateKind::Unresolved,
            WorthQueryRuntimeStateKind::Unresolved,
        ),
    ];

    for (kind, expected_state_kind) in cases {
        let projection = bridge_projection_for(kind);
        let projected = runtime
            .project_async_result_state(
                view.name(),
                &projection,
                &basis_identity,
                &checkpoint_identity,
            )
            .expect("async result state should project");
        let state =
            <&WorthQueryLiveView<WorthQueryUnrefinedLiveShape> as WorthQueryRuntimeStateTarget>::into_state_snapshot(&view, request_execution, &runtime)
            .expect("state should snapshot");
        let inspection = runtime
            .inspect_live_view_explanation(&view)
            .expect("inspection should succeed");

        assert_eq!(state.kind(), expected_state_kind);
        assert_eq!(state.async_result_state(), Some(&projected));
        assert!(state.explanation().contains(kind.as_str()));
        assert_eq!(inspection.async_result_state(), Some(&projected));
    }
}

#[test]
fn runtime_async_result_state_preserves_runtime_and_replay_parity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime_a = stateful_bridge_task_runtime(request_execution);
    let view_a: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime_a
        .declare_live_view(
            request_execution,
            "tasks.async-replay",
            task_live_request(),
            task_schema(),
        )
        .expect("first live view should declare");
    let mut runtime_b = stateful_bridge_task_runtime(request_execution);
    let view_b: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime_b
        .declare_live_view(
            request_execution,
            "tasks.async-replay",
            task_live_request(),
            task_schema(),
        )
        .expect("second live view should declare");

    let (basis_a, checkpoint_a) = live_subscription_async_identity(&runtime_a, view_a.name());
    let (basis_b, checkpoint_b) = live_subscription_async_identity(&runtime_b, view_b.name());
    assert_eq!(basis_a, basis_b);
    assert_eq!(checkpoint_a, checkpoint_b);

    let projection = bridge_projection_for(WorthQueryRuntimeAsyncResultStateKind::Current);
    let runtime_state = runtime_a
        .project_async_result_state(view_a.name(), &projection, &basis_a, &checkpoint_a)
        .expect("runtime-backed async projection should succeed");
    let replay_state = runtime_b
        .project_async_result_state(view_b.name(), &projection, &basis_b, &checkpoint_b)
        .expect("replayed async projection should succeed");

    assert_eq!(
        runtime_state.result_state_for_reporting(),
        replay_state.result_state_for_reporting()
    );
    assert_eq!(
        <&WorthQueryLiveView<WorthQueryUnrefinedLiveShape> as WorthQueryRuntimeStateTarget>::into_state_snapshot(&view_a, request_execution, &runtime_a)
        .expect("runtime-backed state should snapshot")
        .async_result_state()
        .expect("runtime-backed async result state should exist")
        .result_state_for_reporting(),
        <&WorthQueryLiveView<WorthQueryUnrefinedLiveShape> as WorthQueryRuntimeStateTarget>::into_state_snapshot(&view_b, request_execution, &runtime_b)
        .expect("replayed state should snapshot")
        .async_result_state()
        .expect("replayed async result state should exist")
        .result_state_for_reporting()
    );
}

#[test]
fn runtime_async_result_state_fails_closed_for_generation_drift_and_preview_mismatch() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.async-drift",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    let (basis_identity, checkpoint_identity) =
        live_subscription_async_identity(&runtime, view.name());

    let drifted_checkpoint = runtime_async_checkpoint_label_identity("generation:drifted");
    let drifted_basis = runtime_async_checkpoint_label_identity("basis:drifted");

    let generation_drift = runtime
        .project_async_result_state(
            view.name(),
            &WorthQueryRuntimeAsyncResultProjection::completion_state(
                BridgeAsyncCompletionState::Admitted(BridgeAsyncCompletionClass::Fulfilled),
                "async:generation-drift",
            ),
            &basis_identity,
            &drifted_checkpoint,
        )
        .expect_err("current generation drift should deny");
    let preview_mismatch = runtime
        .project_async_result_state(
            view.name(),
            &WorthQueryRuntimeAsyncResultProjection::completion_state(
                BridgeAsyncCompletionState::Admitted(BridgeAsyncCompletionClass::Fulfilled),
                "async:preview-mismatch",
            ),
            &drifted_basis,
            &checkpoint_identity,
        )
        .expect_err("current preview mismatch should deny");
    let superseded = runtime
        .project_async_result_state(
            view.name(),
            &WorthQueryRuntimeAsyncResultProjection::supersession("async:superseded-drift"),
            &basis_identity,
            &drifted_checkpoint,
        )
        .expect("superseded generation drift should stay typed");
    let denied = runtime
        .project_async_result_state(
            view.name(),
            &WorthQueryRuntimeAsyncResultProjection::completion_state(
                BridgeAsyncCompletionState::Denied(
                    BridgeAsyncCompletionDenialClass::SignalLifecycleDenied,
                ),
                "async:denied-preview",
            ),
            &drifted_basis,
            &checkpoint_identity,
        )
        .expect("denied preview mismatch should stay typed");

    match generation_drift {
        WorthQueryRuntimeError::LiveSubscriptionInstallation { stage, message, .. } => {
            assert_eq!(stage, "async-result-state");
            assert!(message.contains("GenerationDriftRequiresTypedState"));
        }
        other => panic!("expected generation drift denial, got {other:?}"),
    }
    match preview_mismatch {
        WorthQueryRuntimeError::LiveSubscriptionInstallation { stage, message, .. } => {
            assert_eq!(stage, "async-result-state");
            assert!(message.contains("PreviewBasisMismatchRequiresTypedState"));
        }
        other => panic!("expected preview mismatch denial, got {other:?}"),
    }

    let state = <&WorthQueryLiveView<WorthQueryUnrefinedLiveShape> as WorthQueryRuntimeStateTarget>::into_state_snapshot(&view, request_execution, &runtime)
    .expect("state should snapshot");
    let inspection = runtime
        .inspect_live_view_explanation(&view)
        .expect("inspection should succeed");
    assert_eq!(state.kind(), WorthQueryRuntimeStateKind::Denied);
    assert_eq!(state.async_result_state(), Some(&denied));
    assert_eq!(inspection.async_result_state(), Some(&denied));
    assert_eq!(superseded.checkpoint_identity(), &drifted_checkpoint);
    assert_eq!(denied.basis_identity(), &drifted_basis);
}
