use super::*;

#[test]
fn computed_dependency_admission_rejects_missing_or_cyclic_upstream_views() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let missing_live = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("computed.missing-live", touches(["title"]))
                .depends_on_live_name_from_workspace_declaration("tasks.not-declared"),
            TitleListMaintainer,
        )
        .expect_err("missing live dependency should reject before registration");
    match missing_live {
        WorthQueryRuntimeError::ComputedDeclaration { message, .. } => {
            assert!(message.contains("tasks.not-declared"));
        }
        other => panic!("expected computed declaration error, got {other:?}"),
    }

    let missing = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("computed.missing", touches(["title.summary"]))
                .depends_on_derived_name_from_workspace_declaration("computed.unknown"),
            SummaryMaintainer,
        )
        .expect_err("missing computed dependency should reject before registration");
    match missing {
        WorthQueryRuntimeError::ComputedDeclaration { message, .. } => {
            assert!(message.contains("computed.unknown"));
        }
        other => panic!("expected computed declaration error, got {other:?}"),
    }

    let first = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("computed.first", touches(["title"]))
                .produces(touches(["title.summary"])),
            TitleListMaintainer,
        )
        .expect("first computed should declare");
    let second = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("computed.second", touches(["title.summary"]))
                .depends_on_derived(&first)
                .produces(touches(["validation.state"])),
            SummaryMaintainer,
        )
        .expect("second computed should declare");

    let cycle = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("computed.first", touches(["validation.state"]))
                .depends_on_derived(&second),
            SummaryMaintainer,
        )
        .expect_err("redeclared computed dependency should not create a cycle");
    match cycle {
        WorthQueryRuntimeError::ComputedDeclaration { message, .. } => {
            assert!(message.contains("cycle"));
        }
        other => panic!("expected computed cycle declaration error, got {other:?}"),
    }
}
