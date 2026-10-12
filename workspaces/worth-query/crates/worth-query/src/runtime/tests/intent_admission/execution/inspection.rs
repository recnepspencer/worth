use super::*;

fn derived_titles_view(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut WorthQueryRuntime,
    view_name: &str,
) -> WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape> {
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.table",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new(view_name, test_aspect_touches(["title"]))
                .depends_on_live(&live)
                .produces(test_aspect_touches(["title.summary"])),
            TitleListMaintainer,
        )
        .expect("derived view should declare")
}

#[test]
fn workspace_materialize_result_delegates_to_derived_materialization_intent_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = read_runtime(request_execution);
    let derived = derived_titles_view(
        request_execution,
        &mut runtime,
        "computed.intent.materialize",
    );
    runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("materialize-1")),
                    ("title.value", test_string_aspect_value("Materialize me")),
                ],
            ),
        )
        .expect("write should materialize derived output");
    let mut workspace = WorthQueryWorkspace::new("materialize-delegation", runtime)
        .expect("workspace should build");

    let delegated = workspace
        .materialize_result(request_execution, &derived)
        .expect("derived materialization should execute");
    let canonical = workspace
        .materialize_intent(&derived)
        .execute(request_execution)
        .expect("canonical materialization should execute");

    assert_eq!(delegated, canonical);
    assert_eq!(canonical.receipt().view_name(), derived.name());
    assert_eq!(
        canonical
            .receipt()
            .execution_provenance()
            .map(|provenance| provenance.entrypoint()),
        Some(WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteDerivedMaterialization)
    );
}

#[test]
fn runtime_read_derived_result_delegates_to_derived_materialization_intent_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = read_runtime(request_execution);
    let derived = derived_titles_view(
        request_execution,
        &mut runtime,
        "computed.intent.runtime-materialize",
    );
    runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    (
                        "identity.id",
                        test_string_aspect_value("runtime-materialize-1"),
                    ),
                    (
                        "title.value",
                        test_string_aspect_value("Runtime materialize me"),
                    ),
                ],
            ),
        )
        .expect("write should materialize derived output");

    let delegated = runtime
        .read_derived_result(request_execution, &derived)
        .expect("derived materialization should execute");
    let canonical = runtime
        .review_runtime_derived_materialization(derived.name().to_string())
        .and_then(|review| {
            runtime.resolve_reviewed_admitted_derived_materialization_handoff(review)
        })
        .map(|handoff| runtime.prepare_derived_materialization_execution_binding(handoff))
        .and_then(|binding| {
            runtime.execute_derived_materialization_execution_binding(request_execution, binding)
        })
        .expect("canonical derived materialization should execute");

    assert_eq!(delegated, canonical);
    assert_eq!(canonical.receipt().view_name(), derived.name());
    assert_eq!(
        canonical
            .receipt()
            .execution_provenance()
            .map(|provenance| provenance.entrypoint()),
        Some(WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteDerivedMaterialization)
    );
}

#[test]
fn workspace_inspect_derived_view_delegates_to_derived_inspection_intent_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = read_runtime(request_execution);
    let derived = derived_titles_view(request_execution, &mut runtime, "computed.intent.inspect");
    runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("inspect-1")),
                    ("title.value", test_string_aspect_value("Inspect me")),
                ],
            ),
        )
        .expect("write should materialize derived output");
    let mut workspace =
        WorthQueryWorkspace::new("inspect-delegation", runtime).expect("workspace should build");

    let delegated = workspace
        .inspect(request_execution, &derived)
        .expect("legacy derived inspection should succeed");
    let canonical = workspace
        .inspect_derived_intent(&derived)
        .execute(request_execution)
        .expect("canonical derived inspection should execute");

    match delegated {
        WorthQueryInspection::DerivedView(evidence) => {
            assert_eq!(&evidence, canonical.evidence());
        }
        other => panic!("expected derived inspection, got {other:?}"),
    }
    assert_eq!(
        canonical
            .receipt()
            .execution_provenance()
            .map(|provenance| provenance.entrypoint()),
        Some(WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteDerivedInspection)
    );
}

#[test]
fn runtime_inspect_derived_view_delegates_to_derived_inspection_intent_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = read_runtime(request_execution);
    let derived = derived_titles_view(
        request_execution,
        &mut runtime,
        "computed.intent.runtime-inspect",
    );
    runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("runtime-inspect-1")),
                    (
                        "title.value",
                        test_string_aspect_value("Runtime inspect me"),
                    ),
                ],
            ),
        )
        .expect("write should materialize derived output");

    let delegated = runtime
        .inspect_derived_view(request_execution, &derived)
        .expect("legacy runtime derived inspection should succeed");
    let canonical = runtime
        .review_runtime_derived_inspection(derived.name().to_string())
        .and_then(|review| runtime.resolve_reviewed_admitted_derived_inspection_handoff(review))
        .map(|handoff| runtime.prepare_derived_inspection_execution_binding(handoff))
        .and_then(|binding| {
            runtime.execute_derived_inspection_execution_binding(request_execution, binding)
        })
        .expect("canonical derived inspection should execute");

    assert_eq!(&delegated, canonical.evidence());
    assert_eq!(
        canonical
            .receipt()
            .execution_provenance()
            .map(|provenance| provenance.entrypoint()),
        Some(WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteDerivedInspection)
    );
}

#[test]
fn workspace_inspection_lane_delegates_to_unified_inspection_intent_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = read_runtime(request_execution);
    let mut workspace =
        WorthQueryWorkspace::new("generic-inspect-delegation", runtime).expect("workspace build");
    let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
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
                .schema_basis("intent-admission-generic-inspection-delegation")
        })
        .expect("live view should declare");

    let inspections = workspace
        .inspections()
        .expect("inspection lane should be admitted");
    let delegated = inspections
        .inspect(request_execution, &live)
        .expect("inspection lane should inspect");
    let canonical = inspections
        .inspect_intent(&live)
        .execute(request_execution)
        .expect("canonical generic inspection should execute");

    assert_eq!(delegated, canonical.inspection().clone());
    assert_eq!(
        canonical
            .receipt()
            .execution_provenance()
            .map(|provenance| provenance.entrypoint()),
        Some(WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteUnifiedInspection)
    );
}

#[test]
fn runtime_inspect_live_view_delegates_to_unified_inspection_intent_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = read_runtime(request_execution);
    let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.table",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");

    let delegated = runtime
        .inspect(request_execution, &live)
        .expect("runtime generic inspection should succeed");
    let canonical = runtime
        .inspect_intent(&live)
        .execute(request_execution)
        .expect("canonical generic inspection should execute");

    assert_eq!(delegated, canonical.inspection().clone());
    assert_eq!(
        canonical
            .receipt()
            .execution_provenance()
            .map(|provenance| provenance.entrypoint()),
        Some(WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteUnifiedInspection)
    );
}

#[test]
fn runtime_specific_intent_inspection_wrappers_delegate_to_unified_inspection_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = intent_runtime_with_authority(request_execution, TestIntentAuthority);
    let receipt = runtime
        .execute_intent(
            request_execution,
            WorthQueryIntentDeclaration::strategy_commit(
                "delegated-intent-receipt",
                "strategy.intent.reconcile",
                "1.0",
                "intent.reconcile.input.v1",
                test_intent_input([("entity", "task-1"), ("title", "Delegated intent title")]),
            ),
        )
        .expect("intent should execute");

    let delegated = runtime
        .inspect_intent_receipt(request_execution, &receipt)
        .expect("direct wrapper should inspect");
    let canonical = runtime
        .inspect_intent(&receipt)
        .execute(request_execution)
        .expect("canonical inspection should execute");

    match canonical.inspection() {
        WorthQueryInspection::IntentReceipt(inspection) => assert_eq!(delegated, *inspection),
        other => panic!("expected intent receipt inspection, got {other:?}"),
    }
    assert_eq!(
        canonical
            .receipt()
            .execution_provenance()
            .map(|provenance| provenance.entrypoint()),
        Some(WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteUnifiedInspection)
    );
}

#[test]
fn runtime_specific_intent_denial_wrapper_delegates_to_unified_inspection_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime =
        intent_runtime_with_authority(request_execution, InvariantViolationIntentAuthority);
    let error = runtime
        .execute_intent(
            request_execution,
            WorthQueryIntentDeclaration::strategy_commit(
                "delegated-intent-denial",
                "strategy.intent.reconcile",
                "1.0",
                "intent.reconcile.input.v1",
                test_intent_input([("entity", "task-1"), ("dependency", "cycle")]),
            ),
        )
        .expect_err("invariant violation must deny");
    let evidence = match error {
        WorthQueryRuntimeError::IntentCommitDenied { evidence, .. } => evidence,
        other => panic!("expected intent denial, got {other:?}"),
    };

    let delegated = runtime
        .inspect_intent_denial(request_execution, &evidence)
        .expect("direct denial wrapper should inspect");
    let canonical = runtime
        .inspect_intent(&evidence)
        .execute(request_execution)
        .expect("canonical inspection should execute");

    match canonical.inspection() {
        WorthQueryInspection::IntentDenial(inspection) => assert_eq!(delegated, *inspection),
        other => panic!("expected intent denial inspection, got {other:?}"),
    }
}

#[path = "inspection/runtime_specific_effect_and_preview_wrappers_delegate_to_unified.rs"]
mod runtime_specific_effect_and_preview_wrappers_delegate_to_unified;
