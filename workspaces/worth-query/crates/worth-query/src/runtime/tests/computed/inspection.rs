use super::*;

#[test]
fn computed_handle_inspection_reports_dependencies_aspects_and_materialization() {
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
        .expect("live should declare");
    let computed = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("computed.inspectable", touches(["title"]))
                .depends_on_live(&live)
                .produces(touches(["title.summary"])),
            TitleListMaintainer,
        )
        .expect("computed should declare");
    runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    ("title.value", test_string_aspect_value("Inspectable task")),
                ],
            ),
        )
        .expect("write should materialize computed output");

    let evidence = runtime
        .inspect_derived_view(request_execution, &computed)
        .expect("computed handle should inspect");

    assert_eq!(evidence.name(), "computed.inspectable");
    assert_eq!(
        evidence.authority_lane(),
        WorthQueryAuthorityLane::DerivedRuntimeState
    );
    assert_eq!(evidence.upstream_live_views(), &["tasks.table".to_string()]);
    assert!(evidence.upstream_derived_views().is_empty());
    assert_eq!(
        evidence.dependency_aspect_touches(),
        test_aspect_touches(["title"]).as_slice()
    );
    assert_eq!(
        evidence.produced_aspect_touches(),
        test_aspect_touches(["title.summary"]).as_slice()
    );
    assert!(evidence.incremental_delivery());
    assert_eq!(evidence.materialized_row_count(), 1);
    assert_eq!(evidence.pending_patch_count(), 1);
    assert_eq!(evidence.pending_incremental_patch_count(), 1);
    assert_eq!(evidence.pending_refresh_fallback_count(), 0);
    assert!(!evidence.declaration_digest().is_empty());
    assert!(!evidence.dependency_digest().is_empty());
    assert!(!evidence.produced_aspect_digest().is_empty());
    assert!(!evidence.materialization_digest().is_empty());
    assert!(!evidence.pending_patch_digest().is_empty());
    assert!(!evidence.inspection_digest().is_empty());

    let foreign_runtime = stateful_bridge_task_runtime(request_execution);
    let error = foreign_runtime
        .inspect_derived_view(request_execution, &computed)
        .expect_err("foreign computed handle should not inspect in another runtime");
    assert!(matches!(
        error,
        WorthQueryRuntimeError::MissingDerivedView(_)
    ));
}

#[test]
fn nested_computed_inspection_explains_dependency_and_patch_posture() {
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
            WorthQueryDerivedView::new("computed.inspect.titles", touches(["title"]))
                .depends_on_live(&live)
                .produces(touches(["title.summary"])),
            TitleListMaintainer,
        )
        .expect("source computed should declare");
    let summary = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("computed.inspect.summary", touches(["title.summary"]))
                .depends_on_derived(&titles)
                .produces(touches(["validation.state"])),
            SummaryMaintainer,
        )
        .expect("nested computed should declare");

    let before = runtime
        .inspect_derived_view(request_execution, &summary)
        .expect("nested computed should inspect before materialization");
    assert_eq!(
        before.upstream_derived_views(),
        &["computed.inspect.titles".to_string()]
    );
    assert!(before.upstream_live_views().is_empty());
    assert_eq!(
        before.dependency_aspect_touches(),
        test_aspect_touches(["title.summary"]).as_slice()
    );
    assert_eq!(
        before.produced_aspect_touches(),
        test_aspect_touches(["validation.state"]).as_slice()
    );
    assert_eq!(before.materialized_row_count(), 0);
    assert_eq!(before.pending_patch_count(), 0);
    assert!(before.incremental_delivery());

    let insert = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    (
                        "title.value",
                        test_string_aspect_value("Nested inspectable"),
                    ),
                ],
            ),
        )
        .expect("insert should update nested computeds");
    let after = runtime
        .inspect_derived_view(request_execution, &summary)
        .expect("nested computed should inspect after materialization");

    assert_eq!(insert.considered_computed_view_count(), 2);
    assert_eq!(after.materialized_row_count(), 1);
    assert_eq!(after.pending_patch_count(), 1);
    assert_eq!(after.pending_incremental_patch_count(), 1);
    assert_eq!(after.pending_refresh_fallback_count(), 0);
    assert_ne!(
        after.materialization_digest(),
        before.materialization_digest()
    );
    assert_ne!(after.pending_patch_digest(), before.pending_patch_digest());
    assert_ne!(after.inspection_digest(), before.inspection_digest());
    assert_eq!(after.dependency_digest(), before.dependency_digest());
    assert_eq!(
        after.produced_aspect_digest(),
        before.produced_aspect_digest()
    );
}

#[test]
fn refresh_fallback_computed_inspection_reports_fallback_posture() {
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
    let computed_definition = runtime
        .declare_derived_view(
            WorthQueryDerivedView::new("computed.inspect.refresh", touches(["title"]))
                .whole_refresh_fallback(),
        )
        .expect("refresh fallback computed should declare");
    let computed = WorthQueryDerivedViewHandle::<WorthQueryUnrefinedLiveShape>::new(
        computed_definition.name(),
    );

    let before = runtime
        .inspect_derived_view(request_execution, &computed)
        .expect("refresh fallback computed should inspect before write");
    assert!(!before.incremental_delivery());

    runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    (
                        "title.value",
                        test_string_aspect_value("Refresh inspectable"),
                    ),
                ],
            ),
        )
        .expect("insert should route fallback computed patch");
    let after = runtime
        .inspect_derived_view(request_execution, &computed)
        .expect("refresh fallback computed should inspect after write");

    assert!(!after.incremental_delivery());
    assert_eq!(after.pending_patch_count(), 1);
    assert_eq!(after.pending_incremental_patch_count(), 0);
    assert_eq!(after.pending_refresh_fallback_count(), 1);
    assert_eq!(
        after.materialization_digest(),
        before.materialization_digest(),
        "fallback inspection must not pretend a full materialization happened"
    );
    assert_ne!(after.pending_patch_digest(), before.pending_patch_digest());
}
