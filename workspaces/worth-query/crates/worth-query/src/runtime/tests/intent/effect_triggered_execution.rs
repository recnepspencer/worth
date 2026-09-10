use super::*;

#[test]
fn effect_triggered_pending_write_intent_executes_through_intent_authority_once() {
    let mut runtime = complete_backend_from_parts_builder()
        .intent_authority(TestIntentAuthority)
        .support_profile(intent_support_profile())
        .build_backend_from_parts()
        .build()
        .expect("intent-capable runtime should build");
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            "tasks.effect-intent",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    let effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::write_intent(
            "effects.reconcile-title",
            WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title.value"])),
            "strategy.intent.reconcile",
        ))
        .expect("write-intent effect should declare");

    let write = runtime
        .write(test_update_string_aspect_command(
            crate::memory_workspace::admit_authored_entity_label("task-1"),
            "title.value",
            "title from write",
        ))
        .expect("write should route pending effect intent");
    assert_eq!(write.pending_write_intent_count(), 1);
    assert_eq!(
        runtime
            .inspect_effect(&effect)
            .expect("effect should inspect")
            .pending_write_intent_count(),
        1
    );

    let effect_intent = runtime
        .execute_next_effect_write_intent(&effect, "1.0", "effect.intent.input.v1")
        .expect("pending effect intent should execute through intent authority");

    assert_eq!(effect_intent.effect_name(), "effects.reconcile-title");
    assert_eq!(
        effect_intent.trigger_commit_evidence_identity().scope(),
        WorthQueryEvidenceScope::EffectTriggerCommitIdentity
    );
    assert_ne!(
        effect_intent.trigger_commit_evidence_identity(),
        write.commit_evidence_identity(),
        "effect trigger identity must wrap, not collapse to, the write receipt commit identity"
    );
    assert_eq!(
        effect_intent.pending_intent_target(),
        "strategy.intent.reconcile"
    );
    assert_eq!(
        effect_intent.source_lane(),
        WorthQueryIntentSourceLane::EffectTriggered
    );
    assert_eq!(
        effect_intent.target_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert_eq!(
        effect_intent.phase_evidence().loop_prevention().as_str(),
        "pending-intent-execution-deferred"
    );
    assert_eq!(
        effect_intent.intent_receipt().source_lane(),
        WorthQueryIntentSourceLane::EffectTriggered
    );
    assert_eq!(
        effect_intent.intent_receipt().strategy_identity(),
        "strategy.intent.reconcile"
    );
    assert!(!effect_intent.receipt_digest().is_empty());

    let inspected = runtime
        .inspect_effect(&effect)
        .expect("effect should retain only follow-up pending intent");
    assert_eq!(
        inspected.pending_write_intent_count(),
        1,
        "executing one pending intent must not recursively auto-execute feedback"
    );

    let effect_intent_inspection = runtime
        .inspect_effect_intent_receipt(&effect_intent)
        .expect("effect intent receipt should inspect");
    assert_eq!(
        effect_intent_inspection.effect_name(),
        "effects.reconcile-title"
    );
    assert_eq!(
        effect_intent_inspection
            .trigger_commit_evidence_identity()
            .scope(),
        WorthQueryEvidenceScope::EffectTriggerCommitIdentity
    );
    assert_ne!(
        effect_intent_inspection.trigger_commit_evidence_identity(),
        write.commit_evidence_identity(),
        "effect intent inspection must preserve the typed trigger wrapper identity"
    );
    assert_eq!(
        effect_intent_inspection.effect_policy(),
        WorthQueryEffectPolicy::SandboxedWriteIntent
    );
    assert_eq!(
        effect_intent_inspection.feedback_graph().termination(),
        WorthQueryFeedbackTermination::CommittedResubscribe
    );
    assert!(!effect_intent_inspection.phase_digest().is_empty());
    assert!(!effect_intent_inspection.inspection_digest().is_empty());

    match runtime
        .inspect(&effect_intent)
        .expect("effect-intent receipt target should inspect")
    {
        WorthQueryInspection::EffectIntentReceipt(inspection) => {
            assert_eq!(
                inspection.feedback_graph().phase_nodes(),
                &[
                    WorthQueryFeedbackPhaseNode::TruthRead,
                    WorthQueryFeedbackPhaseNode::Derive,
                    WorthQueryFeedbackPhaseNode::PendingWriteIntent,
                    WorthQueryFeedbackPhaseNode::Commit,
                    WorthQueryFeedbackPhaseNode::BridgeRoute,
                    WorthQueryFeedbackPhaseNode::Resubscribe,
                ]
            );
        }
        other => panic!("expected effect-intent receipt inspection, got {other:?}"),
    }

    let graph = runtime
        .inspect_effect_feedback_receipt(&effect_intent)
        .expect("effect feedback receipt should expose phase graph");
    assert_eq!(
        graph.phase_nodes(),
        &[
            WorthQueryFeedbackPhaseNode::TruthRead,
            WorthQueryFeedbackPhaseNode::Derive,
            WorthQueryFeedbackPhaseNode::PendingWriteIntent,
            WorthQueryFeedbackPhaseNode::Commit,
            WorthQueryFeedbackPhaseNode::BridgeRoute,
            WorthQueryFeedbackPhaseNode::Resubscribe,
        ]
    );
    assert_eq!(
        graph.termination(),
        WorthQueryFeedbackTermination::CommittedResubscribe
    );
    assert_eq!(graph.resubscribed_live_view_count(), 1);
    assert!(!graph.graph_digest().is_empty());
}
