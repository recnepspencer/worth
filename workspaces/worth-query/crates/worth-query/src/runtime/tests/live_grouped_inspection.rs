use super::support::*;

#[test]
fn grouped_live_view_inspection_preserves_grouped_family_and_baseline_support() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_grouped_task_runtime(request_execution);
    let table: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.seed-table",
            grouped_task_table_live_request(),
            grouped_task_schema(),
        )
        .expect("table live view should declare before grouped view");
    let _ = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    ("title.value", test_string_aspect_value("Seed task")),
                    ("status.value", test_string_aspect_value("todo")),
                ],
            ),
        )
        .expect("seed insert should write through table declaration");
    let _ = runtime.drain_patches(&table);
    let grouped: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.grouped",
            grouped_task_live_request(),
            grouped_task_schema(),
        )
        .expect("grouped live view should declare with backend-owned baseline");

    let inspection = runtime
        .inspect_live_view_explanation(&grouped)
        .expect("grouped live view explanation should inspect retained installation");

    assert_eq!(
        inspection.subscription_family(),
        "grouped_collection_membership"
    );
    assert_eq!(
        inspection.subscription_family_projection().label().as_str(),
        grouped
            .subscription_installation()
            .subscription_family_projection()
            .label()
            .as_str()
    );
    assert_eq!(
        inspection.support_projection().label().as_str(),
        grouped
            .subscription_installation()
            .support_projection()
            .label()
            .as_str()
    );
    assert!(!inspection.support_projection().label().as_str().is_empty());
    assert_eq!(inspection.counters().family_selection_count(), 1);
    assert_eq!(inspection.counters().declaration_count(), 1);
    assert_eq!(inspection.counters().bridge_lowering_count(), 1);
    assert_eq!(inspection.counters().admission_count(), 1);
    assert_eq!(inspection.counters().active_lane_creation_count(), 1);
    assert_eq!(inspection.counters().consumer_attachment_count(), 1);
    assert!(!inspection
        .inspection_projection()
        .label()
        .as_str()
        .is_empty());
}
