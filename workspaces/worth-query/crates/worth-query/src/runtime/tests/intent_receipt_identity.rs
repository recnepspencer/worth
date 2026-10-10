use super::intent_receipt_authoritative_identity_composition::*;
use super::intent_receipt_identity_support::*;
use super::intent_receipt_preview_identity_composition::*;
use super::intent_receipt_preview_identity_fixtures::*;
use super::support::*;
use crate::WorthQueryEvidenceScope;

#[path = "intent_receipt_identity/authoritative_intent_receipt_identity_keeps_typed_scope_under_de.rs"]
mod authoritative_intent_receipt_identity_keeps_typed_scope_under_de;

#[test]
fn effect_triggered_intent_receipt_identity_keeps_nested_receipts_typed() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime =
        bridge_backed_runtime_with_support(request_execution, intent_support_profile());
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.effect-identity",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    let effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::write_intent(
            "effects.identity|receipt",
            WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title.value"])),
            "strategy.intent.reconcile",
        ))
        .expect("write-intent effect should declare");

    runtime
        .write(
            request_execution,
            test_update_string_aspect_command(
                crate::memory_workspace::admit_authored_entity_label("task|identity"),
                "title.value",
                "title:identity",
            ),
        )
        .expect("write should route pending effect intent");
    let receipt = runtime
        .execute_next_effect_write_intent(
            request_execution,
            &effect,
            "1.0",
            "effect.intent.input.v1",
        )
        .expect("pending effect intent should execute");

    assert_eq!(
        receipt.receipt_identity().scope(),
        WorthQueryEvidenceScope::EffectIntentReceipt
    );
    assert_eq!(
        receipt.intent_receipt().receipt_identity().scope(),
        WorthQueryEvidenceScope::AuthoritativeIntentReceipt
    );
    assert_eq!(
        receipt.intent_receipt().receipt_identity(),
        &compose_authoritative_effect_triggered_intent_receipt_identity(&receipt),
        "nested authoritative receipt identity must include the effect trigger digest"
    );
    assert_eq!(
        receipt.receipt_identity(),
        &compose_effect_intent_receipt_identity(&receipt),
        "runtime effect intent receipt identity must match independent typed composition"
    );

    let inspection = runtime
        .inspect_effect_intent_receipt(request_execution, &receipt)
        .expect("effect receipt inspection should inspect");
    assert_eq!(
        inspection.phase_identity().scope(),
        WorthQueryEvidenceScope::EffectIntentReceiptPhase
    );
    assert_eq!(
        inspection.inspection_identity().scope(),
        WorthQueryEvidenceScope::EffectIntentReceiptInspection
    );
    assert_eq!(
        inspection.inspection_identity(),
        &compose_effect_intent_receipt_inspection_identity(&inspection),
        "effect inspection identity must match typed composition"
    );
    assert_eq!(
        inspection.feedback_graph().graph_identity().scope(),
        WorthQueryEvidenceScope::FeedbackPhaseGraph
    );
    assert_eq!(
        inspection
            .feedback_graph()
            .trigger_commit_evidence_identity(),
        receipt.trigger_commit_evidence_identity(),
        "feedback graph should retain the typed trigger commit identity"
    );
    assert_eq!(
        inspection.feedback_graph().inspection_identity().scope(),
        WorthQueryEvidenceScope::FeedbackPhaseGraphInspection
    );
    assert_eq!(
        inspection.intent_receipt_identity(),
        receipt.intent_receipt().receipt_identity(),
        "effect inspection should retain the nested authoritative receipt identity"
    );
}

#[test]
fn effect_triggered_intent_receipt_identity_changes_with_nested_authoritative_receipt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut left_runtime =
        bridge_backed_runtime_with_support(request_execution, intent_support_profile());
    let left_effect = declare_identity_effect(
        request_execution,
        &mut left_runtime,
        "tasks.effect-left|identity",
        "effects.identity|receipt",
    );
    left_runtime
        .write(
            request_execution,
            test_update_string_aspect_command(
                crate::memory_workspace::admit_authored_entity_label("task|identity"),
                "title.value",
                "title:left",
            ),
        )
        .expect("left write should route pending effect intent");
    let left = left_runtime
        .execute_next_effect_write_intent(
            request_execution,
            &left_effect,
            "1.0",
            "effect.intent.input.v1",
        )
        .expect("left pending effect intent should execute");

    let mut right_runtime =
        bridge_backed_runtime_with_support(request_execution, intent_support_profile());
    let right_effect = declare_identity_effect(
        request_execution,
        &mut right_runtime,
        "tasks.effect-right|identity",
        "effects.identity",
    );
    right_runtime
        .write(
            request_execution,
            test_update_string_aspect_command(
                crate::memory_workspace::admit_authored_entity_label("identity|task"),
                "title.value",
                "title:right",
            ),
        )
        .expect("right write should route pending effect intent");
    let right = right_runtime
        .execute_next_effect_write_intent(
            request_execution,
            &right_effect,
            "1.0",
            "effect.intent.input.v1",
        )
        .expect("right pending effect intent should execute");

    assert_eq!(
        left.intent_receipt().receipt_identity(),
        &compose_authoritative_effect_triggered_intent_receipt_identity(&left),
        "left delimiter-pressure receipt must match independent authoritative composition"
    );
    assert_eq!(
        right.intent_receipt().receipt_identity(),
        &compose_authoritative_effect_triggered_intent_receipt_identity(&right),
        "right delimiter-pressure receipt must match independent authoritative composition"
    );
    assert_eq!(
        left.receipt_identity(),
        &compose_effect_intent_receipt_identity(&left),
        "left delimiter-pressure effect receipt must match independent composition"
    );
    assert_eq!(
        right.receipt_identity(),
        &compose_effect_intent_receipt_identity(&right),
        "right delimiter-pressure effect receipt must match independent composition"
    );
    assert_ne!(
        left.intent_receipt().receipt_identity(),
        right.intent_receipt().receipt_identity(),
        "nested authoritative receipt should differ under delimiter pressure"
    );
    assert_ne!(
        left.receipt_identity(),
        right.receipt_identity(),
        "effect receipt identity must preserve nested authoritative receipt identity"
    );
}

#[test]
fn preview_intent_receipt_inspection_identity_keeps_basis_and_receipt_typed() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .write_authority(TestWriteAuthority)
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .intent_authority(TestIntentAuthority)
        .support_profile(intent_support_profile())
        .build_backend_from_parts()
        .build(request_execution)
        .expect("intent-capable runtime should build");

    let mut preview = runtime
        .preview_with_options(
            request_execution,
            test_session_label("preview|identity:basis"),
            WorthQueryPreviewOptions::sandboxed_write_intent(),
        )
        .expect("preview session should be admitted");
    let receipt = preview
        .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
            "preview|receipt:test",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            test_intent_input([("entity", "task|preview"), ("title", "preview:identity")]),
        ))
        .expect("preview intent should be admitted");
    let inspection = runtime
        .inspect_preview_intent_receipt(request_execution, &receipt)
        .expect("preview receipt inspection should be available");

    assert_eq!(
        receipt.receipt_identity().scope(),
        WorthQueryEvidenceScope::PreviewIntentReceipt
    );
    assert_eq!(
        receipt.receipt_identity(),
        &compose_preview_intent_receipt_identity(&receipt),
    );
    assert_eq!(
        inspection.basis_identity().scope(),
        WorthQueryEvidenceScope::PreviewIntentReceiptInspectionBasis
    );
    assert_eq!(
        inspection.inspection_identity().scope(),
        WorthQueryEvidenceScope::PreviewIntentReceiptInspection
    );
    assert_eq!(
        inspection.basis_identity(),
        &compose_preview_intent_receipt_inspection_basis_identity(&receipt),
    );
    assert_eq!(
        inspection.inspection_identity(),
        &compose_preview_intent_receipt_inspection_identity(&receipt, inspection.basis_identity()),
    );
    assert_eq!(
        inspection.admission_identity(),
        receipt.admission_identity()
    );
    assert_eq!(inspection.receipt_identity(), receipt.receipt_identity());
}

#[test]
fn preview_intent_receipt_inspection_basis_resists_delimiter_sequence_pressure() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let left = preview_receipt_with_basis(
        request_execution,
        ["alpha", "beta|gamma"],
        "preview|intent",
        test_intent_input([("entity", "preview|task"), ("title", "left")]),
    );
    let right = preview_receipt_with_basis(
        request_execution,
        ["intent|alpha", "beta|gamma"],
        "preview",
        test_intent_input([("entity", "preview|task"), ("title", "right")]),
    );

    assert_ne!(
        left.inspection.basis_identity(),
        right.inspection.basis_identity(),
        "preview basis identity must preserve sequence boundaries under delimiter pressure"
    );
    assert_ne!(
        left.inspection.inspection_identity(),
        right.inspection.inspection_identity(),
        "preview inspection identity must preserve nested basis identity differences"
    );
    assert_eq!(
        left.inspection.receipt_identity().scope(),
        WorthQueryEvidenceScope::PreviewIntentReceipt
    );
    assert_eq!(
        left.inspection.receipt_identity(),
        left.receipt.receipt_identity(),
        "preview inspection should retain the typed preview receipt identity"
    );
    assert_eq!(
        left.inspection.receipt_identity(),
        &compose_preview_intent_receipt_identity(&left.receipt),
        "hostile preview receipt identity must match independent typed composition"
    );
    assert_eq!(
        left.inspection.basis_identity(),
        &compose_preview_intent_receipt_inspection_basis_identity(&left.receipt),
        "hostile preview basis identity must match independent typed composition"
    );
    assert_eq!(
        left.inspection.basis_identity().scope(),
        WorthQueryEvidenceScope::PreviewIntentReceiptInspectionBasis
    );
    assert_eq!(
        left.inspection.inspection_identity().scope(),
        WorthQueryEvidenceScope::PreviewIntentReceiptInspection
    );
    assert_eq!(
        left.inspection.inspection_identity(),
        &compose_preview_intent_receipt_inspection_identity_for_inspection(&left.inspection),
        "preview inspection identity must match independent typed composition"
    );
}
