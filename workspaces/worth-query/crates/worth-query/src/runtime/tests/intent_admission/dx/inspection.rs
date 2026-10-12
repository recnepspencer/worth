use super::*;

fn derived_summary_view(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
    view_name: &str,
) -> WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape> {
    let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.table", |q| {
            q.from("Task")
                .select([identity_id_field_key(), title_value_field_key()])
                .order_by(title_value_field_key())
                .schema_basis("intent-admission-derived-inspection")
        })
        .expect("live view should declare");
    workspace
        .computed_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new(view_name, test_aspect_touches(["title"]))
                .depends_on_live(&live)
                .produces(test_aspect_touches(["title.summary"])),
            TitleListMaintainer,
        )
        .expect("derived view should declare")
}

#[test]
fn materialize_intent_common_path_helper_executes_through_canonical_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = read_runtime(request_execution);
    let mut workspace = WorthQueryWorkspace::new("derived-materialize-common", runtime)
        .expect("workspace should build");
    let derived = derived_summary_view(
        request_execution,
        &mut workspace,
        "computed.materialize.common",
    );
    workspace
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("derived-common-1")),
                    ("title.value", test_string_aspect_value("Derived common")),
                ],
            ),
        )
        .expect("write should materialize derived output");

    let result = workspace
        .materialize_intent(&derived)
        .execute(request_execution)
        .expect("materialize common path should execute");

    assert_eq!(result.row_count(), 1);
    let value_path = retained_test_field_path("value").expect("value path should admit");
    let retained_value = result
        .single_retained_row()
        .expect("single materialized row should be retained")
        .scalar_value_at(&value_path)
        .expect("materialized row should retain value field");
    assert!(
        matches!(retained_value, AspectValue::String(_)),
        "materialized retained value should be a native string scalar, got {retained_value:?}"
    );
    assert_eq!(
        result
            .receipt()
            .consumer_inspection()
            .expect("consumer lane should exist")
            .covered_entrypoint(),
        Some(WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteDerivedMaterialization)
    );
}

#[test]
fn inspect_derived_intent_advanced_path_helper_exposes_request_eligibility_decision_and_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = read_runtime(request_execution);
    let mut workspace = WorthQueryWorkspace::new("derived-inspect-advanced", runtime)
        .expect("workspace should build");
    let derived = derived_summary_view(
        request_execution,
        &mut workspace,
        "computed.inspect.advanced",
    );

    let review = workspace
        .inspect_derived_intent(&derived)
        .review()
        .expect("derived inspection review should succeed");

    assert_eq!(
        review.request().family(),
        WorthQueryIntentAdmissionFamily::InspectionMaterializationIntent
    );
    assert_eq!(
        review.request().entrypoint(),
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteDerivedInspection
    );
    assert_eq!(
        review.eligibility().routing_support_posture(),
        WorthQueryIntentAdmissionRoutingSupportEligibility::CoveredExecutionSeam(
            WorthQueryIntentAdmissionExecutionSeam::QueryRuntimeInspectionMaterializationRoute
        )
    );
    assert!(review.admitted_handoff().is_some());
    assert_eq!(
        review.consumer_inspection().covered_entrypoint(),
        Some(WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteDerivedInspection)
    );
}

#[test]
fn inspect_intent_common_path_helper_executes_through_canonical_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = read_runtime(request_execution);
    let mut workspace = WorthQueryWorkspace::new("generic-inspect-common", runtime)
        .expect("workspace should build");
    let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.table", |q| {
            q.from("Task")
                .select([identity_id_field_key(), title_value_field_key()])
                .order_by(title_value_field_key())
                .schema_basis("intent-admission-generic-inspection-common")
        })
        .expect("live view should declare");

    let result = workspace
        .inspections()
        .expect("inspection lane should be admitted")
        .inspect_intent(&live)
        .execute(request_execution)
        .expect("generic inspection common path should execute");

    assert!(matches!(
        result.inspection(),
        WorthQueryInspection::LiveView(_)
    ));
    assert_eq!(
        result
            .receipt()
            .consumer_inspection()
            .expect("consumer lane should exist")
            .covered_entrypoint(),
        Some(WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteUnifiedInspection)
    );
}

#[test]
fn inspect_intent_advanced_path_helper_exposes_request_eligibility_decision_and_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = read_runtime(request_execution);
    let mut workspace = WorthQueryWorkspace::new("generic-inspect-advanced", runtime)
        .expect("workspace should build");
    let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.table", |q| {
            q.from("Task")
                .select([identity_id_field_key(), title_value_field_key()])
                .order_by(title_value_field_key())
                .schema_basis("intent-admission-generic-inspection-advanced")
        })
        .expect("live view should declare");

    let review = workspace
        .inspections()
        .expect("inspection lane should be admitted")
        .inspect_intent(&live)
        .review()
        .expect("generic inspection review should succeed");

    assert_eq!(
        review.request().family(),
        WorthQueryIntentAdmissionFamily::InspectionMaterializationIntent
    );
    assert_eq!(
        review.request().entrypoint(),
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteUnifiedInspection
    );
    assert_eq!(
        review.eligibility().routing_support_posture(),
        WorthQueryIntentAdmissionRoutingSupportEligibility::CoveredExecutionSeam(
            WorthQueryIntentAdmissionExecutionSeam::QueryRuntimeInspectionMaterializationRoute
        )
    );
    assert!(review.admitted_handoff().is_some());
    assert_eq!(
        review.consumer_inspection().covered_entrypoint(),
        Some(WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteUnifiedInspection)
    );
}
