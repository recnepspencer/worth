use super::*;

pub(in crate::runtime::tests) struct FakeDsl;

pub(in crate::runtime::tests) struct FakeSchemaAdapter;
pub(in crate::runtime::tests) fn preview_safe_program() -> WorthQueryProgram {
    WorthQueryProgram::new(
        "fake.preview.safe",
        [WorthQueryOperation::new("create_task")
            .with_input(WorthQueryTypedPort::new(
                "title",
                WorthQueryPortType::String,
            ))
            .requires(WorthQueryAuthorityRequirement::Live)
            .requires(WorthQueryAuthorityRequirement::Writeback)
            .with_effect(WorthQueryProgramEffect::WriteTemplate(
                WorthQueryWriteCommandTemplate::InsertAspects {
                    collection: "Task".to_string(),
                    aspects: vec![
                        WorthQueryAdmittedAspectValueTemplate::new(
                            test_aspect_touch("identity.id"),
                            WorthQueryValueExpr::literal(WorthQueryProgramValue::string("")),
                        ),
                        WorthQueryAdmittedAspectValueTemplate::new(
                            test_aspect_touch("title.value"),
                            WorthQueryValueExpr::input("title"),
                        ),
                    ],
                },
            ))
            .with_effect(WorthQueryProgramEffect::ReadLive {
                view_name: "tasks.table".to_string(),
            })
            .with_effect(WorthQueryProgramEffect::DrainPatches {
                view_name: "tasks.table".to_string(),
            })],
    )
    .expect("preview-safe test program should build")
}

pub(in crate::runtime::tests) struct TitleListMaintainer;
pub(in crate::runtime::tests) struct SummaryMaintainer;
pub(in crate::runtime::tests) struct RefreshCountMaintainer;

fn test_delta_display_identity(delta: &crate::memory_workspace::WorthQueryMutationDelta) -> String {
    if let Some(upstream_view) = delta.collection().strip_prefix("derived:")
        && delta.entity_identity
            == crate::memory_workspace::admit_authored_entity_label(upstream_view)
    {
        return upstream_view.to_string();
    }
    delta
        .entity_identity
        .evidence_identity()
        .as_str()
        .to_string()
}

impl WorthQueryDerivedViewMaintainer for TitleListMaintainer {
    fn maintain(
        &mut self,
        view: &WorthQueryDerivedView,
        delta: &crate::memory_workspace::WorthQueryMutationDelta,
        materialization: &mut WorthQueryDerivedViewMaterialization,
    ) -> WorthQueryDerivedPatch {
        let retained_row = retained_string_test_row("value", test_delta_display_identity(delta));
        materialization.push_retained_row(retained_row.clone());
        WorthQueryDerivedPatch::incremental(
            view.name(),
            crate::memory_workspace::admit_external_commit_label("derived-test-commit"),
            delta.entity_identity.clone(),
            if view.produced_aspect_touches().is_empty() {
                delta.admitted_touched_aspects().to_vec()
            } else {
                view.produced_aspect_touches().to_vec()
            },
            WorthQueryDerivedPatchPayload::from_retained_row(retained_row),
        )
    }
}

impl WorthQueryDerivedViewMaintainer for SummaryMaintainer {
    fn maintain(
        &mut self,
        view: &WorthQueryDerivedView,
        delta: &crate::memory_workspace::WorthQueryMutationDelta,
        materialization: &mut WorthQueryDerivedViewMaterialization,
    ) -> WorthQueryDerivedPatch {
        let retained_row = retained_string_test_row(
            "value",
            format!("summary:{}", test_delta_display_identity(delta)),
        );
        materialization.replace_retained_rows([retained_row.clone()]);
        WorthQueryDerivedPatch::incremental(
            view.name(),
            crate::memory_workspace::admit_external_commit_label("derived-summary-commit"),
            delta.entity_identity.clone(),
            if view.produced_aspect_touches().is_empty() {
                delta.admitted_touched_aspects().to_vec()
            } else {
                view.produced_aspect_touches().to_vec()
            },
            WorthQueryDerivedPatchPayload::from_retained_row(retained_row),
        )
    }
}

impl WorthQueryDerivedViewMaintainer for RefreshCountMaintainer {
    fn maintain(
        &mut self,
        view: &WorthQueryDerivedView,
        delta: &crate::memory_workspace::WorthQueryMutationDelta,
        materialization: &mut WorthQueryDerivedViewMaterialization,
    ) -> WorthQueryDerivedPatch {
        let retained_row = retained_string_test_row(
            "value",
            format!("incremental:{}", test_delta_display_identity(delta)),
        );
        materialization.replace_retained_rows([retained_row.clone()]);
        WorthQueryDerivedPatch::incremental(
            view.name(),
            crate::memory_workspace::admit_external_commit_label("refresh-count-incremental"),
            delta.entity_identity.clone(),
            if view.produced_aspect_touches().is_empty() {
                delta.admitted_touched_aspects().to_vec()
            } else {
                view.produced_aspect_touches().to_vec()
            },
            WorthQueryDerivedPatchPayload::from_retained_row(retained_row),
        )
    }

    fn refresh_from_upstreams(
        &mut self,
        view: &WorthQueryDerivedView,
        _refresh: &WorthQueryRetainedRefreshContext,
        upstreams: &WorthQueryRetainedUpstreamInputs,
        materialization: &mut WorthQueryDerivedViewMaterialization,
    ) -> Option<WorthQueryDerivedPatch> {
        let count = upstreams
            .declared_live_row_sets(view)
            .map(<[crate::memory_workspace::WorthQueryEntity]>::len)
            .sum::<usize>();
        let retained_row = retained_string_test_row("value", format!("count:{count}"));
        materialization.replace_retained_rows([retained_row.clone()]);
        Some(WorthQueryDerivedPatch::whole_refresh_materialized(
            view.name(),
            crate::memory_workspace::admit_external_commit_label("refresh-count-rebuild"),
            if view.produced_aspect_touches().is_empty() {
                view.dependency_aspect_touches().to_vec()
            } else {
                view.produced_aspect_touches().to_vec()
            },
            WorthQueryDerivedPatchPayload::from_retained_row(retained_row),
            "retained-live-snapshot-rebuild",
        ))
    }
}

impl WorthQuerySchemaAdapter for FakeSchemaAdapter {
    fn schema_view(&self, operation_id: &str) -> Option<QuerySchemaView> {
        (operation_id == "create_task").then(task_schema)
    }
}

impl WorthQueryProgramSource for FakeDsl {
    fn compile_program<A>(
        self,
        schema_adapter: &A,
    ) -> Result<WorthQueryProgram, WorthQueryProgramError>
    where
        A: WorthQuerySchemaAdapter + ?Sized,
    {
        let schema_view = schema_adapter
            .schema_view("create_task")
            .ok_or_else(|| WorthQueryProgramError::new("missing schema for create_task"))?;
        WorthQueryProgram::new(
            "fake.strict.dsl",
            [WorthQueryOperation::new("create_task")
                .with_input(WorthQueryTypedPort::new(
                    "title",
                    WorthQueryPortType::String,
                ))
                .requires(WorthQueryAuthorityRequirement::Live)
                .requires(WorthQueryAuthorityRequirement::Writeback)
                .with_effect(WorthQueryProgramEffect::DeclareLiveView {
                    name: "tasks.table".to_string(),
                    request: task_live_request(),
                    schema_view,
                })
                .with_effect(WorthQueryProgramEffect::WriteTemplate(
                    WorthQueryWriteCommandTemplate::InsertAspects {
                        collection: "Task".to_string(),
                        aspects: vec![
                            WorthQueryAdmittedAspectValueTemplate::new(
                                test_aspect_touch("identity.id"),
                                WorthQueryValueExpr::literal(WorthQueryProgramValue::string("")),
                            ),
                            WorthQueryAdmittedAspectValueTemplate::new(
                                test_aspect_touch("title.value"),
                                WorthQueryValueExpr::input("title"),
                            ),
                        ],
                    },
                ))
                .with_effect(WorthQueryProgramEffect::ReadLive {
                    view_name: "tasks.table".to_string(),
                })
                .with_effect(WorthQueryProgramEffect::DrainPatches {
                    view_name: "tasks.table".to_string(),
                })],
        )
    }
}
