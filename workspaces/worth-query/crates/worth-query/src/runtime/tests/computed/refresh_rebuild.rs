use super::*;

#[test]
fn refresh_fallback_maintainer_rebuilds_from_retained_live_rows() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.table",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    let computed = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("computed.refresh.count", touches(["title"]))
                .depends_on_live(&live)
                .produces(touches(["summary.count"]))
                .whole_refresh_fallback(),
            RefreshCountMaintainer,
        )
        .expect("refresh maintainer computed should declare");

    let first = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    ("title.value", test_string_aspect_value("First title")),
                ],
            ),
        )
        .expect("first insert should route retained refresh rebuild");
    let first_patches = runtime.drain_derived_patches(&computed);

    assert_eq!(
        first.terminal_affected_derived_view_ids_projection(),
        &["computed.refresh.count".to_string()]
    );
    assert!(first.refresh_fallback());
    assert_eq!(
        read_derived_value_aspects(request_execution, &runtime, &computed),
        vec![test_string_aspect_value("count:1".to_string())]
    );
    assert_eq!(first_patches.derived_patches.len(), 1);
    assert!(first_patches.derived_patches[0].is_refresh_fallback());
    assert_eq!(
        first_patches.derived_patches[0].aspect_touches(),
        test_aspect_touches(["summary.count"]).as_slice()
    );
    assert_eq!(
        retained_value_aspects(first_patches.derived_patches[0].retained_payload_rows()),
        vec![test_string_aspect_value("count:1".to_string())]
    );

    runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    ("title.value", test_string_aspect_value("Second title")),
                ],
            ),
        )
        .expect("second insert should rebuild from retained live rows");

    assert_eq!(
        read_derived_value_aspects(request_execution, &runtime, &computed),
        vec![test_string_aspect_value("count:2".to_string())]
    );
}

#[test]
fn refresh_fallback_maintainer_seeds_retained_live_rows_during_declaration() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.table",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");

    runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    (
                        "title.value",
                        test_string_aspect_value("Seeded before computed declaration"),
                    ),
                ],
            ),
        )
        .expect("task insert should retain upstream live row before computed declaration");

    let computed = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("computed.refresh.seeded", touches(["title"]))
                .depends_on_live(&live)
                .produces(touches(["summary.count"]))
                .whole_refresh_fallback(),
            RefreshCountMaintainer,
        )
        .expect("refresh maintainer should seed from retained live rows during declaration");

    assert_eq!(
        read_derived_value_aspects(request_execution, &runtime, &computed),
        vec![test_string_aspect_value("count:1".to_string())]
    );
    assert_eq!(
        runtime
            .inspect_derived_view(request_execution, &computed)
            .expect("seeded refresh computed should inspect")
            .materialized_row_count(),
        1
    );
    assert!(
        runtime
            .drain_derived_patches(&computed)
            .derived_patches
            .is_empty(),
        "declaration-time retained seeding should not enqueue derived patches",
    );
}

#[test]
fn refresh_fallback_maintainer_receives_all_declared_upstream_live_rows() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_issue_runtime(request_execution);
    let tasks = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.table",
            task_live_request(),
            task_schema(),
        )
        .expect("task live view should declare");
    let issues = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "issues.table",
            issue_live_request(),
            issue_schema(),
        )
        .expect("issue live view should declare");
    let computed = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("computed.refresh.multi", touches(["title"]))
                .depends_on_live(&tasks)
                .depends_on_live(&issues)
                .produces(touches(["summary.count"]))
                .whole_refresh_fallback(),
            RefreshCountMaintainer,
        )
        .expect("multi-upstream refresh maintainer should declare");

    runtime
        .write(
            request_execution,
            insert_command(
                "Issue",
                [
                    ("identity.id", test_string_aspect_value("")),
                    (
                        "summary.value",
                        test_string_aspect_value("Sibling upstream row"),
                    ),
                ],
            ),
        )
        .expect("issue insert should seed sibling upstream");
    runtime.drain_derived_patches(&computed);

    let insert = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    (
                        "title.value",
                        test_string_aspect_value("Task wakes refresh computed"),
                    ),
                ],
            ),
        )
        .expect("task insert should rebuild from both retained upstreams");
    let patches = runtime.drain_derived_patches(&computed);

    assert_eq!(
        insert.terminal_affected_derived_view_ids_projection(),
        &["computed.refresh.multi".to_string()]
    );
    assert!(insert.refresh_fallback());
    assert_eq!(
        read_derived_value_aspects(request_execution, &runtime, &computed),
        vec![test_string_aspect_value("count:2".to_string())]
    );
    assert_eq!(patches.derived_patches.len(), 1);
    assert!(patches.derived_patches[0].is_refresh_fallback());
    assert_eq!(
        retained_value_aspects(patches.derived_patches[0].retained_payload_rows()),
        vec![test_string_aspect_value("count:2".to_string())]
    );
}
