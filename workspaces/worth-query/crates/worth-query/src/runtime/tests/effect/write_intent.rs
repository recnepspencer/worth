use super::super::support::*;

#[test]
fn write_intent_effect_lowers_to_pending_intent_with_phase_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.table",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    let effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(
            WorthQueryEffectDeclaration::write_intent(
                "intent.reconcile-title",
                WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title"])),
                "reconcile-title-slug",
            )
            .with_condition(WorthQueryEffectCondition::expression(
                "expr.title.needs-slug",
                test_aspect_touches(["title"]),
                test_aspect_touches(["intent.slug"]),
            )),
        )
        .expect("pending write-intent effect should declare");

    let write = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    ("title.value", test_string_aspect_value("Intent task")),
                ],
            ),
        )
        .expect("write should route pending intent effect");
    let evidence = runtime
        .inspect_effect(request_execution, &effect)
        .expect("pending intent effect should inspect");
    let deliveries = runtime
        .drain_effect_deliveries(&effect)
        .expect("pending intent work should drain through effect queue");

    assert_eq!(
        effect.authority_lane(),
        WorthQueryAuthorityLane::PendingWriteIntent
    );
    assert_eq!(write.considered_effect_count(), 1);
    assert_eq!(write.delivered_effect_count(), 0);
    assert_eq!(write.pending_write_intent_count(), 1);
    assert_eq!(evidence.pending_delivery_count(), 0);
    assert_eq!(evidence.pending_write_intent_count(), 1);
    assert_eq!(evidence.pending_delivered_count(), 0);
    assert_eq!(evidence.pending_suppressed_count(), 0);
    assert_eq!(evidence.pending_expression_failure_count(), 0);
    assert_eq!(
        evidence.latest_delivery_family(),
        Some(&WorthQueryEffectDeliveryFamily::PendingWriteIntent)
    );
    assert!(!evidence.trigger_digest().is_empty());
    assert!(!evidence.condition_digest().is_empty());
    assert!(!evidence.declaration_digest().is_empty());
    assert!(!evidence.pending_delivery_digest().is_empty());
    assert!(evidence.latest_phase_digest().is_some());
    assert!(!evidence.inspection_digest().is_empty());
    assert_eq!(
        evidence
            .latest_phase_evidence()
            .expect("phase evidence should exist")
            .phases(),
        &[
            WorthQueryEffectPhase::TruthRead,
            WorthQueryEffectPhase::Derive,
            WorthQueryEffectPhase::PendingWriteIntent,
        ]
    );
    assert_eq!(deliveries.len(), 1);
    assert_eq!(
        deliveries[0].family(),
        &WorthQueryEffectDeliveryFamily::PendingWriteIntent
    );
    assert_eq!(
        deliveries[0].authority_lane(),
        WorthQueryAuthorityLane::PendingWriteIntent
    );
    assert_eq!(deliveries[0].target(), "reconcile-title-slug");
    assert_eq!(
        deliveries[0].phase_evidence().loop_prevention(),
        WorthQueryEffectLoopPrevention::PendingIntentExecutionDeferred
    );
    assert_eq!(
        deliveries[0].phase_evidence().loop_prevention().as_str(),
        "pending-intent-execution-deferred"
    );
    assert_eq!(
        deliveries[0].phase_evidence().idempotence(),
        WorthQueryEffectIdempotence::PendingIntentReceiptIdentity
    );
    assert!(deliveries[0]
        .reason()
        .expect("pending intent explanation should exist")
        .contains("pending write intent"));
}

#[test]
fn write_intent_effect_rejects_authoritative_truth_target() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.table",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    let declaration = WorthQueryEffectDeclaration::write_intent(
        "intent.truth-smuggle",
        WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title"])),
        "direct-truth-write",
    )
    .with_effect_policy(WorthQueryEffectPolicy::AuthoritativeAllowed)
    .with_target_lane(WorthQueryAuthorityLane::AuthoritativeTruth);

    let error = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(declaration)
        .expect_err("write intent cannot target truth directly");

    match error {
        WorthQueryRuntimeError::EffectDeclaration { stage, message, .. } => {
            assert_eq!(stage, "write-intent-admission");
            assert!(message.contains("pending write intent authority"));
        }
        other => panic!("expected write intent admission denial, got {other:?}"),
    }
}
