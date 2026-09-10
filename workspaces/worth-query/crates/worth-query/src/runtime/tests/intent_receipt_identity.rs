use super::intent_receipt_authoritative_identity_composition::*;
use super::intent_receipt_identity_support::*;
use super::intent_receipt_preview_identity_composition::*;
use super::intent_receipt_preview_identity_fixtures::*;
use super::support::*;
use crate::WorthQueryEvidenceScope;

#[test]
fn authoritative_intent_receipt_identity_keeps_typed_scope_under_delimiter_pressure() {
    let mut runtime = bridge_backed_runtime_with_support(intent_support_profile());

    let left = runtime
        .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
            "intent|receipt",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            test_intent_input([("entity", "task|1"), ("title", "alpha:beta")]),
        ))
        .expect("left intent should execute");
    let right = runtime
        .execute_intent(WorthQueryIntentDeclaration::strategy_commit(
            "intent",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            test_intent_input([("entity", "receipt|task"), ("title", "alpha:beta")]),
        ))
        .expect("right intent should execute");

    assert_eq!(
        left.receipt_identity().scope(),
        WorthQueryEvidenceScope::AuthoritativeIntentReceipt
    );
    assert_eq!(
        left.execution_provenance()
            .execution_provenance_chain_identity()
            .scope(),
        WorthQueryEvidenceScope::IntentExecutionProvenanceChain
    );
    assert_ne!(
        left.receipt_identity(),
        right.receipt_identity(),
        "intent receipt identity must not collapse delimiter-shaped field boundaries"
    );
    assert_ne!(
        left.execution_provenance()
            .execution_provenance_chain_identity(),
        right
            .execution_provenance()
            .execution_provenance_chain_identity(),
        "provenance chain identity must not collapse delimiter-shaped field boundaries"
    );
    assert_eq!(
        left.receipt_identity(),
        &compose_authoritative_intent_receipt_identity(&left),
        "runtime authoritative intent receipt identity must match independent typed composition"
    );
    assert_eq!(
        left.execution_provenance()
            .execution_provenance_chain_identity(),
        &compose_intent_execution_provenance_chain_identity(&left),
        "runtime provenance chain identity must match independent typed composition"
    );

    let inspection = runtime
        .inspect_intent_receipt(&left)
        .expect("authoritative receipt inspection should inspect");
    let right_inspection = runtime
        .inspect_intent_receipt(&right)
        .expect("right authoritative receipt inspection should inspect");
    assert_eq!(
        inspection.delivery_counters().counter_identity().scope(),
        WorthQueryEvidenceScope::IntentInspectionDeliveryCounters
    );
    assert_eq!(
        inspection.inspection_identity().scope(),
        WorthQueryEvidenceScope::IntentReceiptInspection
    );
    assert_eq!(
        inspection.inspection_identity(),
        &compose_authoritative_intent_receipt_inspection_identity(&inspection),
        "authoritative inspection identity must match typed composition"
    );
    assert_eq!(
        inspection.receipt_identity(),
        left.receipt_identity(),
        "authoritative inspection should retain the typed receipt identity"
    );
    assert_ne!(
        inspection.inspection_identity(),
        right_inspection.inspection_identity(),
        "authoritative inspection identity must not collapse delimiter-shaped receipt fields"
    );
}

#[test]
fn effect_triggered_intent_receipt_identity_keeps_nested_receipts_typed() {
    let mut runtime = bridge_backed_runtime_with_support(intent_support_profile());
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
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
        .write(test_update_string_aspect_command(
            crate::memory_workspace::admit_authored_entity_label("task|identity"),
            "title.value",
            "title:identity",
        ))
        .expect("write should route pending effect intent");
    let receipt = runtime
        .execute_next_effect_write_intent(&effect, "1.0", "effect.intent.input.v1")
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
        .inspect_effect_intent_receipt(&receipt)
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
    let mut left_runtime = bridge_backed_runtime_with_support(intent_support_profile());
    let left_effect = declare_identity_effect(
        &mut left_runtime,
        "tasks.effect-left|identity",
        "effects.identity|receipt",
    );
    left_runtime
        .write(test_update_string_aspect_command(
            crate::memory_workspace::admit_authored_entity_label("task|identity"),
            "title.value",
            "title:left",
        ))
        .expect("left write should route pending effect intent");
    let left = left_runtime
        .execute_next_effect_write_intent(&left_effect, "1.0", "effect.intent.input.v1")
        .expect("left pending effect intent should execute");

    let mut right_runtime = bridge_backed_runtime_with_support(intent_support_profile());
    let right_effect = declare_identity_effect(
        &mut right_runtime,
        "tasks.effect-right|identity",
        "effects.identity",
    );
    right_runtime
        .write(test_update_string_aspect_command(
            crate::memory_workspace::admit_authored_entity_label("identity|task"),
            "title.value",
            "title:right",
        ))
        .expect("right write should route pending effect intent");
    let right = right_runtime
        .execute_next_effect_write_intent(&right_effect, "1.0", "effect.intent.input.v1")
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
        .build()
        .expect("intent-capable runtime should build");

    let mut preview = runtime
        .preview_with_options(
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
        .inspect_preview_intent_receipt(&receipt)
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
    let left = preview_receipt_with_basis(
        ["alpha", "beta|gamma"],
        "preview|intent",
        test_intent_input([("entity", "preview|task"), ("title", "left")]),
    );
    let right = preview_receipt_with_basis(
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
