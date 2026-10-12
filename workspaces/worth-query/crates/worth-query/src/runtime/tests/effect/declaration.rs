use super::super::support::*;

#[test]
fn effect_declaration_rejects_missing_triggers_before_registration() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let missing = WorthQueryEffectDeclaration::deliver(
        "ui.missing",
        WorthQueryEffectTrigger::live_view_name("tasks.missing", test_aspect_touches(["title"])),
        "ui.badges",
    );
    let error = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(missing)
        .expect_err("missing live trigger should reject");

    match error {
        WorthQueryRuntimeError::EffectDeclaration {
            effect_name,
            stage,
            message,
        } => {
            assert_eq!(effect_name, "ui.missing");
            assert_eq!(stage, "trigger-admission");
            assert!(message.contains("tasks.missing"));
        }
        other => panic!("expected effect declaration denial, got {other:?}"),
    }
}

#[test]
fn effect_declaration_rejects_truth_delivery_without_intent_boundary() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.table",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    let declaration = WorthQueryEffectDeclaration::deliver(
        "ui.truth-smuggle",
        WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title"])),
        "Task",
    )
    .with_target_lane(WorthQueryAuthorityLane::AuthoritativeTruth);

    let error = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(declaration)
        .expect_err("effect delivery must not target truth");

    match error {
        WorthQueryRuntimeError::EffectDeclaration { stage, message, .. } => {
            assert_eq!(stage, "authority-admission");
            assert!(message.contains("intent authority"));
        }
        other => panic!("expected authority admission denial, got {other:?}"),
    }
}
