use super::super::support::*;
use worth_runtime_bridge::facade::{
    BridgeAsyncRequestTruthViewBasis, BridgeMixedCauseOrderingInput,
    BridgeMixedCauseOrderingLaneKind, BridgeMixedCauseOrderingRequest,
};
use worth_signal::facade::NodeId;

#[test]
fn late_result_retains_basis_drift_against_the_current_bridge_binding() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let basis_a = authoritative_basis("commit-a", "snapshot-a");
    let basis_b = authoritative_basis("commit-b", "snapshot-b");
    let batch = late_result_after_revalidation(request_execution, basis_a, basis_b);
    let state = batch.states().last().expect("late result must project");

    assert_ne!(state.basis_identity(), batch.expected_basis_identity());
    assert_ne!(
        state.checkpoint_identity(),
        batch.expected_checkpoint_identity()
    );
}

#[test]
fn late_result_retains_generation_drift_when_the_truth_basis_is_stable() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let basis = authoritative_basis("commit-stable", "snapshot-stable");
    let batch = late_result_after_retry(request_execution, basis);
    let state = batch.states().last().expect("late result must project");

    assert_eq!(state.basis_identity(), batch.expected_basis_identity());
    assert_ne!(
        state.checkpoint_identity(),
        batch.expected_checkpoint_identity()
    );
}

fn late_result_after_retry(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    basis: BridgeAsyncRequestTruthViewBasis,
) -> WorthQueryAsyncResultTransitionBatch {
    let bridge = test_bridge();
    let request = worth_runtime_bridge::certification::retryable_async_request(
        &bridge,
        NodeId::new(410, 0),
        basis,
    );
    let mut workspace = WorthQueryWorkspace::new(
        "bridge-async-generation-drift",
        stateful_bridge_task_runtime(request_execution),
    )
    .expect("valid Query workspace");
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .declare_bridge_async_live_view(
            request_execution,
            "tasks.production-async-generation-drift",
            task_live_request(),
            task_schema(),
            &request,
        )
        .expect("generation-drift live view should declare");
    let (cancelled, retry) =
        worth_runtime_bridge::certification::cancel_and_retry_async_request(&bridge, &request);
    let cancelled = admit_input(
        &bridge,
        &mut workspace,
        &view,
        BridgeMixedCauseOrderingInput::AsyncDeniedCompletion(cancelled),
    );
    assert_eq!(
        cancelled.states()[0].kind(),
        WorthQueryRuntimeAsyncResultStateKind::Cancelled
    );
    let retried = admit_input(
        &bridge,
        &mut workspace,
        &view,
        BridgeMixedCauseOrderingInput::AsyncRetryLineage(retry),
    );
    assert_eq!(
        retried.states()[0].kind(),
        WorthQueryRuntimeAsyncResultStateKind::Retried
    );
    let late =
        worth_runtime_bridge::certification::observe_late_async_completion(&bridge, &request);
    admit_input(
        &bridge,
        &mut workspace,
        &view,
        BridgeMixedCauseOrderingInput::AsyncDeniedCompletion(late),
    )
}

fn late_result_after_revalidation(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    original_basis: BridgeAsyncRequestTruthViewBasis,
    replacement_basis: BridgeAsyncRequestTruthViewBasis,
) -> WorthQueryAsyncResultTransitionBatch {
    let bridge = test_bridge();
    let (request, completion) =
        admitted_async_request_and_completion(&bridge, NodeId::new(409, 0), original_basis, 64);
    let mut workspace = WorthQueryWorkspace::new(
        "bridge-async-drift",
        stateful_bridge_task_runtime(request_execution),
    )
    .expect("valid Query workspace");
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .declare_bridge_async_live_view(
            request_execution,
            "tasks.production-async-drift",
            task_live_request(),
            task_schema(),
            &request,
        )
        .expect("drift live view should declare");
    let current = admit_input(
        &bridge,
        &mut workspace,
        &view,
        BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
    );
    assert_eq!(
        current.states()[0].kind(),
        WorthQueryRuntimeAsyncResultStateKind::Current
    );
    let revalidation = bridge
        .revalidate_async_request(&request, replacement_basis)
        .expect("Bridge must issue revalidation lineage");
    let revalidating = admit_input(
        &bridge,
        &mut workspace,
        &view,
        BridgeMixedCauseOrderingInput::AsyncRevalidationLineage(revalidation),
    );
    assert_eq!(
        revalidating
            .states()
            .iter()
            .map(WorthQueryRuntimeAsyncResultState::kind)
            .collect::<Vec<_>>(),
        [
            WorthQueryRuntimeAsyncResultStateKind::Stale,
            WorthQueryRuntimeAsyncResultStateKind::Revalidating,
        ]
    );
    let late =
        worth_runtime_bridge::certification::observe_late_async_completion(&bridge, &request);
    admit_input(
        &bridge,
        &mut workspace,
        &view,
        BridgeMixedCauseOrderingInput::AsyncDeniedCompletion(late),
    )
}

fn admit_input(
    bridge: &RuntimeBridge,
    workspace: &mut WorthQueryWorkspace,
    view: &WorthQueryLiveView<WorthQueryUnrefinedLiveShape>,
    input: BridgeMixedCauseOrderingInput,
) -> WorthQueryAsyncResultTransitionBatch {
    let ordering = bridge.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![input],
    ));
    workspace
        .admit_bridge_async_result_transitions(view, &ordering)
        .expect("Bridge-issued transition must reach Query")
}

fn authoritative_basis(commit: &str, snapshot: &str) -> BridgeAsyncRequestTruthViewBasis {
    BridgeAsyncRequestTruthViewBasis::authoritative(
        TruthBranchIdentity::from_bridge_harness_label("truth-main"),
        TruthCommitIdentity::from_bridge_harness_label(commit),
        TruthSnapshotIdentity::from_bridge_harness_label(snapshot),
    )
}
