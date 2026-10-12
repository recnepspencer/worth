use super::*;

#[test]
fn refresh_fallback_maintainer_receives_retained_mutation_metadata() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    struct MetadataSnapshotMaintainer;

    impl WorthQueryDerivedViewMaintainer for MetadataSnapshotMaintainer {
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
                external_commit("metadata-snapshot-incremental"),
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
            refresh: &WorthQueryRetainedRefreshContext,
            _upstreams: &WorthQueryRetainedUpstreamInputs,
            materialization: &mut WorthQueryDerivedViewMaterialization,
        ) -> Option<WorthQueryDerivedPatch> {
            let author = refresh
                .refresh_metadata()
                .get(&test_mutation_metadata_key("author"))
                .map(|value| value.terminal_digest_text())
                .unwrap_or("missing");
            let row = retained_string_test_row(
                "value",
                format!(
                    "{}:{}:{}",
                    refresh
                        .refresh_identity()
                        .terminal_projection_for_reporting(),
                    terminal_touched_aspect_paths_projection(refresh.admitted_touched_aspects())
                        .join("|"),
                    author
                ),
            );
            let payload = WorthQueryDerivedPatchPayload::from_retained_row(row.clone());
            materialization.replace_retained_rows([row]);
            Some(WorthQueryDerivedPatch::whole_refresh_materialized(
                view.name(),
                external_commit("metadata-snapshot-refresh"),
                if view.produced_aspect_touches().is_empty() {
                    view.dependency_aspect_touches().to_vec()
                } else {
                    view.produced_aspect_touches().to_vec()
                },
                payload,
                "retained-mutation-metadata",
            ))
        }
    }

    fn terminal_touched_aspect_paths_projection(touches: &[WorthQueryAspectTouch]) -> Vec<String> {
        touches
            .iter()
            .map(|touch| touch.admitted_touch_digest_part().to_string())
            .collect()
    }

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("computed.refresh.metadata")
        .expect("task runtime should open a named workspace");
    let tasks: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.table", |q| {
            q.from("Task")
                .select([
                    crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                        .unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                ])
                .order_by(
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                )
                .schema_basis("tasks-metadata-table")
        })
        .expect("task live view should declare");
    let metadata: WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape> = workspace
        .computed_view(
            request_execution,
            WorthQueryDerivedView::new("computed.refresh.metadata", touches(["title"]))
                .depends_on_live(&tasks)
                .produces(touches(["summary.metadata"]))
                .whole_refresh_fallback(),
            MetadataSnapshotMaintainer,
        )
        .expect("metadata refresh maintainer should declare");

    let receipt = workspace
        .insert(request_execution, "Task", |builder| {
            builder.metadata("author", "worth-topo").set_aspect(
                touch("title.value"),
                test_authored_string_aspect_value("Metadata proof"),
            )
        })
        .expect("task insert should succeed");

    assert_eq!(
        retained_value_aspects(
            workspace
                .materialize_result(request_execution, &metadata)
                .expect("metadata materialization should execute")
                .retained_rows(),
        ),
        vec![test_string_aspect_value(format!(
            "{}:title:<whole-aspect>:worth-topo",
            receipt
                .commit_identity()
                .terminal_projection_for_reporting()
        ))]
    );
}
