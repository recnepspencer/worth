use crate::domain_computation::primary_graph::application_installation::program::speculation::counted_read_stops::*;

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
    let original_identity = (*preview.handle_for_test()).session_identity().clone();
    let original_source = std::sync::Arc::clone(preview._source_for_test());
    let observer = (*preview.handle_for_test()).liveness_observer();
    let (preview, denial) = other
        .with_host_advancement(|phase| preview.readmit(&phase, &runtime))
        .unwrap()
        .unwrap_err();
    assert_eq!(
        denial,
        WorthQueryApplicationPreviewReadmissionDenial::ForeignPhase
    );
    assert!(std::sync::Arc::ptr_eq(
        preview._source_for_test(),
        &original_source
    ));
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
        (*preview.session_for_test())._source_for_test(),
        &original_source
    ));
    assert_eq!(
        preview
            .session_for_test()
            .handle_for_test()
            .liveness_observer(),
        observer
    );
    let outcome = preview.promote();
    assert_eq!(outcome.session_identity(), &original_identity);
    assert!(observer.admit_active_session().is_none());
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
    let source = std::sync::Arc::clone(preview._source_for_test());
    let observer = (*preview.handle_for_test()).liveness_observer();
    let fault = preview.reject_discard_for_test();
    let preview = runtime
        .with_host_advancement(|phase| preview.readmit(&phase, &runtime))
        .unwrap()
        .unwrap();
    let (preview, denial) = preview.discard().unwrap_err();
    assert_eq!(denial, worth_query_execution::facade::application_installation::WorthQueryApplicationPreviewDiscardDenial::CleanupRefused);
    assert!(std::sync::Arc::ptr_eq(
        (*preview.session_for_test())._source_for_test(),
        &source
    ));
    assert!(observer.admit_active_session().is_some());
    drop(fault);
    preview.discard().unwrap();
    assert!(observer.admit_active_session().is_none());
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
    let observer = (*preview.handle_for_test()).liveness_observer();
    assert_eq!(runtime.close_conditional_runtime().unwrap_err().kind(),
        worth_query_execution::facade::domain_computation::WorthQueryConditionalRuntimeInstallationDenialKind::RuntimeKeepaliveRetained);
    assert!(observer.admit_active_session().is_some());
    preview.discard().unwrap();
    assert!(observer.admit_active_session().is_none());
    drop(observation);
    runtime.close_conditional_runtime().unwrap();
}
