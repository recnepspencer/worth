use super::*;

#[test]
fn derived_materialization_bundle_binds_one_exact_retained_artifact() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    struct TitleRowMaintainer {
        tasks: WorthQueryLiveView<WorthQueryUnrefinedLiveShape>,
    }
    struct CountRowMaintainer {
        title_row: WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape>,
    }

    impl WorthQueryDerivedViewMaintainer for TitleRowMaintainer {
        fn maintain(
            &mut self,
            view: &WorthQueryDerivedView,
            delta: &crate::memory_workspace::WorthQueryMutationDelta,
            materialization: &mut WorthQueryDerivedViewMaterialization,
        ) -> WorthQueryDerivedPatch {
            let row = retained_string_test_row(
                "value",
                delta
                    .entity_identity
                    .terminal_projection_for_reporting()
                    .to_string(),
            );
            let payload = WorthQueryDerivedPatchPayload::from_retained_row(row.clone());
            materialization.replace_retained_rows([row]);
            WorthQueryDerivedPatch::incremental(
                view.name(),
                external_commit("title-row"),
                delta.entity_identity.clone(),
                delta_or_produced_touches(view, delta),
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
            let row = retained_string_test_row(
                "value",
                upstreams
                    .declared_live_rows_for(view, &self.tasks)
                    .and_then(|rows| rows.first())
                    .map(|entity| entity.identity().terminal_projection_for_reporting())
                    .unwrap_or_else(|| "missing".to_string()),
            );
            let payload = WorthQueryDerivedPatchPayload::from_retained_row(row.clone());
            materialization.replace_retained_rows([row]);
            Some(WorthQueryDerivedPatch::whole_refresh_materialized(
                view.name(),
                external_commit("title-row-refresh"),
                dependency_or_produced_touches(view),
                payload,
                "title-row-refresh",
            ))
        }
    }

    impl WorthQueryDerivedViewMaintainer for CountRowMaintainer {
        fn maintain(
            &mut self,
            view: &WorthQueryDerivedView,
            _delta: &crate::memory_workspace::WorthQueryMutationDelta,
            materialization: &mut WorthQueryDerivedViewMaterialization,
        ) -> WorthQueryDerivedPatch {
            let row = retained_test_row([("count", AspectValue::UInt64(1))]);
            let payload = WorthQueryDerivedPatchPayload::from_retained_row(row.clone());
            materialization.replace_retained_rows([row]);
            WorthQueryDerivedPatch::whole_refresh_materialized(
                view.name(),
                external_commit("count-row"),
                dependency_or_produced_touches(view),
                payload,
                "count-refresh",
            )
        }

        fn refresh_from_upstreams(
            &mut self,
            view: &WorthQueryDerivedView,
            _refresh: &WorthQueryRetainedRefreshContext,
            upstreams: &WorthQueryRetainedUpstreamInputs,
            materialization: &mut WorthQueryDerivedViewMaterialization,
        ) -> Option<WorthQueryDerivedPatch> {
            let row = retained_test_row([(
                "count",
                AspectValue::UInt64(
                    upstreams
                        .declared_retained_computed_rows_for(view, &self.title_row)
                        .len() as u64,
                ),
            )]);
            let payload = WorthQueryDerivedPatchPayload::from_retained_row(row.clone());
            materialization.replace_retained_rows([row]);
            Some(WorthQueryDerivedPatch::whole_refresh_materialized(
                view.name(),
                external_commit("count-row-refresh"),
                dependency_or_produced_touches(view),
                payload,
                "count-refresh",
            ))
        }
    }

    let runtime = stateful_bridge_task_runtime(request_execution);
    let mut workspace = runtime.workspace("computed-bundle-binding").unwrap();
    let tasks: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "computed.bundle.binding.tasks", |q| {
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
                .schema_basis("computed-bundle-binding")
        })
        .expect("task live view should declare");
    let title_row: WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape> = workspace
        .computed_view(
            request_execution,
            WorthQueryDerivedView::new("computed.bundle.binding.title_row", touches(["title"]))
                .depends_on_live(&tasks)
                .produces(touches(["title.row"]))
                .whole_refresh_fallback(),
            TitleRowMaintainer {
                tasks: tasks.clone(),
            },
        )
        .expect("title-row computed should declare");
    let count_row: WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape> = workspace
        .computed_view(
            request_execution,
            WorthQueryDerivedView::new("computed.bundle.binding.count_row", touches(["title.row"]))
                .depends_on_derived(&title_row)
                .produces(touches(["count.row"]))
                .whole_refresh_fallback(),
            CountRowMaintainer {
                title_row: title_row.clone(),
            },
        )
        .expect("count-row computed should declare");

    workspace
        .insert(request_execution, "Task", |builder| {
            builder.set_aspect(
                touch("title.value"),
                test_authored_string_aspect_value("Artifact proof"),
            )
        })
        .expect("task insert should succeed");

    let binding = workspace
        .materialize_derived_artifact_binding(
            request_execution,
            "computed.bundle.binding.artifact",
            [(&title_row).into(), (&count_row).into()],
        )
        .expect("exact retained artifact binding should succeed");

    assert_eq!(binding.artifact_name(), "computed.bundle.binding.artifact");
    assert_eq!(binding.target_count(), 2);
    assert_eq!(
        binding
            .terminal_target_view_names_projection()
            .collect::<Vec<_>>(),
        vec![count_row.name(), title_row.name()]
    );
    assert_eq!(
        binding.snapshot_identity(),
        binding
            .materialization(&title_row)
            .expect("title-row result should stay in artifact")
            .receipt()
            .snapshot_identity()
    );

    let title = binding
        .materialization(&title_row)
        .expect("title row should stay in retained artifact binding")
        .single_retained_row()
        .expect("title materialization should retain one row");
    let count = binding
        .materialization(&count_row)
        .expect("count row should stay in retained artifact binding")
        .single_retained_row()
        .expect("count materialization should retain one row");
    assert!(!binding.binding_for_reporting().is_empty());
    assert!(!retained_string_field(title, "value").is_empty());
    assert_eq!(retained_u64_field(count, "count"), 1);

    let mismatch = workspace
        .materialize_derived_artifact_bundle(request_execution, [(&title_row).into()])
        .expect("bundle should materialize retained title row")
        .bind_retained_artifact(
            "computed.bundle.binding.mismatch",
            [(&title_row).into(), (&count_row).into()],
        )
        .expect_err("missing retained target should fail closed");
    match mismatch {
        WorthQueryRuntimeError::RetainedRowDecode {
            view_name, stage, ..
        } => {
            assert_eq!(view_name, "computed.bundle.binding.mismatch");
            assert_eq!(stage, "derived-artifact-binding");
        }
        other => panic!("expected retained-row decode error, got {other:?}"),
    }
}
