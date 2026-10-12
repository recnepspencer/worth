use super::*;

#[test]
fn retained_upstreams_decode_single_computed_rows_through_query_runtime_floor() {
    let upstreams = WorthQueryRetainedUpstreamInputs::from_retained_computed_rows(
        Vec::<(WorthQueryLiveArtifactTarget, Vec<WorthQueryEntity>)>::new(),
        [(
            WorthQueryDerivedMaterializationTarget::test_only("computed.materialized"),
            vec![retained_test_row([("count", AspectValue::UInt64(4))])],
        )],
    );

    let materialized =
        WorthQueryDerivedViewHandle::<WorthQueryUnrefinedLiveShape>::new("computed.materialized");
    let row = upstreams
        .single_retained_computed_row_for(&materialized)
        .expect("single retained computed row should be available");
    assert_eq!(retained_u64_field(row, "count"), 4);

    let missing_handle =
        WorthQueryDerivedViewHandle::<WorthQueryUnrefinedLiveShape>::new("computed.missing");
    let missing = upstreams
        .single_retained_computed_row_for(&missing_handle)
        .expect_err("missing retained row should fail closed");
    match missing {
        WorthQueryRuntimeError::RetainedRowDecode {
            view_name, stage, ..
        } => {
            assert_eq!(view_name, "computed.missing");
            assert_eq!(stage, "retained-upstream");
        }
        other => panic!("expected retained-row decode error, got {other:?}"),
    }

    let declaration = WorthQueryDerivedView::new("computed.consumer", touches(["count"]))
        .depends_on_derived_name_from_workspace_declaration("computed.materialized");
    let declared_row = upstreams
        .single_declared_retained_computed_row_for(&declaration, &materialized)
        .expect("declared retained upstream row should be available");
    assert_eq!(retained_u64_field(declared_row, "count"), 4);

    let undeclared_handle =
        WorthQueryDerivedViewHandle::<WorthQueryUnrefinedLiveShape>::new("computed.other");
    let undeclared = upstreams
        .single_declared_retained_computed_row_for(&declaration, &undeclared_handle)
        .expect_err("undeclared retained upstream row should fail closed");
    match undeclared {
        WorthQueryRuntimeError::RetainedRowDecode {
            view_name, stage, ..
        } => {
            assert_eq!(view_name, "computed.other");
            assert_eq!(stage, "retained-upstream");
        }
        other => panic!("expected retained-row decode error, got {other:?}"),
    }
}

#[test]
fn derived_materialization_bundle_decodes_multiple_retained_rows_through_query_runtime_floor() {
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

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("computed.materialization.bundle")
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
                .schema_basis("tasks-bundle-table")
        })
        .expect("task live view should declare");
    let title_row: WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape> = workspace
        .computed_view(
            request_execution,
            WorthQueryDerivedView::new("computed.bundle.title_row", touches(["title"]))
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
            WorthQueryDerivedView::new("computed.bundle.count_row", touches(["title.row"]))
                .depends_on_derived(&title_row)
                .produces(touches(["count.row"]))
                .whole_refresh_fallback(),
            CountRowMaintainer {
                title_row: title_row.clone(),
            },
        )
        .expect("count-row computed should declare");

    let receipt = workspace
        .insert(request_execution, "Task", |builder| {
            builder.set_aspect(
                touch("title.value"),
                test_authored_string_aspect_value("Bundle proof"),
            )
        })
        .expect("task insert should succeed");
    let bundle = workspace
        .materialize_derived_artifact_bundle(
            request_execution,
            [
                (&title_row).into(),
                (&count_row).into(),
                (&title_row).into(),
            ],
        )
        .expect("bundle should materialize retained computed rows");

    assert_eq!(bundle.target_count(), 2);
    assert!(bundle.includes_target(
        &crate::runtime::surface::WorthQueryDerivedMaterializationTarget::from(&title_row)
    ));
    assert!(bundle.includes_target(
        &crate::runtime::surface::WorthQueryDerivedMaterializationTarget::from(&count_row)
    ));
    assert_eq!(
        bundle.snapshot_identity(),
        bundle
            .materialization(&title_row)
            .expect("title-row result should stay in bundle")
            .receipt()
            .snapshot_identity()
    );
    let materialized_title = bundle
        .materialization(&title_row)
        .expect("title row should stay in bundle")
        .single_retained_row()
        .expect("title materialization should retain one row");
    let materialized_count = bundle
        .materialization(&count_row)
        .expect("count row should stay in bundle")
        .single_retained_row()
        .expect("count materialization should retain one row");
    assert_eq!(
        retained_string_field(materialized_title, "value"),
        receipt.deltas()[0]
            .entity_identity
            .terminal_projection_for_reporting()
            .to_string()
    );
    assert_eq!(retained_u64_field(materialized_count, "count"), 1);

    let missing = bundle
        .materialization_for_target(&WorthQueryDerivedMaterializationTarget::test_only(
            "computed.bundle.missing",
        ))
        .expect_err("missing retained bundle target should fail closed");
    match missing {
        WorthQueryRuntimeError::RetainedRowDecode {
            view_name, stage, ..
        } => {
            assert_eq!(view_name, "computed.bundle.missing");
            assert_eq!(stage, "derived-materialization-bundle");
        }
        other => panic!("expected retained-row decode error, got {other:?}"),
    }
}
