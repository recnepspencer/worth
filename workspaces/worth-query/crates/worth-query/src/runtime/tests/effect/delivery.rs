use super::super::support::*;

#[test]
fn effect_delivery_routes_from_live_trigger_with_expression_metadata() {
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
                "ui.title-badges",
                WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title"])),
                "ui.badges",
            )
            .with_condition(WorthQueryEffectCondition::expression(
                "expr.title.badge",
                test_aspect_touches(["title"]),
                test_aspect_touches(["ui.badge"]),
            )),
        )
        .expect("effect should declare");

    let write = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    ("title.value", test_string_aspect_value("Effect task")),
                ],
            ),
        )
        .expect("write should route effect");
    let evidence = runtime
        .inspect_effect(request_execution, &effect)
        .expect("effect should inspect before drain");
    let deliveries = runtime
        .drain_effect_deliveries(&effect)
        .expect("effect deliveries should drain");

    assert_eq!(write.considered_effect_count(), 1);
    assert_eq!(write.delivered_effect_count(), 1);
    assert_eq!(write.suppressed_effect_count(), 0);
    assert_eq!(write.effect_expression_failure_count(), 0);
    assert_eq!(evidence.name(), "ui.title-badges");
    assert_eq!(evidence.trigger_source(), "tasks.table");
    assert_eq!(
        evidence.trigger_source_kind(),
        WorthQueryEffectTriggerSourceKind::LiveView
    );
    assert_eq!(evidence.condition_descriptor(), "expr.title.badge");
    assert_eq!(
        evidence.condition_input_touches(),
        test_aspect_touches(["title"]).as_slice()
    );
    assert_eq!(
        evidence.condition_output_touches(),
        test_aspect_touches(["ui.badge"]).as_slice()
    );
    assert_eq!(evidence.pending_delivery_count(), 1);
    assert_eq!(evidence.pending_delivered_count(), 1);
    assert_eq!(evidence.pending_suppressed_count(), 0);
    assert_eq!(evidence.pending_expression_failure_count(), 0);
    assert_eq!(
        evidence.latest_delivery_family(),
        Some(&WorthQueryEffectDeliveryFamily::Delivered)
    );
    assert!(!evidence.trigger_digest().is_empty());
    assert!(!evidence.condition_digest().is_empty());
    assert!(!evidence.declaration_digest().is_empty());
    assert!(!evidence.pending_delivery_digest().is_empty());
    assert!(evidence.latest_phase_digest().is_some());
    assert!(!evidence.inspection_digest().is_empty());
    let feedback_graph = evidence
        .feedback_graph()
        .expect("effect inspection should carry feedback graph");
    assert_eq!(
        feedback_graph.phase_nodes(),
        &[
            WorthQueryFeedbackPhaseNode::TruthRead,
            WorthQueryFeedbackPhaseNode::Derive,
            WorthQueryFeedbackPhaseNode::EffectDelivery,
        ]
    );
    assert_eq!(
        feedback_graph.termination(),
        WorthQueryFeedbackTermination::Delivered
    );
    assert_eq!(deliveries.len(), 1);
    assert_eq!(
        deliveries[0].family(),
        &WorthQueryEffectDeliveryFamily::Delivered
    );
    assert_eq!(deliveries[0].target(), "ui.badges");
    assert_eq!(
        deliveries[0].authority_lane(),
        WorthQueryAuthorityLane::EffectDeliveryState
    );
    assert_eq!(
        deliveries[0].aspect_touches(),
        test_aspect_touches(["title"]).as_slice()
    );
    assert_eq!(
        deliveries[0].payload().condition(),
        Some("expr.title.badge")
    );

    let graph = runtime
        .inspect_feedback_path(&effect)
        .expect("delivered effect should expose feedback graph");
    assert_eq!(graph.effect_name(), "ui.title-badges");
    assert_eq!(
        graph.phase_nodes(),
        &[
            WorthQueryFeedbackPhaseNode::TruthRead,
            WorthQueryFeedbackPhaseNode::Derive,
            WorthQueryFeedbackPhaseNode::EffectDelivery,
        ]
    );
    assert_eq!(
        graph.termination(),
        WorthQueryFeedbackTermination::Delivered
    );
    assert_eq!(
        graph.effect_policy(),
        Some(WorthQueryEffectPolicy::AuthoritativeAllowed)
    );
    assert!(!graph.graph_digest().is_empty());
    assert!(!graph.inspection_digest().is_empty());
}

#[test]
fn effect_delivery_routes_from_computed_trigger_after_computed_patch() {
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
    let titles = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new("computed.titles.effect", test_aspect_touches(["title"]))
                .depends_on_live(&live)
                .produces(test_aspect_touches(["title.summary"])),
            TitleListMaintainer,
        )
        .expect("computed should declare");
    let effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::deliver(
            "ui.summary-badges",
            WorthQueryEffectTrigger::computed_view(&titles, test_aspect_touches(["title.summary"])),
            "ui.summary",
        ))
        .expect("computed-triggered effect should declare");

    let write = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    (
                        "title.value",
                        test_string_aspect_value("Computed effect task"),
                    ),
                ],
            ),
        )
        .expect("write should route computed effect");
    let deliveries = runtime
        .drain_effect_deliveries(&effect)
        .expect("effect deliveries should drain");

    assert_eq!(write.considered_computed_view_count(), 1);
    assert_eq!(write.considered_effect_count(), 1);
    assert_eq!(write.delivered_effect_count(), 1);
    assert_eq!(deliveries.len(), 1);
    assert_eq!(
        deliveries[0].trigger_source_kind(),
        WorthQueryEffectTriggerSourceKind::ComputedView
    );
    assert_eq!(deliveries[0].trigger_source(), "computed.titles.effect");
    assert_eq!(
        deliveries[0].aspect_touches(),
        test_aspect_touches(["title.summary"]).as_slice()
    );
    let value_path =
        retained_test_field_path("value").expect("test retained value path should parse");
    let materialized = runtime
        .read_derived_result(request_execution, &titles)
        .expect("computed title materialization should execute");
    let retained_value = materialized.retained_rows()[0]
        .scalar_value_at(&value_path)
        .expect("computed title row should retain value");
    assert_eq!(
        retained_value,
        &crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
            write.deltas()[0]
                .entity_identity
                .terminal_projection_for_reporting()
                .to_string()
        )
    );
}

#[test]
fn computed_effect_does_not_replay_stale_undrained_computed_patch() {
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
    let titles = runtime
        .declare_maintained_derived_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            WorthQueryDerivedView::new(
                "computed.titles.stale-effect",
                test_aspect_touches(["title"]),
            )
            .depends_on_live(&live)
            .produces(test_aspect_touches(["title.summary"])),
            TitleListMaintainer,
        )
        .expect("computed should declare");
    let effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::deliver(
            "ui.stale-summary-badges",
            WorthQueryEffectTrigger::computed_view(&titles, test_aspect_touches(["title.summary"])),
            "ui.summary",
        ))
        .expect("computed-triggered effect should declare");

    let first_write = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    ("title.value", test_string_aspect_value("First effect task")),
                ],
            ),
        )
        .expect("first write should route computed effect");
    let first_deliveries = runtime
        .drain_effect_deliveries(&effect)
        .expect("first effect deliveries should drain");
    assert_eq!(first_deliveries.len(), 1);

    let unrelated = runtime
        .write(
            request_execution,
            test_update_string_aspect_command(
                first_write.deltas()[0].entity_identity.clone(),
                "identity.id",
                "irrelevant",
            ),
        )
        .expect("irrelevant write should not replay stale computed patch");
    let stale_deliveries = runtime
        .drain_effect_deliveries(&effect)
        .expect("stale effect deliveries should drain");

    assert_eq!(unrelated.considered_computed_view_count(), 1);
    assert!(unrelated
        .terminal_affected_derived_view_ids_projection()
        .is_empty());
    assert_eq!(unrelated.considered_effect_count(), 0);
    assert!(stale_deliveries.is_empty());
}
