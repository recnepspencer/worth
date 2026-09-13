use super::*;

#[test]
fn effect_triggered_idempotent_intent_noop_consumes_pending_work_without_feedback() {
    let routed = std::rc::Rc::new(std::cell::Cell::new(0));
    let mut runtime = complete_backend_from_parts_builder()
        .signal_sink(CountingSignalSink {
            routed: routed.clone(),
        })
        .intent_authority(NoopIntentAuthority)
        .support_profile(intent_support_profile())
        .build_backend_from_parts()
        .build()
        .expect("intent-capable runtime should build");
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            "tasks.effect-noop-intent",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    let effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::write_intent(
            "effects.noop-reconcile-title",
            WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title.value"])),
            "strategy.intent.reconcile",
        ))
        .expect("write-intent effect should declare");

    let write = runtime
        .write(test_update_string_aspect_command(
            crate::memory_workspace::admit_authored_entity_label("task-1"),
            "title.value",
            "already reconciled title",
        ))
        .expect("write should route pending effect intent");
    assert_eq!(write.pending_write_intent_count(), 1);
    assert_eq!(
        routed.get(),
        1,
        "the original authoritative write routes once"
    );

    let effect_intent = runtime
        .execute_next_effect_write_intent(&effect, "1.0", "effect.intent.input.v1")
        .expect("pending effect intent should execute as an idempotent no-op");

    assert_eq!(
        effect_intent.intent_receipt().execution_kind(),
        WorthQueryIntentExecutionKind::IdempotentNoop
    );
    assert!(effect_intent.intent_receipt().is_idempotent_noop());
    assert_eq!(
        effect_intent.intent_receipt().source_lane(),
        WorthQueryIntentSourceLane::EffectTriggered
    );
    assert_eq!(
        effect_intent.phase_evidence().idempotence(),
        WorthQueryEffectIdempotence::PendingIntentReceiptIdentity
    );
    assert_eq!(
        runtime
            .inspect_effect(&effect)
            .expect("pending no-op should be consumed")
            .pending_write_intent_count(),
        0
    );
    assert_eq!(
        routed.get(),
        1,
        "idempotent effect intent must not emit a second signal invalidation"
    );

    let graph = runtime
        .inspect_effect_feedback_receipt(&effect_intent)
        .expect("idempotent effect intent should expose coalesced graph");
    assert_eq!(
        graph.phase_nodes(),
        &[
            WorthQueryFeedbackPhaseNode::TruthRead,
            WorthQueryFeedbackPhaseNode::Derive,
            WorthQueryFeedbackPhaseNode::PendingWriteIntent,
            WorthQueryFeedbackPhaseNode::Commit,
        ]
    );
    assert_eq!(
        graph.termination(),
        WorthQueryFeedbackTermination::CoalescedNoMutation
    );
    assert_eq!(graph.resubscribed_live_view_count(), 0);
}
