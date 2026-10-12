use super::support::*;

#[test]
fn runtime_declares_live_view_and_routes_minimal_write_patches() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.table",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");

    let insert = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    ("title.value", test_string_aspect_value("Buy milk")),
                ],
            ),
        )
        .expect("insert should execute through runtime facade");
    let task_id = insert.deltas()[0].entity_identity.clone();
    let insert_patches = runtime.drain_patches(&view);

    assert_eq!(insert.deltas().len(), 1);
    assert_eq!(
        insert.deltas()[0].admitted_touched_aspects(),
        test_aspect_touches(["identity", "title"]).as_slice()
    );
    assert_eq!(
        insert.terminal_affected_live_view_ids_projection(),
        &["tasks.table".to_string()]
    );
    assert!(insert_patches.live_patches.is_empty());
    assert_eq!(insert_patches.query_delivery_batches.len(), 1);
    assert_eq!(
        insert_patches.query_delivery_batches[0].patch_group_kind(),
        QueryPatchGroupKind::CollectionMembershipPatchGroup
    );
    assert_eq!(insert_patches.query_delivery_batches[0].sequence(), 1);

    let update = runtime
        .write(
            request_execution,
            test_update_string_aspect_command(task_id, "title.value", "Buy oat milk"),
        )
        .expect("update should execute through runtime facade");
    let update_patches = runtime.drain_patches(&view);

    assert_eq!(
        update.deltas()[0].admitted_touched_aspects(),
        test_aspect_touches(["title.value"]).as_slice()
    );
    assert!(update_patches.live_patches.is_empty());
    assert_eq!(update_patches.query_delivery_batches.len(), 1);
    assert_eq!(
        update_patches.query_delivery_batches[0].patch_group_kind(),
        QueryPatchGroupKind::DetailFieldPatchGroup
    );
    assert_eq!(update_patches.query_delivery_batches[0].sequence(), 2);

    let irrelevant = runtime
        .write(
            request_execution,
            test_update_string_aspect_command(
                update.deltas()[0].entity_identity.clone(),
                "description.value",
                "ignored by task table",
            ),
        )
        .expect("irrelevant update should execute");
    let irrelevant_patches = runtime.drain_patches(&view);
    assert!(irrelevant
        .terminal_affected_live_view_ids_projection()
        .is_empty());
    assert!(irrelevant_patches.query_delivery_batches.is_empty());
}

#[test]
fn runtime_grouped_live_view_uses_backend_baseline_and_delivers_grouped_membership_patch() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_grouped_task_runtime(request_execution);
    let table: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.seed-table",
            grouped_task_table_live_request(),
            grouped_task_schema(),
        )
        .expect("table live view should declare before seed write");
    let seed = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    ("title.value", test_string_aspect_value("Seed task")),
                    ("status.value", test_string_aspect_value("todo")),
                ],
            ),
        )
        .expect("seed insert should write through table declaration");
    let task_id = seed.deltas()[0].entity_identity.clone();
    let _ = runtime.drain_patches(&table);
    let grouped: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.grouped",
            grouped_task_live_request(),
            grouped_task_schema(),
        )
        .expect("grouped live view should declare with backend-owned baseline");

    let receipt = runtime
        .write(
            request_execution,
            test_update_string_aspect_command(task_id, "status.value", "done"),
        )
        .expect("grouping aspect update should write");
    let patches = runtime.drain_patches(&grouped);

    assert!(receipt
        .terminal_affected_live_view_ids_projection()
        .contains(&"tasks.grouped".to_string()));
    assert_eq!(patches.query_delivery_batches.len(), 1);
    assert_eq!(
        patches.query_delivery_batches[0].patch_group_kind(),
        QueryPatchGroupKind::GroupedMembershipPatchGroup
    );
    assert_eq!(
        grouped.subscription_installation().subscription_family(),
        "grouped_collection_membership"
    );
}

#[test]
fn unified_inspect_routes_live_effect_and_write_receipt_targets() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.inspect-target",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    let effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::deliver(
            "ui.inspect-target",
            WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title"])),
            "ui.inspect",
        ))
        .expect("effect should declare");
    let receipt = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    ("title.value", test_string_aspect_value("Inspect target")),
                ],
            ),
        )
        .expect("write should execute");

    let live_inspection = runtime
        .inspect(request_execution, &live)
        .expect("live target should inspect");
    let effect_inspection = runtime
        .inspect(request_execution, &effect)
        .expect("effect target should inspect");
    let receipt_inspection = runtime
        .inspect(request_execution, &receipt)
        .expect("receipt target should inspect");

    match live_inspection {
        WorthQueryInspection::LiveView(inspection) => {
            assert_eq!(inspection.view_name(), "tasks.inspect-target");
        }
        other => panic!("expected live inspection, got {other:?}"),
    }

    match effect_inspection {
        WorthQueryInspection::Effect(inspection) => {
            assert_eq!(inspection.name(), "ui.inspect-target");
            assert!(inspection.feedback_graph().is_some());
        }
        other => panic!("expected effect inspection, got {other:?}"),
    }

    match receipt_inspection {
        WorthQueryInspection::WriteReceipt(inspection) => {
            assert_eq!(inspection.mutation_family(), "insert");
            assert_eq!(
                inspection.authority_lane(),
                WorthQueryAuthorityLane::AuthoritativeTruth
            );
            assert_eq!(
                inspection.basis_lane(),
                WorthQueryAuthorityLane::AuthoritativeTruth
            );
            assert_eq!(inspection.declared_collection(), Some("Task"));
            assert_eq!(inspection.commit_identity(), receipt.commit_identity());
            assert!(!inspection.inspection_digest().is_empty());
        }
        other => panic!("expected write receipt inspection, got {other:?}"),
    }
}

#[test]
fn live_view_inspection_reconstructs_subscription_proof_chain() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.table",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");

    let installation = view.subscription_installation();
    let inspection = runtime
        .inspect_live_view_explanation(&view)
        .expect("live view explanation should inspect retained installation");

    assert_eq!(inspection.view_name(), "tasks.table");
    assert_eq!(
        inspection.authority_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert_eq!(
        inspection.query_projection().label().as_str(),
        installation.query_projection().label().as_str()
    );
    assert_eq!(
        inspection.view_shape_projection().label().as_str(),
        installation.view_shape_projection().label().as_str()
    );
    assert_eq!(
        inspection.subscription_family(),
        installation.subscription_family()
    );
    assert_eq!(
        inspection.subscription_family_projection().label().as_str(),
        installation
            .subscription_family_projection()
            .label()
            .as_str()
    );
    assert_eq!(
        inspection
            .subscription_declaration_projection()
            .label()
            .as_str(),
        installation
            .subscription_declaration_projection()
            .label()
            .as_str()
    );
    assert_eq!(
        inspection.bridge_declaration_projection().label().as_str(),
        installation
            .bridge_declaration_projection()
            .label()
            .as_str()
    );
    assert_eq!(
        inspection.admission_projection().label().as_str(),
        installation.admission_projection().label().as_str()
    );
    assert_eq!(
        inspection.activation_projection().label().as_str(),
        installation.activation_projection().label().as_str()
    );
    assert_eq!(
        inspection.basis_binding_projection().label().as_str(),
        installation.basis_binding_projection().label().as_str()
    );
    assert_eq!(
        inspection.signal_strategy_projection().label().as_str(),
        installation.signal_strategy_projection().label().as_str()
    );
    assert_eq!(
        inspection.active_lane_projection().label().as_str(),
        installation.active_lane_projection().label().as_str()
    );
    assert_eq!(
        inspection.consumer_attachment_projection().label().as_str(),
        installation
            .consumer_attachment_projection()
            .label()
            .as_str()
    );
    assert_eq!(
        inspection.consumer_projection().label().as_str(),
        installation.consumer_projection().label().as_str()
    );
    assert_eq!(
        inspection.delivery_cursor_projection().label().as_str(),
        installation.delivery_cursor_projection().label().as_str()
    );
    assert_eq!(
        inspection.subscription_budget_policy(),
        installation.subscription_budget_policy()
    );
    assert_eq!(
        inspection.active_lifecycle_budget_policy(),
        installation.active_lifecycle_budget_policy()
    );
    assert_eq!(
        inspection.consumer_attachment_budget_policy(),
        installation.consumer_attachment_budget_policy()
    );
    assert_eq!(
        inspection.runtime_budget_projection().label().as_str(),
        installation.runtime_budget_projection().label().as_str()
    );
    assert_eq!(
        inspection.support_projection().label().as_str(),
        installation.support_projection().label().as_str()
    );
    assert_eq!(
        inspection.installation_projection().label().as_str(),
        installation.installation_projection().label().as_str()
    );
    assert!(!inspection
        .inspection_projection()
        .label()
        .as_str()
        .is_empty());

    let counters = inspection.counters();
    assert_eq!(
        counters.declaration_counter_for_reporting(),
        installation.counters().counter_projection().label()
    );
    assert_eq!(
        counters.active_lane_counter_for_reporting(),
        installation
            .active_lane_counters()
            .counter_projection()
            .label()
    );
    assert_eq!(
        counters.consumer_attachment_counter_for_reporting(),
        installation
            .consumer_attachment_counters()
            .counter_projection()
            .label()
    );
    assert_eq!(counters.family_selection_count(), 1);
    assert_eq!(counters.declaration_count(), 1);
    assert_eq!(counters.bridge_lowering_count(), 1);
    assert_eq!(counters.admission_count(), 1);
    assert_eq!(counters.activation_input_count(), 1);
    assert_eq!(counters.active_lane_admission_count(), 1);
    assert_eq!(counters.active_lane_creation_count(), 1);
    assert_eq!(counters.active_lane_handle_issue_count(), 1);
    assert_eq!(counters.consumer_attachment_count(), 1);
    assert_eq!(counters.consumer_attachment_denial_count(), 0);
}
