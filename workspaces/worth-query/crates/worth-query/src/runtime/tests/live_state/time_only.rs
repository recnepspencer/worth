use super::super::support::*;
use crate::program::{
    WorthQueryAuthorityRequirement, WorthQueryOperation, WorthQueryProgram, WorthQueryProgramEffect,
};
use crate::subscription::QuerySubscriptionDeliveryCauseKind;

fn drain_only_program(view_name: &str) -> WorthQueryProgram {
    WorthQueryProgram::new(
        "time-only.delivery.program",
        [WorthQueryOperation::new("drain_time_only")
            .requires(WorthQueryAuthorityRequirement::Live)
            .with_effect(WorthQueryProgramEffect::DrainPatches {
                view_name: view_name.to_string(),
            })],
    )
    .expect("time-only drain program should build")
}

#[test]
fn runtime_time_only_delivery_is_canonical_without_relational_patch() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.time-only",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");

    let emitted = runtime
        .emit_time_only_delivery(
            view.name(),
            QuerySubscriptionDeliveryCauseKind::FreshnessOnly,
            "tick:freshness-only",
            false,
            true,
        )
        .expect("time-only delivery should emit");
    let drained = runtime.drain_patches(&view);

    assert_eq!(
        emitted.delivery_cause_kind(),
        QuerySubscriptionDeliveryCauseKind::FreshnessOnly
    );
    assert!(!emitted.has_relational_patch());
    assert_eq!(
        emitted.patch_group_kind(),
        QueryPatchGroupKind::TimeOnlyDeliveryGroup
    );
    assert_eq!(drained.query_delivery_batches.len(), 1);
    assert_eq!(
        drained.query_delivery_batches[0].delivery_cause_kind(),
        QuerySubscriptionDeliveryCauseKind::FreshnessOnly
    );
    assert!(!drained.query_delivery_batches[0].has_relational_patch());
}

#[test]
fn runtime_state_and_inspection_retain_last_time_only_delivery_after_drain() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.inspect-time-only",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");

    let emitted = runtime
        .emit_time_only_delivery(
            view.name(),
            QuerySubscriptionDeliveryCauseKind::WindowEntry,
            "tick:window-entry",
            false,
            true,
        )
        .expect("time-only delivery should emit");
    let _ = runtime.drain_patches(&view);

    let state = <&WorthQueryLiveView<WorthQueryUnrefinedLiveShape> as WorthQueryRuntimeStateTarget>::into_state_snapshot(&view, request_execution, &runtime)
    .expect("state should snapshot");
    let inspection = runtime
        .inspect_live_view_explanation(&view)
        .expect("inspection should retain live state");

    assert!(state.explanation().contains("window_entry"));
    assert!(state.explanation().contains("relational_patch=false"));
    assert_eq!(
        state.result_shape_for_reporting(),
        view.subscription_installation()
            .view_shape_projection()
            .label()
    );
    assert_eq!(
        inspection.last_delivery_cause_kind(),
        Some(QuerySubscriptionDeliveryCauseKind::WindowEntry)
    );
    assert_eq!(
        inspection
            .last_delivery_cause_projection()
            .as_ref()
            .map(|projection| projection.label().as_str()),
        Some(emitted.delivery_cause_for_reporting())
    );
    assert!(!inspection.last_delivery_had_relational_patch());
}

#[test]
fn runtime_time_only_delivery_matches_program_drain_lane_and_denies_missing_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.program-time-only",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");

    runtime
        .emit_time_only_delivery(
            view.name(),
            QuerySubscriptionDeliveryCauseKind::Deadline,
            "tick:deadline-direct",
            false,
            true,
        )
        .expect("direct time-only delivery should emit");
    let direct = runtime.drain_patches(&view).query_delivery_batches;

    runtime
        .emit_time_only_delivery(
            view.name(),
            QuerySubscriptionDeliveryCauseKind::Deadline,
            "tick:deadline-program",
            false,
            true,
        )
        .expect("program time-only delivery should emit");
    let program = drain_only_program(view.name());
    let installed = runtime
        .install_program(program)
        .expect("program should install");
    let operation = installed
        .operation("drain_time_only")
        .expect("operation should resolve");
    let receipt = runtime
        .run_operation_with_request(request_execution, operation, Vec::new())
        .expect("program should drain patches");
    let program_batches = &receipt.patch_batches[0].query_delivery_batches;

    assert_eq!(direct.len(), 1);
    assert_eq!(program_batches.len(), 1);
    assert_eq!(
        direct[0].delivery_cause_kind(),
        program_batches[0].delivery_cause_kind()
    );
    assert_eq!(
        direct[0].has_relational_patch(),
        program_batches[0].has_relational_patch()
    );
    assert_eq!(
        direct[0].patch_group_kind(),
        program_batches[0].patch_group_kind()
    );

    let missing_previous = runtime
        .emit_time_only_delivery(
            view.name(),
            QuerySubscriptionDeliveryCauseKind::PreviousValueTransition,
            "tick:missing-previous",
            false,
            true,
        )
        .expect_err("missing previous-value evidence should deny");
    let stale_basis = runtime
        .emit_time_only_delivery(
            view.name(),
            QuerySubscriptionDeliveryCauseKind::WindowExit,
            "tick:stale-basis",
            false,
            false,
        )
        .expect_err("stale temporal basis should deny");

    match missing_previous {
        WorthQueryRuntimeError::LiveSubscriptionInstallation { stage, message, .. } => {
            assert_eq!(stage, "time-only-delivery");
            assert!(message.contains("MissingPreviousValueEvidence"));
        }
        other => panic!("expected time-only delivery denial, got {other:?}"),
    }
    match stale_basis {
        WorthQueryRuntimeError::LiveSubscriptionInstallation { stage, message, .. } => {
            assert_eq!(stage, "time-only-delivery");
            assert!(message.contains("StaleTemporalBasis"));
        }
        other => panic!("expected stale temporal basis denial, got {other:?}"),
    }
}
