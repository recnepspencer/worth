use super::support::*;

pub(super) fn declare_identity_effect(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut WorthQueryRuntime,
    live_name: &str,
    effect_name: &str,
) -> WorthQueryEffectHandle<WorthQueryUnrefinedLiveShape> {
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            live_name,
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::write_intent(
            effect_name,
            WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title.value"])),
            "strategy.intent.reconcile",
        ))
        .expect("write-intent effect should declare")
}
