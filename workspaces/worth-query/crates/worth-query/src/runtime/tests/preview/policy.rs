use super::super::support::*;

#[test]
fn runtime_surfaces_authority_lanes_on_public_handles_and_receipts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.authority",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    let derived = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("task_titles.authority", test_aspect_touches(["title"])),
            TitleListMaintainer,
        )
        .expect("derived view should declare");

    let receipt = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    (
                        "title.value",
                        test_string_aspect_value("Authority lane task"),
                    ),
                ],
            ),
        )
        .expect("insert should write");
    let patches = runtime.drain_derived_patches(&derived);
    let inspector = runtime.inspect_receipt(&receipt);

    assert_eq!(
        live.authority_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert_eq!(
        derived.authority_lane(),
        WorthQueryAuthorityLane::DerivedRuntimeState
    );
    assert_eq!(
        receipt.authority_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert_eq!(
        inspector.authority_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert_eq!(
        patches.derived_patches[0].authority_lane(),
        WorthQueryAuthorityLane::DerivedRuntimeState
    );
}

#[test]
fn preview_defaults_to_derive_only_effect_policy_but_keeps_explicit_writes_preview_local() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let mut preview = runtime
        .preview(request_execution, test_session_label("default policy"))
        .expect("preview session should be admitted");

    assert_eq!(preview.effect_policy(), WorthQueryEffectPolicy::DeriveOnly);
    assert!(preview
        .admit_effect_action(
            WorthQueryEffectAction::Derive,
            WorthQueryAuthorityLane::DerivedRuntimeState
        )
        .is_ok());

    let delivery_denial = preview
        .admit_effect_action(
            WorthQueryEffectAction::Deliver,
            WorthQueryAuthorityLane::EffectDeliveryState,
        )
        .expect_err("derive-only preview should deny effect delivery");
    assert!(matches!(
        delivery_denial,
        WorthQueryRuntimeError::EffectPolicyDenied(_)
    ));

    let write_denial = preview
        .admit_effect_action(
            WorthQueryEffectAction::WriteIntent,
            WorthQueryAuthorityLane::AuthoritativeTruth,
        )
        .expect_err("derive-only preview should deny authoritative write intent");
    assert!(matches!(
        write_denial,
        WorthQueryRuntimeError::EffectPolicyDenied(_)
    ));

    let preview_receipt = preview
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    (
                        "title.value",
                        test_string_aspect_value("Preview-local task"),
                    ),
                ],
            ),
        )
        .expect("explicit preview write should stage");
    assert_eq!(
        preview_receipt.authority_lane(),
        WorthQueryAuthorityLane::PreviewTruth
    );

    let outcome = preview.discard();
    assert_eq!(outcome.effect_policy(), WorthQueryEffectPolicy::DeriveOnly);
    assert_eq!(outcome.source_lane(), WorthQueryAuthorityLane::PreviewTruth);
    assert_eq!(outcome.target_lane(), WorthQueryAuthorityLane::PreviewTruth);
}

#[test]
fn sandboxed_preview_policy_admits_only_sandboxed_write_intents() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let preview = runtime
        .preview_with_options(
            request_execution,
            test_session_label("sandboxed writes"),
            WorthQueryPreviewOptions::sandboxed_write_intent(),
        )
        .expect("preview session should be admitted");

    let admission = preview
        .admit_effect_action(
            WorthQueryEffectAction::WriteIntent,
            WorthQueryAuthorityLane::PreviewTruth,
        )
        .expect("sandboxed write intent should be admitted to preview truth");
    assert_eq!(
        admission.policy(),
        WorthQueryEffectPolicy::SandboxedWriteIntent
    );
    assert_eq!(admission.action(), WorthQueryEffectAction::WriteIntent);
    assert_eq!(
        admission.target_lane(),
        WorthQueryAuthorityLane::PreviewTruth
    );

    let denial = preview
        .admit_effect_action(
            WorthQueryEffectAction::WriteIntent,
            WorthQueryAuthorityLane::AuthoritativeTruth,
        )
        .expect_err("sandboxed write intent must not target authoritative truth");
    assert!(matches!(
        denial,
        WorthQueryRuntimeError::EffectPolicyDenied(_)
    ));
}

#[test]
fn derive_only_preview_denies_operation_write_effects() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.table",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare before preview-safe operation runs");
    let program = preview_safe_program();
    let installed = runtime
        .install_program(program)
        .expect("program should install");
    let operation = installed
        .operation("create_task")
        .expect("operation ref should build");

    let mut preview = runtime
        .preview(
            request_execution,
            test_session_label("derive-only operation"),
        )
        .expect("preview session should be admitted");
    let error = preview
        .run_operation(
            request_execution,
            operation,
            vec![WorthQueryOperationInput::new(
                "title",
                WorthQueryProgramValue::string("Should not stage"),
            )],
        )
        .expect_err("derive-only preview should deny write-effect operations");

    assert!(matches!(
        error,
        WorthQueryRuntimeError::EffectPolicyDenied(_)
    ));
    assert_eq!(preview.compare_to_authoritative().write_count(), 0);
}
