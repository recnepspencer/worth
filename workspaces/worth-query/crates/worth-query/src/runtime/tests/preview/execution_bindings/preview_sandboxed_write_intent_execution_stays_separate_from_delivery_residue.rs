use super::*;

#[test]
fn preview_sandboxed_write_intent_execution_stays_separate_from_delivery_residue() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.preview-intent-exec",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    let intent_effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(
            WorthQueryEffectDeclaration::write_intent(
                "intent.preview-execution",
                WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title"])),
                "preview-intent",
            )
            .expect("fixture digest must fit"),
        )
        .expect("write-intent effect should declare");

    let (execution_evidence, outcome) = {
        let mut preview = runtime
            .preview_with_options(
                request_execution,
                test_session_label("preview intent execution"),
                WorthQueryPreviewOptions::sandboxed_write_intent(),
            )
            .expect("preview session should be admitted");
        preview.use_view(&live);
        preview
            .use_effect(request_execution, &intent_effect)
            .expect("sandboxed preview should admit write-intent effect");
        preview
            .write(
                request_execution,
                insert_command(
                    "Task",
                    [
                        (
                            "identity.id",
                            test_string_aspect_value("preview-intent-task"),
                        ),
                        (
                            "title.value",
                            test_string_aspect_value("Preview intent task"),
                        ),
                    ],
                ),
            )
            .expect("preview write should route pending intent");
        (
            preview.preview_execution_evidence().to_vec(),
            preview.discard(),
        )
    };

    assert!(execution_evidence.iter().any(|evidence| {
        evidence.kind() == WorthQueryPreviewExecutionKind::PendingWriteIntent
            && evidence.handle_name() == "intent.preview-execution"
    }));
    let closeout = outcome
        .expect("fixture digest must fit")
        .closeout_evidence();
    assert_eq!(closeout.effect_delivery_residue_count(), 0);
    assert_eq!(closeout.pending_write_intent_residue_count(), 1);
    assert_eq!(closeout.authoritative_residue_count(), 0);
    assert!(runtime
        .drain_effect_deliveries(&intent_effect)
        .expect("authoritative pending intent queue should exist")
        .is_empty());
}
