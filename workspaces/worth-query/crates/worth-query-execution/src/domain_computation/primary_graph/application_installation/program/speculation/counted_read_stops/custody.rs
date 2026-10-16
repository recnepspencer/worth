use super::*;

#[test]
fn preview_readmission_retains_original_source_and_runtime_until_promotion() {
    let runtime = installed_preview_runtime();
    let other = installed_preview_runtime();
    let observation = runtime
        .with_host_advancement(|phase| {
            runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap()
                .retain_application_read()
        })
        .unwrap();
    let preview = runtime
        .with_host_advancement(|phase| {
            runtime.begin_preview(
                &phase,
                &observation,
                WorthQueryApplicationPreviewRequest::new("fresh-preview-promotion"),
            )
        })
        .unwrap()
        .unwrap();
    let original_source = std::sync::Arc::clone(&preview._source);
    let observer = preview.handle.liveness_observer();
    let original_execution = preview.handle.execution_record().digest().to_owned();
    let (preview, denial) = other
        .with_host_advancement(|phase| preview.readmit(&phase, &runtime))
        .unwrap()
        .unwrap_err();
    assert_eq!(
        denial,
        WorthQueryApplicationPreviewReadmissionDenial::ForeignPhase
    );
    assert!(std::sync::Arc::ptr_eq(&preview._source, &original_source));
    let (preview, denial) = other
        .with_host_advancement(|phase| preview.readmit(&phase, &other))
        .unwrap()
        .unwrap_err();
    assert_eq!(
        denial,
        WorthQueryApplicationPreviewReadmissionDenial::ForeignRuntime
    );
    let preview = runtime
        .with_host_advancement(|phase| preview.readmit(&phase, &runtime))
        .unwrap()
        .unwrap();
    assert!(observer.admit_active_session().is_some());
    assert!(std::sync::Arc::ptr_eq(
        &preview.session._source,
        &original_source
    ));
    assert_eq!(
        preview.session.handle.execution_record().digest(),
        original_execution
    );
    let outcome = preview.promote().unwrap();
    assert_eq!(
        outcome.session().session_identity().as_str(),
        "fresh-preview-promotion:session"
    );
    assert!(observer.admit_active_session().is_none());
    assert_eq!(
        runtime
            .runtime()
            .bridge
            .diagnostics()
            .preview_execution_records()
            .len(),
        1
    );
    assert_eq!(
        runtime
            .runtime()
            .bridge
            .diagnostics()
            .preview_promotion_records()
            .len(),
        1
    );
    assert!(runtime
        .runtime()
        .bridge
        .diagnostics()
        .preview_discard_records()
        .is_empty());
}

#[test]
fn readmitted_preview_discard_refusal_retains_source_and_liveness_until_real_close() {
    let runtime = installed_preview_runtime();
    let observation = runtime
        .with_host_advancement(|phase| {
            runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap()
                .retain_application_read()
        })
        .unwrap();
    let mut preview = runtime
        .with_host_advancement(|phase| {
            runtime.begin_preview(
                &phase,
                &observation,
                WorthQueryApplicationPreviewRequest::new("readmitted-discard-refusal"),
            )
        })
        .unwrap()
        .unwrap();
    let source = std::sync::Arc::clone(&preview._source);
    let observer = preview.handle.liveness_observer();
    let fault = preview.reject_discard_for_test();
    let preview = runtime
        .with_host_advancement(|phase| preview.readmit(&phase, &runtime))
        .unwrap()
        .unwrap();
    let (preview, denial) = preview.discard().unwrap_err();
    assert_eq!(denial.kind(), worth_runtime_bridge::facade::BridgeSpeculationErrorKind::PreviewResidueClassificationMismatch);
    assert!(std::sync::Arc::ptr_eq(&preview.session._source, &source));
    assert!(observer.admit_active_session().is_some());
    assert!(runtime
        .runtime()
        .bridge
        .diagnostics()
        .preview_discard_records()
        .is_empty());
    drop(fault);
    preview.discard().unwrap();
    assert!(observer.admit_active_session().is_none());
    assert_eq!(
        runtime
            .runtime()
            .bridge
            .diagnostics()
            .preview_execution_records()
            .len(),
        1
    );
    assert_eq!(
        runtime
            .runtime()
            .bridge
            .diagnostics()
            .preview_discard_records()
            .len(),
        1
    );
    assert!(runtime
        .runtime()
        .bridge
        .diagnostics()
        .preview_promotion_records()
        .is_empty());
}

#[test]
fn preview_keeps_installed_runtime_alive_until_explicit_discard() {
    let mut runtime = installed_preview_runtime();
    let observation = runtime
        .with_host_advancement(|phase| {
            runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap()
                .retain_application_read()
        })
        .unwrap();
    let preview = runtime
        .with_host_advancement(|phase| {
            runtime.begin_preview(
                &phase,
                &observation,
                WorthQueryApplicationPreviewRequest::new("preview-runtime-custody"),
            )
        })
        .unwrap()
        .unwrap();
    let observer = preview.handle.liveness_observer();
    assert_eq!(runtime.close_conditional_runtime().unwrap_err().kind(),
        crate::domain_computation::primary_graph::WorthQueryConditionalRuntimeInstallationDenialKind::RuntimeKeepaliveRetained);
    assert!(observer.admit_active_session().is_some());
    preview.discard().unwrap();
    assert!(observer.admit_active_session().is_none());
    drop(observation);
    runtime.close_conditional_runtime().unwrap();
}
