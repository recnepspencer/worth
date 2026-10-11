use super::super::support::*;
use worth_runtime_bridge::facade::{
    AdmittedBridgeAsyncCompletion, AdmittedBridgeAsyncRequestIdentity, BridgeAsyncDeniedCompletion,
    BridgeAsyncRequestTruthViewBasis, BridgeMixedCauseOrderingInput,
    BridgeMixedCauseOrderingLaneKind, BridgeMixedCauseOrderingRequest,
};
use worth_signal::facade::NodeId;

#[test]
fn bridge_denied_completion_classes_reach_query_without_synthetic_result_states() {
    assert_denied_kind(
        "bridge-async-rejected",
        NodeId::new(404, 0),
        worth_runtime_bridge::certification_for_test::reject_async_request_for_test,
        WorthQueryRuntimeAsyncResultStateKind::Failed,
    );
    assert_denied_kind(
        "bridge-async-cancelled",
        NodeId::new(405, 0),
        worth_runtime_bridge::certification_for_test::cancel_async_request_for_test,
        WorthQueryRuntimeAsyncResultStateKind::Cancelled,
    );
    assert_denied_kind(
        "bridge-async-superseded",
        NodeId::new(406, 0),
        |bridge, request| {
            worth_runtime_bridge::certification_for_test::supersede_async_request_for_test(
                bridge, request,
            )
            .0
        },
        WorthQueryRuntimeAsyncResultStateKind::Superseded,
    );
    assert_denied_kind(
        "bridge-async-lifecycle-denied",
        NodeId::new(407, 0),
        worth_runtime_bridge::certification_for_test::deny_oversized_async_completion_for_test,
        WorthQueryRuntimeAsyncResultStateKind::Denied,
    );
}

#[test]
fn bridge_cancellation_and_retry_lineage_reach_query_in_order() {
    let bridge = test_bridge();
    let request = worth_runtime_bridge::certification_for_test::retryable_async_request_for_test(
        &bridge,
        NodeId::new(408, 0),
        authoritative_async_basis("commit-a", "snapshot-a"),
    );
    let mut workspace =
        WorthQueryWorkspace::new("bridge-async-retry", stateful_bridge_task_runtime())
            .expect("valid Query workspace");
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .declare_bridge_async_live_view(
            "tasks.retry-async",
            task_live_request(),
            task_schema(),
            &request,
        )
        .expect("retry live view should declare");
    let (cancelled, retry) =
        worth_runtime_bridge::certification_for_test::cancel_and_retry_async_request_for_test(
            &bridge, &request,
        );
    let cancelled = admit_authoritative_input(
        &bridge,
        &mut workspace,
        &view,
        BridgeMixedCauseOrderingInput::AsyncDeniedCompletion(cancelled),
    );
    assert_eq!(
        kinds(&cancelled),
        vec![WorthQueryRuntimeAsyncResultStateKind::Cancelled]
    );
    let retried = admit_authoritative_input(
        &bridge,
        &mut workspace,
        &view,
        BridgeMixedCauseOrderingInput::AsyncRetryLineage(retry),
    );
    assert_eq!(
        kinds(&retried),
        vec![WorthQueryRuntimeAsyncResultStateKind::Retried]
    );
}

#[test]
fn bridge_backed_async_source_reaches_current_stale_revalidating_current() {
    let bridge = test_bridge();
    let basis_a = authoritative_async_basis("commit-a", "snapshot-a");
    let (request_a, completion_a) =
        admitted_async_request_and_completion(&bridge, NodeId::new(401, 0), basis_a, 64);
    let mut workspace =
        WorthQueryWorkspace::new("bridge-async-progression", stateful_bridge_task_runtime())
            .expect("valid Query workspace");
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .declare_bridge_async_live_view(
            "tasks.production-async",
            task_live_request(),
            task_schema(),
            &request_a,
        )
        .expect("live view should declare");
    let pending = workspace
        .take_bridge_async_initial_result(&view)
        .expect("initial Pending must be delivered once");
    assert_eq!(
        kinds(&pending),
        vec![WorthQueryRuntimeAsyncResultStateKind::Pending]
    );
    assert_eq!(
        pending.states()[0].basis_identity(),
        pending.expected_basis_identity()
    );
    assert_eq!(
        pending.states()[0].checkpoint_identity(),
        pending.expected_checkpoint_identity()
    );
    let replay = workspace
        .take_bridge_async_initial_result(&view)
        .expect_err("initial Pending delivery must be affine");
    assert_eq!(
        replay.kind(),
        WorthQueryAsyncSourceBindingErrorKind::InitialStateAlreadyDelivered
    );
    assert_eq!(
        workspace_async_kind(&workspace, &view),
        WorthQueryRuntimeAsyncResultStateKind::Pending
    );

    admit_initial_completion(&bridge, &mut workspace, &view, completion_a);
    let request_b = admit_revalidation(&bridge, &mut workspace, &view, &request_a);
    admit_refreshed_completion(&bridge, &mut workspace, &view, &request_b);
}

#[test]
fn async_source_binding_rejects_foreign_runtime() {
    let bridge = test_bridge();
    let foreign_bridge = test_bridge();
    let request = admitted_async_request(
        &bridge,
        NodeId::new(402, 0),
        authoritative_async_basis("commit-a", "snapshot-a"),
    );
    let mut runtime = stateful_bridge_task_runtime();
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_bridge_async_live_view(
            "tasks.hostile-async",
            task_live_request(),
            task_schema(),
            &request,
        )
        .expect("live view should declare");
    let (_, foreign_completion) = admitted_async_request_and_completion(
        &foreign_bridge,
        NodeId::new(402, 0),
        authoritative_async_basis("commit-a", "snapshot-a"),
        64,
    );
    let foreign_ordering =
        foreign_bridge.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
            BridgeMixedCauseOrderingLaneKind::Authoritative,
            vec![BridgeMixedCauseOrderingInput::AsyncCompletion(
                foreign_completion,
            )],
        ));
    let foreign = runtime
        .admit_bridge_async_result_transitions(&view, &foreign_ordering)
        .expect_err("equal-looking foreign Bridge evidence must fail");
    assert_eq!(
        foreign.kind(),
        WorthQueryAsyncSourceBindingErrorKind::ForeignBridgeRuntime
    );
    assert_eq!(
        runtime_async_kind(&runtime, &view),
        WorthQueryRuntimeAsyncResultStateKind::Pending
    );
}

fn assert_denied_kind(
    label: &str,
    node: NodeId,
    deny: impl FnOnce(
        &worth_runtime_bridge::facade::RuntimeBridge,
        &AdmittedBridgeAsyncRequestIdentity,
    ) -> BridgeAsyncDeniedCompletion,
    expected: WorthQueryRuntimeAsyncResultStateKind,
) {
    let bridge = test_bridge();
    let request = admitted_async_request(
        &bridge,
        node,
        authoritative_async_basis("commit-a", "snapshot-a"),
    );
    let mut workspace = WorthQueryWorkspace::new(label, stateful_bridge_task_runtime())
        .expect("valid Query workspace");
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .declare_bridge_async_live_view(
            format!("tasks.{label}"),
            task_live_request(),
            task_schema(),
            &request,
        )
        .expect("denied-state live view should declare");
    let denied = deny(&bridge, &request);
    let batch = admit_authoritative_input(
        &bridge,
        &mut workspace,
        &view,
        BridgeMixedCauseOrderingInput::AsyncDeniedCompletion(denied),
    );
    assert_eq!(kinds(&batch), vec![expected]);
    assert_eq!(workspace_async_kind(&workspace, &view), expected);
}

#[test]
fn duplicate_bridge_completion_is_suppressed_before_query_projection() {
    let bridge = test_bridge();
    let (request, completion) = admitted_async_request_and_completion(
        &bridge,
        NodeId::new(403, 0),
        authoritative_async_basis("commit-a", "snapshot-a"),
        64,
    );
    let mut runtime = stateful_bridge_task_runtime();
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_bridge_async_live_view(
            "tasks.duplicate-async",
            task_live_request(),
            task_schema(),
            &request,
        )
        .expect("live view should declare");
    let ordering = bridge.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![
            BridgeMixedCauseOrderingInput::AsyncCompletion(completion.clone()),
            BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
        ],
    ));
    let batch = runtime
        .admit_bridge_async_result_transitions(&view, &ordering)
        .expect("one completion should project");
    assert_eq!(batch.runtime_provenance(), runtime.runtime_provenance());
    assert_eq!(batch.view_name(), view.name());
    assert_eq!(batch.states().len(), 1);
    assert_eq!(batch.suppressed_duplicate_count(), 1);
    assert_eq!(
        runtime_async_kind(&runtime, &view),
        WorthQueryRuntimeAsyncResultStateKind::Current
    );
}

fn authoritative_async_basis(commit: &str, snapshot: &str) -> BridgeAsyncRequestTruthViewBasis {
    BridgeAsyncRequestTruthViewBasis::authoritative(
        TruthBranchIdentity::from_bridge_harness_label("truth-main"),
        TruthCommitIdentity::from_bridge_harness_label(commit),
        TruthSnapshotIdentity::from_bridge_harness_label(snapshot),
    )
}

fn kinds(
    batch: &WorthQueryAsyncResultTransitionBatch,
) -> Vec<WorthQueryRuntimeAsyncResultStateKind> {
    batch.states().iter().map(|state| state.kind()).collect()
}

fn admit_authoritative_input(
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
        .expect("admitted Bridge async input should reach Query")
}

fn admit_initial_completion(
    bridge: &RuntimeBridge,
    workspace: &mut WorthQueryWorkspace,
    view: &WorthQueryLiveView<WorthQueryUnrefinedLiveShape>,
    completion: AdmittedBridgeAsyncCompletion,
) {
    let current = admit_authoritative_input(
        bridge,
        workspace,
        view,
        BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
    );
    assert_eq!(
        kinds(&current),
        vec![WorthQueryRuntimeAsyncResultStateKind::Current]
    );
    assert_eq!(
        workspace_async_kind(workspace, view),
        WorthQueryRuntimeAsyncResultStateKind::Current
    );
}

fn admit_revalidation(
    bridge: &RuntimeBridge,
    workspace: &mut WorthQueryWorkspace,
    view: &WorthQueryLiveView<WorthQueryUnrefinedLiveShape>,
    request: &AdmittedBridgeAsyncRequestIdentity,
) -> AdmittedBridgeAsyncRequestIdentity {
    let revalidation = bridge
        .revalidate_async_request(request, authoritative_async_basis("commit-b", "snapshot-b"))
        .expect("Bridge should issue revalidation lineage");
    let next_request = revalidation.newer_request().clone();
    let refresh = admit_authoritative_input(
        bridge,
        workspace,
        view,
        BridgeMixedCauseOrderingInput::AsyncRevalidationLineage(revalidation),
    );
    assert_eq!(
        kinds(&refresh),
        vec![
            WorthQueryRuntimeAsyncResultStateKind::Stale,
            WorthQueryRuntimeAsyncResultStateKind::Revalidating,
        ]
    );
    assert_eq!(
        workspace_async_kind(workspace, view),
        WorthQueryRuntimeAsyncResultStateKind::Revalidating
    );
    next_request
}

fn admit_refreshed_completion(
    bridge: &RuntimeBridge,
    workspace: &mut WorthQueryWorkspace,
    view: &WorthQueryLiveView<WorthQueryUnrefinedLiveShape>,
    request: &AdmittedBridgeAsyncRequestIdentity,
) {
    let completion = admitted_async_completion_for_request(bridge, request, 72);
    let refreshed = admit_authoritative_input(
        bridge,
        workspace,
        view,
        BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
    );
    assert_eq!(
        kinds(&refreshed),
        vec![WorthQueryRuntimeAsyncResultStateKind::Current]
    );
    assert_eq!(
        workspace_async_kind(workspace, view),
        WorthQueryRuntimeAsyncResultStateKind::Current
    );
}

fn workspace_async_kind(
    workspace: &WorthQueryWorkspace,
    view: &WorthQueryLiveView<WorthQueryUnrefinedLiveShape>,
) -> WorthQueryRuntimeAsyncResultStateKind {
    workspace
        .state(view)
        .expect("async live state should remain independently inspectable")
        .async_result_state()
        .expect("async live state should retain a result posture")
        .kind()
}

fn runtime_async_kind(
    runtime: &WorthQueryRuntime,
    view: &WorthQueryLiveView<WorthQueryUnrefinedLiveShape>,
) -> WorthQueryRuntimeAsyncResultStateKind {
    <&WorthQueryLiveView<WorthQueryUnrefinedLiveShape> as WorthQueryRuntimeStateTarget>::into_state_snapshot(
        view, runtime,
    )
    .expect("async live state should remain independently inspectable")
    .async_result_state()
    .expect("async live state should retain a result posture")
    .kind()
}
