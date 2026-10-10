use super::*;

#[test]
fn effect_triggered_async_completion_receipt_preserves_write_adjacent_trigger_class() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = intent_runtime_with_authority(request_execution, TestIntentAuthority);
    let origin_identity = test_write_adjacent_origin_identity(
        WorthQueryEffectWriteAdjacentTriggerClass::AsyncCompletion,
        "async-completion:cause:task-title",
    );
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.async-follow-on",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    let effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(
            WorthQueryEffectDeclaration::write_intent(
                "effects.async-follow-on",
                WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title.value"])),
                "strategy.intent.reconcile",
            )
            .with_write_adjacent_trigger(
                WorthQueryEffectWriteAdjacentTriggerClass::AsyncCompletion,
                origin_identity.clone(),
            ),
        )
        .expect("async completion write-intent effect should declare");

    runtime
        .write(
            request_execution,
            test_update_string_aspect_command(
                crate::memory_workspace::admit_authored_entity_label("task-1"),
                "title.value",
                "title from async completion",
            ),
        )
        .expect("write should queue pending intent");

    let receipt = runtime
        .next_effect_write_intent(&effect, "1.0", "effect.intent.input.v1")
        .execute(request_execution)
        .expect("effect-triggered intent should execute");
    let inspection = runtime
        .inspect_effect_intent_receipt(request_execution, &receipt)
        .expect("effect-intent receipt should inspect");

    assert_eq!(
        receipt.write_adjacent_trigger_class(),
        WorthQueryEffectWriteAdjacentTriggerClass::AsyncCompletion
    );
    assert_eq!(
        receipt.write_adjacent_trigger().origin_evidence_identity(),
        &origin_identity
    );
    assert_eq!(
        receipt.intent_receipt().source_lane(),
        WorthQueryIntentSourceLane::EffectTriggered
    );
    assert_eq!(
        inspection.feedback_graph().write_adjacent_trigger_class(),
        WorthQueryEffectWriteAdjacentTriggerClass::AsyncCompletion
    );
    assert_eq!(
        inspection
            .feedback_graph()
            .write_adjacent_trigger_origin_evidence_identity(),
        &origin_identity
    );
}

#[test]
fn consumed_pending_write_intent_cannot_admit_a_second_authority_path() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = intent_runtime_with_authority(request_execution, TestIntentAuthority);
    let origin_identity = test_write_adjacent_origin_identity(
        WorthQueryEffectWriteAdjacentTriggerClass::ReplayDrift,
        "replay-drift:cause:task-title",
    );
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.duplicate-follow-on",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    let effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(
            WorthQueryEffectDeclaration::write_intent(
                "effects.duplicate-follow-on",
                WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title.value"])),
                "strategy.intent.reconcile",
            )
            .with_write_adjacent_trigger(
                WorthQueryEffectWriteAdjacentTriggerClass::ReplayDrift,
                origin_identity,
            ),
        )
        .expect("replay drift write-intent effect should declare");

    runtime
        .write(
            request_execution,
            test_update_string_aspect_command(
                crate::memory_workspace::admit_authored_entity_label("task-1"),
                "title.value",
                "title from replay drift",
            ),
        )
        .expect("write should queue pending intent");

    let (pending_delivery, handoff) = runtime
        .admit_next_effect_write_intent_for_execution(
            effect.name(),
            "1.0",
            "effect.intent.input.v1",
        )
        .expect("initial effect-triggered handoff should admit");
    let binding = runtime.prepare_effect_intent_execution_binding(handoff, &pending_delivery);
    let duplicate_binding = binding.clone();

    runtime
        .execute_effect_intent_execution_binding(request_execution, binding)
        .expect("first effect-triggered execution should succeed");

    let error = runtime
        .execute_effect_intent_execution_binding(request_execution, duplicate_binding)
        .expect_err("the same pending delivery must not execute twice");

    match error {
        WorthQueryRuntimeError::IntentCommitDenied { stage, message, .. } => {
            assert_eq!(stage, "pending-write-intent-binding");
            assert!(message.contains("has no pending write intent delivery"));
        }
        other => panic!("expected duplicate-admission denial, got {other:?}"),
    }
}
