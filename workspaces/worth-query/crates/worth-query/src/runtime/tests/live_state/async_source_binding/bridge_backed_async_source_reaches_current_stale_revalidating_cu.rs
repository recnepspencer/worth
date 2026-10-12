//! bridge backed async source reaches current stale revalidating current.
use super::*;

#[test]
fn bridge_backed_async_source_reaches_current_stale_revalidating_current() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let bridge = test_bridge();
    let basis_a = authoritative_async_basis("commit-a", "snapshot-a");
    let (request_a, completion_a) =
        admitted_async_request_and_completion(&bridge, NodeId::new(401, 0), basis_a, 64);
    let mut workspace = WorthQueryWorkspace::new(
        "bridge-async-progression",
        stateful_bridge_task_runtime(request_execution),
    )
    .expect("valid Query workspace");
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .declare_bridge_async_live_view(
            request_execution,
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
        workspace_async_kind(request_execution, &workspace, &view),
        WorthQueryRuntimeAsyncResultStateKind::Pending
    );

    admit_initial_completion(
        request_execution,
        &bridge,
        &mut workspace,
        &view,
        completion_a,
    );
    let request_b = admit_revalidation(
        request_execution,
        &bridge,
        &mut workspace,
        &view,
        &request_a,
    );
    admit_refreshed_completion(
        request_execution,
        &bridge,
        &mut workspace,
        &view,
        &request_b,
    );
}
