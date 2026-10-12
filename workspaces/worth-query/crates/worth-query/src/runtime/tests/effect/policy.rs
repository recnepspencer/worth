use super::super::support::*;

#[test]
fn preview_effect_policy_bindings_distinguish_delivery_and_write_intent() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.preview-policy",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    let delivery_effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::deliver(
            "ui.preview-policy",
            WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title"])),
            "ui.preview",
        ))
        .expect("delivery effect should declare");
    let intent_effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::write_intent(
            "intent.preview-policy",
            WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title"])),
            "preview-intent",
        ))
        .expect("write-intent effect should declare");

    let muted = {
        let mut preview = runtime
            .preview_with_options(
                request_execution,
                test_session_label("muted effect"),
                WorthQueryPreviewOptions::derive_only()
                    .with_effect_policy(WorthQueryEffectPolicy::Muted),
            )
            .expect("preview session should be admitted");
        preview
            .use_effect(request_execution, &delivery_effect)
            .expect("muted policy should bind but not activate")
    };
    assert_eq!(
        muted.effect_disposition(),
        Some(WorthQueryPreviewEffectBindingDisposition::Muted)
    );
    assert!(!muted.effect_delivery_admitted());

    let redirected = {
        let mut preview = runtime
            .preview_with_options(
                request_execution,
                test_session_label("redirected effect"),
                WorthQueryPreviewOptions::derive_only()
                    .with_effect_policy(WorthQueryEffectPolicy::Redirected),
            )
            .expect("preview session should be admitted");
        preview
            .use_effect(request_execution, &delivery_effect)
            .expect("redirected policy should admit preview delivery")
    };
    assert_eq!(
        redirected.effect_disposition(),
        Some(WorthQueryPreviewEffectBindingDisposition::RedirectedDelivery)
    );
    assert!(redirected.effect_delivery_admitted());

    let sandboxed = {
        let mut preview = runtime
            .preview_with_options(
                request_execution,
                test_session_label("sandboxed effect"),
                WorthQueryPreviewOptions::derive_only()
                    .with_effect_policy(WorthQueryEffectPolicy::SandboxedWriteIntent),
            )
            .expect("preview session should be admitted");
        preview
            .use_effect(request_execution, &intent_effect)
            .expect("sandboxed policy should admit preview write intent")
    };
    assert_eq!(
        sandboxed.effect_disposition(),
        Some(WorthQueryPreviewEffectBindingDisposition::SandboxedWriteIntent)
    );
    assert!(sandboxed.pending_write_intent_admitted());

    let denied = {
        let mut preview = runtime
            .preview_with_options(
                request_execution,
                test_session_label("sandboxed delivery denial"),
                WorthQueryPreviewOptions::derive_only()
                    .with_effect_policy(WorthQueryEffectPolicy::SandboxedWriteIntent),
            )
            .expect("preview session should be admitted");
        preview
            .use_effect(request_execution, &delivery_effect)
            .expect_err("sandboxed write intent policy should not admit delivery effects")
    };
    assert!(matches!(
        denied,
        WorthQueryRuntimeError::EffectPolicyDenied(_)
    ));
}
