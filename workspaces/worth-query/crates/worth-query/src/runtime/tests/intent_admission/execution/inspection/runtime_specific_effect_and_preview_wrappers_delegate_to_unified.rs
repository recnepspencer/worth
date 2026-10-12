//! runtime specific effect and preview wrappers delegate to unified inspection execution.
use super::*;

#[test]
fn runtime_specific_effect_and_preview_wrappers_delegate_to_unified_inspection_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.wrapper-delegation",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    let effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::deliver(
            "ui.wrapper-delegation",
            WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title"])),
            "ui.preview",
        ))
        .expect("effect should declare");

    let effect_delegated = runtime
        .inspect_effect(request_execution, &effect)
        .expect("effect wrapper should inspect");
    let effect_canonical = runtime
        .inspect_intent(&effect)
        .execute(request_execution)
        .expect("canonical effect inspection should execute");
    match effect_canonical.inspection() {
        WorthQueryInspection::Effect(inspection) => assert_eq!(effect_delegated, *inspection),
        other => panic!("expected effect inspection, got {other:?}"),
    }

    let outcome = {
        let mut preview = runtime
            .preview(
                request_execution,
                test_session_label("wrapper delegation preview"),
            )
            .expect("preview session should be admitted");
        preview.use_view(&live);
        preview.discard()
    };

    let preview_delegated = runtime
        .inspect_preview_outcome(request_execution, &outcome)
        .expect("preview outcome wrapper should inspect");
    let preview_canonical = runtime
        .inspect_intent(&outcome)
        .execute(request_execution)
        .expect("canonical preview inspection should execute");
    match preview_canonical.inspection() {
        WorthQueryInspection::PreviewOutcome(inspection) => {
            assert_eq!(preview_delegated, *inspection)
        }
        other => panic!("expected preview outcome inspection, got {other:?}"),
    }
}
