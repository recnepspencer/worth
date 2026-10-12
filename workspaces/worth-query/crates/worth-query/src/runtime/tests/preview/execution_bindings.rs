use super::super::support::*;

#[test]
fn derive_only_preview_binds_handles_and_mutes_effects_without_residue() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.preview-bind",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    let computed = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("computed.preview-bind", test_aspect_touches(["title"]))
                .depends_on_live(&live)
                .produces(test_aspect_touches(["title.summary"])),
            TitleListMaintainer,
        )
        .expect("computed should declare");
    let delivery_effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::deliver(
            "ui.preview-bind",
            WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title"])),
            "ui.preview",
        ))
        .expect("delivery effect should declare");
    let intent_effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::write_intent(
            "intent.preview-bind",
            WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title"])),
            "preview-intent",
        ))
        .expect("write-intent effect should declare");

    let (live_binding, computed_binding, delivery_binding, intent_binding, outcome) = {
        let mut preview = runtime
            .preview(
                request_execution,
                test_session_label("derive-only bindings"),
            )
            .expect("preview session should be admitted");
        let live_binding = preview.use_view(&live);
        let computed_binding = preview.use_computed(&computed);
        let delivery_binding = preview
            .use_effect(request_execution, &delivery_effect)
            .expect("derive-only preview should bind delivery effect muted");
        let intent_binding = preview
            .use_effect(request_execution, &intent_effect)
            .expect("derive-only preview should bind write-intent effect muted");

        assert_eq!(
            live_binding.family(),
            WorthQueryPreviewHandleBindingFamily::LiveView
        );
        assert_eq!(
            live_binding.preview_lane(),
            WorthQueryAuthorityLane::PreviewTruth
        );
        assert_eq!(
            computed_binding.source_lane(),
            WorthQueryAuthorityLane::DerivedRuntimeState
        );
        assert_eq!(
            delivery_binding.effect_disposition(),
            Some(WorthQueryPreviewEffectBindingDisposition::MutedByDeriveOnly)
        );
        assert!(!delivery_binding.effect_delivery_admitted());
        assert_eq!(
            intent_binding.effect_disposition(),
            Some(WorthQueryPreviewEffectBindingDisposition::MutedByDeriveOnly)
        );
        assert!(!intent_binding.pending_write_intent_admitted());

        (
            live_binding,
            computed_binding,
            delivery_binding,
            intent_binding,
            preview.discard(),
        )
    };

    assert_eq!(outcome.preview_binding_count(), 4);
    assert_eq!(outcome.effect_binding_count(), 2);
    assert_eq!(outcome.effect_delivery_residue_count(), 0);
    assert_eq!(outcome.pending_write_intent_residue_count(), 0);
    assert_eq!(outcome.authoritative_residue_count(), 0);
    assert!(runtime
        .drain_effect_deliveries(&delivery_effect)
        .expect("authoritative delivery queue should still exist")
        .is_empty());
    assert!(runtime
        .drain_effect_deliveries(&intent_effect)
        .expect("authoritative intent queue should still exist")
        .is_empty());

    let live_inspection = runtime
        .inspect_preview_binding(request_execution, &live_binding)
        .expect("preview binding inspection should succeed");
    assert_eq!(
        live_inspection.family(),
        WorthQueryPreviewHandleBindingFamily::LiveView
    );
    assert_eq!(
        live_inspection.source_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert_eq!(
        live_inspection.preview_lane(),
        WorthQueryAuthorityLane::PreviewTruth
    );
    assert!(!live_inspection.basis_digest().is_empty());
    assert!(!live_inspection.admission_digest().is_empty());

    let computed_inspection = runtime
        .inspect_preview_binding(request_execution, &computed_binding)
        .expect("computed preview binding inspection should succeed");
    assert_eq!(
        computed_inspection.family(),
        WorthQueryPreviewHandleBindingFamily::ComputedView
    );
    assert_eq!(
        computed_inspection.source_lane(),
        WorthQueryAuthorityLane::DerivedRuntimeState
    );

    let delivery_inspection = runtime
        .inspect_preview_binding(request_execution, &delivery_binding)
        .expect("delivery preview binding inspection should succeed");
    assert_eq!(
        delivery_inspection.effect_disposition(),
        Some("muted-by-derive-only")
    );
    assert!(!delivery_inspection.effect_delivery_admitted());

    let intent_inspection = runtime
        .inspect_preview_binding(request_execution, &intent_binding)
        .expect("intent preview binding inspection should succeed");
    assert_eq!(
        intent_inspection.effect_disposition(),
        Some("muted-by-derive-only")
    );
    assert!(!intent_inspection.pending_write_intent_admitted());

    let outcome_inspection = runtime
        .inspect_preview_outcome(request_execution, &outcome)
        .expect("preview outcome inspection should succeed");
    assert_eq!(
        outcome_inspection.closeout_kind(),
        WorthQueryPreviewCloseoutKind::Discarded
    );
    assert!(outcome_inspection.discarded());
    assert!(!outcome_inspection.promoted());
    assert_eq!(
        outcome_inspection.source_lane(),
        WorthQueryAuthorityLane::PreviewTruth
    );
    assert_eq!(
        outcome_inspection.target_lane(),
        WorthQueryAuthorityLane::PreviewTruth
    );
    assert_eq!(outcome_inspection.effect_binding_count(), 2);
    assert_eq!(outcome_inspection.subscription_residue_count(), 0);
    assert_eq!(outcome_inspection.derived_runtime_residue_count(), 0);
    assert_eq!(outcome_inspection.preview_write_staging_count(), 0);
    assert_eq!(outcome_inspection.promoted_write_count(), 0);
    assert_eq!(outcome_inspection.authoritative_residue_count(), 0);
    assert!(!outcome_inspection.closeout_digest().is_empty());
    assert!(!outcome_inspection.residue_digest().is_empty());
    assert!(!outcome_inspection.inspection_digest().is_empty());
}

#[test]
fn preview_write_routes_bound_live_computed_and_redirected_effect_without_authoritative_residue() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.preview-execution",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    let computed = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new(
                "computed.preview-execution",
                test_aspect_touches(["title"]),
            )
            .depends_on_live(&live)
            .produces(test_aspect_touches(["title.summary"])),
            TitleListMaintainer,
        )
        .expect("computed should declare");
    let delivery_effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::deliver(
            "ui.preview-execution",
            WorthQueryEffectTrigger::computed_view(
                &computed,
                test_aspect_touches(["title.summary"]),
            ),
            "ui.preview",
        ))
        .expect("delivery effect should declare");

    let (execution_evidence, outcome) = {
        let mut preview = runtime
            .preview_with_options(
                request_execution,
                test_session_label("preview execution"),
                WorthQueryPreviewOptions::redirected_delivery(),
            )
            .expect("preview session should be admitted");
        preview.use_view(&live);
        preview.use_computed(&computed);
        preview
            .use_effect(request_execution, &delivery_effect)
            .expect("redirected preview should admit delivery effect");
        preview
            .write(
                request_execution,
                insert_command(
                    "Task",
                    [
                        (
                            "identity.id",
                            test_string_aspect_value("preview-execution-task"),
                        ),
                        (
                            "title.value",
                            test_string_aspect_value("Preview execution task"),
                        ),
                    ],
                ),
            )
            .expect("preview write should stage and route");
        (
            preview.preview_execution_evidence().to_vec(),
            preview.discard(),
        )
    };

    assert!(execution_evidence.iter().any(|evidence| {
        evidence.kind() == WorthQueryPreviewExecutionKind::LivePatch
            && evidence.handle_name() == "tasks.preview-execution"
            && evidence.preview_lane() == WorthQueryAuthorityLane::PreviewTruth
            && evidence.execution_identity().as_str() == evidence.execution_digest()
            && !evidence.execution_digest().is_empty()
    }));
    assert!(execution_evidence.iter().any(|evidence| {
        evidence.kind() == WorthQueryPreviewExecutionKind::ComputedPatch
            && evidence.handle_name() == "computed.preview-execution"
            && evidence.aspect_touches() == test_aspect_touches(["title.summary"]).as_slice()
    }));
    assert!(execution_evidence.iter().any(|evidence| {
        evidence.kind() == WorthQueryPreviewExecutionKind::EffectDelivery
            && evidence.handle_name() == "ui.preview-execution"
    }));

    let closeout = outcome.closeout_evidence();
    assert_eq!(
        closeout.class_count(WorthQueryPreviewResidueClass::SubscriptionState),
        1
    );
    assert_eq!(
        closeout.class_count(WorthQueryPreviewResidueClass::DerivedRuntimeState),
        1
    );
    assert_eq!(closeout.effect_delivery_residue_count(), 1);
    assert_eq!(closeout.pending_write_intent_residue_count(), 0);
    assert_eq!(closeout.authoritative_residue_count(), 0);
    assert!(runtime
        .drain_patches(&live)
        .query_delivery_batches
        .is_empty());
    assert_eq!(
        runtime
            .read_derived_result(request_execution, &computed)
            .expect("computed materialization should execute")
            .row_count(),
        0
    );
    assert!(runtime
        .drain_effect_deliveries(&delivery_effect)
        .expect("authoritative effect queue should exist")
        .is_empty());
}

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
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::write_intent(
            "intent.preview-execution",
            WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title"])),
            "preview-intent",
        ))
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
    let closeout = outcome.closeout_evidence();
    assert_eq!(closeout.effect_delivery_residue_count(), 0);
    assert_eq!(closeout.pending_write_intent_residue_count(), 1);
    assert_eq!(closeout.authoritative_residue_count(), 0);
    assert!(runtime
        .drain_effect_deliveries(&intent_effect)
        .expect("authoritative pending intent queue should exist")
        .is_empty());
}
