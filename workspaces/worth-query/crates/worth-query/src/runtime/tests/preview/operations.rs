use super::super::support::*;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

#[test]
fn sandboxed_preview_run_operation_stages_compiled_writes_until_promote() {
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

    let preview_run = {
        let mut preview = runtime
            .preview_with_options(
                request_execution,
                test_session_label("draft create"),
                WorthQueryPreviewOptions::sandboxed_write_intent(),
            )
            .expect("preview session should be admitted");
        let run = preview
            .run_operation(
                request_execution,
                operation.clone(),
                vec![WorthQueryOperationInput::new(
                    "title",
                    WorthQueryProgramValue::string("Preview-only task"),
                )],
            )
            .expect("preview operation should run");

        assert_eq!(run.write_receipts().len(), 1);
        let draft_label = test_session_label("draft create");
        let expected_commit_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::PreviewWriteReceiptIdentity,
        )
        .field_value(
            WorthQueryEvidenceTag::new("session_label_identity"),
            draft_label.identity_digest().as_str(),
        )
        .field_usize(WorthQueryEvidenceTag::new("sequence"), 1)
        .seal();
        assert_eq!(
            run.write_receipts()[0]
                .commit_identity()
                .evidence_identity(),
            expected_commit_identity
        );
        assert_eq!(
            run.write_receipts()[0].authority_lane(),
            WorthQueryAuthorityLane::PreviewTruth
        );
        run
    };

    assert_eq!(
        preview_run.outputs()[0]
            .value()
            .array_len()
            .expect("preview output should be an array"),
        0
    );

    {
        let mut preview = runtime
            .preview_with_options(
                request_execution,
                test_session_label("promote create"),
                WorthQueryPreviewOptions::sandboxed_write_intent(),
            )
            .expect("preview session should be admitted");
        preview
            .run_operation(
                request_execution,
                operation,
                vec![WorthQueryOperationInput::new(
                    "title",
                    WorthQueryProgramValue::string("Promoted preview task"),
                )],
            )
            .expect("preview operation should stage");
        let outcome = preview
            .promote(request_execution)
            .expect("preview promotion should succeed");
        assert!(outcome.promoted());
        assert_eq!(outcome.write_count(), 1);
        assert_eq!(
            outcome.effect_policy(),
            WorthQueryEffectPolicy::SandboxedWriteIntent
        );
        assert_eq!(outcome.source_lane(), WorthQueryAuthorityLane::PreviewTruth);
        assert_eq!(
            outcome.target_lane(),
            WorthQueryAuthorityLane::AuthoritativeTruth
        );
    }

    let view = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.after-preview",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    let rows = runtime.read_live(request_execution, &view);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        test_native_string_value(&rows[0], "title.value").as_deref(),
        Some("Promoted preview task")
    );
}

#[test]
fn preview_run_operation_discard_keeps_authoritative_state_unchanged() {
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

    {
        let mut preview = runtime
            .preview_with_options(
                request_execution,
                test_session_label("discard create"),
                WorthQueryPreviewOptions::sandboxed_write_intent(),
            )
            .expect("preview session should be admitted");
        preview
            .run_operation(
                request_execution,
                operation,
                vec![WorthQueryOperationInput::new(
                    "title",
                    WorthQueryProgramValue::string("Discarded preview task"),
                )],
            )
            .expect("preview operation should stage");
        let outcome = preview.discard();
        assert!(outcome.discarded());
    }

    let view = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.after-discard",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    assert!(runtime.read_live(request_execution, &view).is_empty());
}

#[test]
fn preview_run_operation_rejects_declaration_effects_before_runtime_mutation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let program =
        WorthQueryProgram::compile(FakeDsl, &FakeSchemaAdapter).expect("fake DSL should compile");
    let installed = runtime
        .install_program(program)
        .expect("program should install");
    let operation = installed
        .operation("create_task")
        .expect("operation ref should build");

    let mut preview = runtime
        .preview_with_options(
            request_execution,
            test_session_label("deny declaration effects"),
            WorthQueryPreviewOptions::sandboxed_write_intent(),
        )
        .expect("preview session should be admitted");
    let error = preview
        .run_operation(
            request_execution,
            operation,
            vec![WorthQueryOperationInput::new(
                "title",
                WorthQueryProgramValue::string("Should never install runtime declarations"),
            )],
        )
        .expect_err("preview operation must deny declaration effects");

    match error {
        WorthQueryRuntimeError::PreviewOperationEffectDenied {
            label,
            stage,
            message,
        } => {
            assert_eq!(label, test_session_label("deny declaration effects"));
            assert_eq!(stage, "effect-admission");
            assert!(message.contains("cannot install live view `tasks.table`"));
        }
        other => panic!("expected preview declaration denial, got {other:?}"),
    }

    let view = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.after-denied-declaration",
            task_live_request(),
            task_schema(),
        )
        .expect("authoritative runtime should remain available after denial");
    assert!(runtime.read_live(request_execution, &view).is_empty());
}
