use super::super::support::*;

#[test]
fn effect_expression_suppression_and_failure_are_typed_and_counted() {
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
    let suppressed_effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(
            WorthQueryEffectDeclaration::deliver(
                "ui.suppressed",
                WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title"])),
                "ui.suppressed",
            )
            .with_condition(WorthQueryEffectCondition::expression(
                "expr.needs-validation",
                test_aspect_touches(["validation.state"]),
                test_aspect_touches(["ui.badge"]),
            )),
        )
        .expect("suppressed effect should declare");
    let failing_effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(
            WorthQueryEffectDeclaration::deliver(
                "ui.failing",
                WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title"])),
                "ui.failing",
            )
            .with_condition(WorthQueryEffectCondition::failing_expression(
                "expr.fail.validation",
                test_aspect_touches(["title"]),
                test_aspect_touches(["ui.badge"]),
            )),
        )
        .expect("failing effect should declare");

    let write = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    ("title.value", test_string_aspect_value("Conditional task")),
                ],
            ),
        )
        .expect("write should route effects");
    let suppressed_evidence = runtime
        .inspect_effect(request_execution, &suppressed_effect)
        .expect("suppressed terminal artifact should inspect before drain");
    let suppressed = runtime
        .drain_effect_deliveries(&suppressed_effect)
        .expect("suppressed effect should drain");
    let failed = runtime
        .drain_effect_deliveries(&failing_effect)
        .expect("failing effect should drain");

    assert_eq!(write.considered_effect_count(), 2);
    assert_eq!(write.delivered_effect_count(), 0);
    assert_eq!(write.suppressed_effect_count(), 1);
    assert_eq!(write.effect_expression_failure_count(), 1);
    assert_eq!(
        suppressed[0].family(),
        &WorthQueryEffectDeliveryFamily::Suppressed
    );
    assert_eq!(suppressed_evidence.pending_delivery_count(), 1);
    assert_eq!(suppressed_evidence.pending_delivered_count(), 0);
    assert_eq!(suppressed_evidence.pending_suppressed_count(), 1);
    assert_eq!(suppressed_evidence.pending_expression_failure_count(), 0);
    assert_eq!(
        suppressed_evidence.latest_delivery_family(),
        Some(&WorthQueryEffectDeliveryFamily::Suppressed)
    );
    assert!(suppressed_evidence.latest_phase_digest().is_some());
    assert!(suppressed[0]
        .reason()
        .expect("suppression reason should exist")
        .contains("inputs were not changed"));
    assert_eq!(
        failed[0].family(),
        &WorthQueryEffectDeliveryFamily::ExpressionFailed
    );
    assert!(failed[0]
        .reason()
        .expect("failure reason should exist")
        .contains("deterministic failure"));
}

#[test]
fn meaningful_change_suppression_counts_semantic_delta_suppression() {
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
            WorthQueryEffectDeclaration::deliver(
                "ui.meaningful-title",
                WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title"])),
                "ui.badges",
            )
            .with_meaningful_change_suppression(),
        )
        .expect("meaningful effect should declare");

    let inserted = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    ("title.value", test_string_aspect_value("Meaningful task")),
                ],
            ),
        )
        .expect("insert should deliver because whole-row delta is meaningful");
    assert_eq!(inserted.delivered_effect_count(), 1);
    assert_eq!(inserted.meaningful_effect_suppression_count(), 0);
    assert_eq!(
        runtime
            .drain_effect_deliveries(&effect)
            .expect("insert delivery should drain")
            .len(),
        1
    );

    let churn = runtime
        .write(
            request_execution,
            test_update_string_aspect_command(
                inserted.deltas()[0].entity_identity.clone(),
                "identity.id",
                "semantic-churn",
            ),
        )
        .expect("irrelevant aspect update should be suppressed as churn");
    let evidence = runtime
        .inspect_effect(request_execution, &effect)
        .expect("meaningful effect should inspect");
    let suppressed = runtime
        .drain_effect_deliveries(&effect)
        .expect("suppressed effect should drain");

    assert_eq!(churn.considered_effect_count(), 1);
    assert_eq!(churn.delivered_effect_count(), 0);
    assert_eq!(churn.suppressed_effect_count(), 1);
    assert_eq!(churn.meaningful_effect_suppression_count(), 1);
    assert_eq!(
        evidence.suppression_policy(),
        WorthQueryEffectSuppressionPolicy::MeaningfulSemanticDelta
    );
    assert_eq!(evidence.counters().meaningful_suppressions(), 1);
    assert_eq!(evidence.pending_delivered_count(), 0);
    assert_eq!(evidence.pending_suppressed_count(), 1);
    assert_eq!(evidence.pending_expression_failure_count(), 0);
    assert_eq!(
        evidence.latest_delivery_family(),
        Some(&WorthQueryEffectDeliveryFamily::Suppressed)
    );
    assert_eq!(suppressed.len(), 1);
    assert_eq!(
        suppressed[0].family(),
        &WorthQueryEffectDeliveryFamily::Suppressed
    );
    assert_eq!(
        suppressed[0].suppression_policy(),
        WorthQueryEffectSuppressionPolicy::MeaningfulSemanticDelta
    );
    assert!(suppressed[0]
        .reason()
        .expect("meaningful suppression should explain itself")
        .contains("meaningful semantic delta suppression"));
}
