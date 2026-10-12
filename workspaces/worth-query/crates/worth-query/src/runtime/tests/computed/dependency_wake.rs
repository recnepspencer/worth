use super::*;

#[test]
fn refresh_rebuilt_computed_wakes_downstream_dependencies_through_produced_aspects() {
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
    let refresh = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("computed.refresh.count", touches(["title"]))
                .depends_on_live(&live)
                .produces(touches(["summary.count"]))
                .whole_refresh_fallback(),
            RefreshCountMaintainer,
        )
        .expect("refresh maintainer computed should declare");
    let downstream = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("computed.refresh.summary", touches(["summary.count"]))
                .depends_on_derived(&refresh)
                .produces(touches(["validation.state"])),
            SummaryMaintainer,
        )
        .expect("downstream computed should declare");

    let insert = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    ("title.value", test_string_aspect_value("Nested refresh")),
                ],
            ),
        )
        .expect("insert should wake refresh and downstream computeds");
    let refresh_patches = runtime.drain_derived_patches(&refresh);
    let downstream_patches = runtime.drain_derived_patches(&downstream);

    assert_eq!(
        insert.terminal_affected_derived_view_ids_projection(),
        &[
            "computed.refresh.count".to_string(),
            "computed.refresh.summary".to_string(),
        ]
    );
    assert_eq!(insert.considered_computed_view_count(), 2);
    assert_eq!(refresh_patches.derived_patches.len(), 1);
    assert!(refresh_patches.derived_patches[0].is_refresh_fallback());
    assert_eq!(
        downstream_patches.derived_patches[0].aspect_touches(),
        test_aspect_touches(["validation.state"]).as_slice()
    );
    assert_eq!(
        read_derived_value_aspects(request_execution, &runtime, &downstream),
        vec![test_string_aspect_value(
            "summary:computed.refresh.count".to_string()
        )]
    );
}

#[test]
fn downstream_refresh_fallback_receives_declared_live_siblings_through_computed_dependency_chain() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    struct MixedUpstreamSnapshotMaintainer;

    impl WorthQueryDerivedViewMaintainer for MixedUpstreamSnapshotMaintainer {
        fn maintain(
            &mut self,
            view: &WorthQueryDerivedView,
            delta: &crate::memory_workspace::WorthQueryMutationDelta,
            materialization: &mut WorthQueryDerivedViewMaterialization,
        ) -> WorthQueryDerivedPatch {
            let row = retained_string_test_row(
                "value",
                format!(
                    "incremental:{}",
                    delta.entity_identity.terminal_projection_for_reporting()
                ),
            );
            let payload = WorthQueryDerivedPatchPayload::from_retained_row(row.clone());
            materialization.replace_retained_rows([row]);
            WorthQueryDerivedPatch::incremental(
                view.name(),
                external_commit("mixed-upstream-incremental"),
                delta.entity_identity.clone(),
                if view.produced_aspect_touches().is_empty() {
                    delta.admitted_touched_aspects().to_vec()
                } else {
                    view.produced_aspect_touches().to_vec()
                },
                payload,
            )
        }

        fn refresh_from_upstreams(
            &mut self,
            view: &WorthQueryDerivedView,
            _refresh: &WorthQueryRetainedRefreshContext,
            upstreams: &WorthQueryRetainedUpstreamInputs,
            materialization: &mut WorthQueryDerivedViewMaterialization,
        ) -> Option<WorthQueryDerivedPatch> {
            let derived_count = upstreams
                .declared_retained_computed_row_sets(view)
                .map(<[WorthQueryRetainedMaterializedRow]>::len)
                .sum::<usize>();
            let live_count = upstreams
                .declared_live_row_sets(view)
                .map(<[crate::memory_workspace::WorthQueryEntity]>::len)
                .sum::<usize>();
            let row = retained_string_test_row(
                "value",
                format!("derived:{derived_count}|live:{live_count}"),
            );
            let payload = WorthQueryDerivedPatchPayload::from_retained_row(row.clone());
            materialization.replace_retained_rows([row]);
            Some(WorthQueryDerivedPatch::whole_refresh_materialized(
                view.name(),
                external_commit("mixed-upstream-refresh"),
                if view.produced_aspect_touches().is_empty() {
                    view.dependency_aspect_touches().to_vec()
                } else {
                    view.produced_aspect_touches().to_vec()
                },
                payload,
                "retained-mixed-upstream-snapshot",
            ))
        }
    }

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
    let refresh = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("computed.refresh.count", touches(["title"]))
                .depends_on_live(&tasks)
                .produces(touches(["summary.count"]))
                .whole_refresh_fallback(),
            RefreshCountMaintainer,
        )
        .expect("upstream refresh maintainer should declare");
    let downstream = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new(
                "computed.refresh.mixed",
                touches(["summary.count", "summary"]),
            )
            .depends_on_derived(&refresh)
            .depends_on_live(&issues)
            .produces(touches(["validation.state"]))
            .whole_refresh_fallback(),
            MixedUpstreamSnapshotMaintainer,
        )
        .expect("downstream mixed refresh maintainer should declare");

    runtime
        .write(
            request_execution,
            insert_command(
                "Issue",
                [
                    ("identity.id", test_string_aspect_value("")),
                    (
                        "summary.value",
                        test_string_aspect_value("Issue sibling stays retained"),
                    ),
                ],
            ),
        )
        .expect("issue insert should seed sibling live state");
    runtime.drain_derived_patches(&refresh);
    runtime.drain_derived_patches(&downstream);

    runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    (
                        "title.value",
                        test_string_aspect_value("Task wakes upstream refresh"),
                    ),
                ],
            ),
        )
        .expect("task insert should rebuild downstream from derived and live siblings");
    let downstream_rows = read_derived_value_aspects(request_execution, &runtime, &downstream);

    assert_eq!(
        downstream_rows,
        vec![test_string_aspect_value("derived:1|live:1".to_string())]
    );
}
