use super::*;

#[test]
fn derived_view_receives_narrow_or_fallback_patch_notes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.table",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    let task_titles = runtime
        .declare_derived_view(
            WorthQueryDerivedView::new("task_titles", touches(["title"])).whole_refresh_fallback(),
        )
        .expect("derived view should declare");
    let task_titles_handle =
        WorthQueryDerivedViewHandle::<WorthQueryUnrefinedLiveShape>::new(task_titles.name());
    let insert = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    ("title.value", test_string_aspect_value("Derived task")),
                ],
            ),
        )
        .expect("insert should route to derived view");
    let update = runtime
        .write(
            request_execution,
            update_string_aspect(
                insert.deltas()[0].entity_identity.clone(),
                "title.value",
                "Derived task renamed",
            ),
        )
        .expect("title update should route to derived view");

    let patches = runtime.drain_derived_patches(&task_titles_handle);

    assert_eq!(
        update.terminal_affected_derived_view_ids_projection(),
        &["task_titles".to_string()]
    );
    assert!(update.refresh_fallback());
    assert!(patches
        .derived_patch_notes
        .iter()
        .any(|note| note.starts_with("whole-refresh-fallback")));
}

#[test]
fn maintained_derived_view_materializes_incremental_patches() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.table",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    let titles = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("task_titles", touches(["title"])),
            TitleListMaintainer,
        )
        .expect("maintained derived view should declare");

    let insert = runtime
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
        .expect("insert should route derived patch");
    let patches = runtime.drain_derived_patches(&titles);

    assert_eq!(
        insert.terminal_affected_derived_view_ids_projection(),
        &["task_titles".to_string()]
    );
    let expected_row = test_string_aspect_value(
        insert.deltas()[0]
            .entity_identity
            .terminal_projection_for_reporting()
            .to_string(),
    );
    assert_eq!(
        read_derived_value_aspects(request_execution, &runtime, &titles),
        vec![expected_row.clone()]
    );
    assert_eq!(patches.derived_patches.len(), 1);
    assert_eq!(
        retained_value_aspects(patches.derived_patches[0].retained_payload_rows()),
        vec![expected_row]
    );

    runtime
        .write(
            request_execution,
            update_string_aspect(
                insert.deltas()[0].entity_identity.clone(),
                "identity.id",
                "ignored",
            ),
        )
        .expect("irrelevant update should not route derived patch");
    let irrelevant = runtime.drain_derived_patches(&titles);

    assert!(irrelevant.derived_patches.is_empty());
}

#[test]
fn nested_computed_views_route_in_deterministic_dependency_order() {
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
    let titles = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("computed.titles", touches(["title"]))
                .depends_on_live(&live)
                .produces(touches(["title.summary"])),
            TitleListMaintainer,
        )
        .expect("source computed view should declare");
    let summary = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("computed.summary", touches(["title.summary"]))
                .depends_on_derived(&titles)
                .produces(touches(["validation.state"])),
            SummaryMaintainer,
        )
        .expect("nested computed view should declare");

    let insert = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    ("title.value", test_string_aspect_value("Nested title")),
                ],
            ),
        )
        .expect("insert should update nested computeds");
    let title_patches = runtime.drain_derived_patches(&titles);
    let summary_patches = runtime.drain_derived_patches(&summary);

    assert_eq!(
        insert.terminal_affected_derived_view_ids_projection(),
        &[
            "computed.summary".to_string(),
            "computed.titles".to_string()
        ]
    );
    assert_eq!(insert.considered_computed_view_count(), 2);
    assert_eq!(title_patches.derived_patches.len(), 1);
    assert_eq!(
        title_patches.derived_patches[0].aspect_touches(),
        test_aspect_touches(["title.summary"]).as_slice()
    );
    assert_eq!(summary_patches.derived_patches.len(), 1);
    assert_eq!(
        summary_patches.derived_patches[0].aspect_touches(),
        test_aspect_touches(["validation.state"]).as_slice()
    );
    assert_eq!(
        read_derived_value_aspects(request_execution, &runtime, &summary),
        vec![test_string_aspect_value(format!(
            "summary:{}",
            insert.deltas()[0]
                .entity_identity
                .terminal_projection_for_reporting()
        ))]
    );

    runtime
        .write(
            request_execution,
            update_string_aspect(
                insert.deltas()[0].entity_identity.clone(),
                "identity.id",
                "ignored",
            ),
        )
        .expect("irrelevant update should still write");
    assert!(runtime
        .drain_derived_patches(&titles)
        .derived_patches
        .is_empty());
    assert!(runtime
        .drain_derived_patches(&summary)
        .derived_patches
        .is_empty());
}

#[test]
fn computed_dependency_index_replaces_redeclared_view_membership() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_issue_runtime(request_execution);
    let task_live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.table",
            task_live_request(),
            task_schema(),
        )
        .expect("task live should declare");
    let issue_live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "issues.table",
            issue_live_request(),
            issue_schema(),
        )
        .expect("issue live should declare");
    let computed = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("computed.shared", touches(["title"]))
                .depends_on_live(&task_live)
                .produces(touches(["title.summary"])),
            TitleListMaintainer,
        )
        .expect("task-backed computed should declare");

    runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("computed.shared", touches(["summary"]))
                .depends_on_live(&issue_live)
                .produces(touches(["issue.summary"])),
            SummaryMaintainer,
        )
        .expect("redeclared computed should replace old dependency index membership");

    let task_write = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    (
                        "title.value",
                        test_string_aspect_value("Task should not wake redeclared computed"),
                    ),
                ],
            ),
        )
        .expect("task write should execute");
    assert!(task_write
        .terminal_affected_derived_view_ids_projection()
        .is_empty());
    assert_eq!(task_write.considered_computed_view_count(), 0);
    assert!(runtime
        .drain_derived_patches(&computed)
        .derived_patches
        .is_empty());

    let issue_write = runtime
        .write(
            request_execution,
            insert_command(
                "Issue",
                [
                    ("identity.id", test_string_aspect_value("")),
                    (
                        "summary.value",
                        test_string_aspect_value("Issue wakes computed"),
                    ),
                ],
            ),
        )
        .expect("issue write should execute");
    let issue_patches = runtime.drain_derived_patches(&computed);

    assert_eq!(
        issue_write.terminal_affected_derived_view_ids_projection(),
        &["computed.shared".to_string()]
    );
    assert_eq!(issue_write.considered_computed_view_count(), 1);
    assert_eq!(issue_patches.derived_patches.len(), 1);
    assert_eq!(
        issue_patches.derived_patches[0].aspect_touches(),
        test_aspect_touches(["issue.summary"]).as_slice()
    );
}
