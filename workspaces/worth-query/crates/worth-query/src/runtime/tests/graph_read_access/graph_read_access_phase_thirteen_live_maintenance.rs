use crate::runtime::tests::graph_read_access::support;

use crate::runtime::{
    plan_admitted_graph_read_access_for_family, WorthQueryGraphReadAccessRequirementKind,
    WorthQueryLiveGraphReadAccessPosture, WorthQueryLiveGraphReadMaintenanceBudget,
    WorthQueryUnrefinedLiveShape,
};
use crate::subscription::QueryPatchGroupKind;

use crate::runtime::tests::graph_read_access::support::graph_read_access_cost_model::{
    simple_traversal_family, workspace,
};
use support::aspect_touch as touch;
use support::graph_index_inventory::runtime_profiles::profile_requiring_store_backed_graph_index;
use support::graph_read_access::persistent_requirements::{
    persistent_predicate_family, persistent_requirement_workspace,
};

#[test]
fn live_plan_preserves_one_shot_access_shape_and_required_index_digests() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(request_execution, "phase-thirteen-live-plan-parity");
    let family = simple_traversal_family(&mut workspace, "phase-thirteen-simple");
    let one_shot = match plan_admitted_graph_read_access_for_family(&family) {
        Ok(Some(plan)) => plan,
        Ok(None) => panic!("one-shot graph read did not admit"),
        Err(error) => panic!("one-shot graph read planning failed: {error:?}"),
    };

    let live_plan = workspace
        .plan_live_graph_read_access(&family, WorthQueryLiveGraphReadMaintenanceBudget::bounded())
        .expect("simple traversal should admit live maintenance");

    assert_eq!(
        live_plan.one_shot_access_shape_digest(),
        one_shot
            .admission()
            .requirement_set()
            .access_shape_digest()
            .render_hex()
    );
    assert_eq!(
        live_plan.required_index_digest(),
        one_shot
            .graph_index_support()
            .requirement_set_digest()
            .render_hex()
    );
    assert_eq!(
        live_plan.posture(),
        &WorthQueryLiveGraphReadAccessPosture::AdmittedLiveIncrementalMaintenance
    );
    assert_eq!(
        live_plan
            .mutation_delta_scope()
            .affected_requirement_row_count(),
        one_shot.admission().requirement_set().rows().len()
    );
    assert!(!live_plan
        .mutation_delta_scope()
        .delta_scope_digest()
        .is_empty());
}

#[test]
fn live_planning_denies_one_shot_plan_that_requires_stronger_maintenance_support() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(request_execution, "phase-thirteen-live-plan-budget-denial");
    let family = simple_traversal_family(&mut workspace, "phase-thirteen-budget-denied");

    let denial = workspace
        .plan_live_graph_read_access(
            &family,
            WorthQueryLiveGraphReadMaintenanceBudget::strict_incremental(64, 16, 0),
        )
        .expect_err("one-shot safe read must not become live-maintained beyond budget");

    assert_eq!(
        denial.posture(),
        &WorthQueryLiveGraphReadAccessPosture::DeniedLiveMaintenanceBudget
    );
    assert!(!denial.digest().is_empty());
}

#[test]
fn denied_one_shot_live_planning_returns_live_specific_support_posture() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    // A missing store-backed predicate index denies the one-shot read for a
    // support reason, independent of any byte forecast.
    let mut workspace = persistent_requirement_workspace(
        request_execution,
        "phase-thirteen-live-plan-denial",
        profile_requiring_store_backed_graph_index(
            WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
        ),
    );
    let family = persistent_predicate_family(&mut workspace, "phase-thirteen-support-denied");

    let denial = workspace
        .plan_live_graph_read_access(&family, WorthQueryLiveGraphReadMaintenanceBudget::bounded())
        .expect_err("a denied one-shot read must not silently become live-maintained");

    assert_eq!(
        denial.posture(),
        &WorthQueryLiveGraphReadAccessPosture::DeniedLiveMaintenanceSupport
    );
    assert!(!denial.digest().is_empty());
}

#[test]
fn live_read_receipt_exposes_live_graph_access_counters() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(request_execution, "phase-thirteen-live-receipt");
    let live_view = workspace
        .live_view::<WorthQueryUnrefinedLiveShape>(request_execution, "tasks.table", |query| {
            query
                .from("Task")
                .select([
                    crate::facade::foundation::AspectFieldKey::from_authoring_parts(
                        "identity", "id",
                    )
                    .unwrap(),
                    crate::facade::foundation::AspectFieldKey::from_authoring_parts(
                        "title", "value",
                    )
                    .unwrap(),
                ])
                .order_by(
                    crate::facade::foundation::AspectFieldKey::from_authoring_parts(
                        "title", "value",
                    )
                    .unwrap(),
                )
                .schema_basis("phase-thirteen-live-receipt")
        })
        .expect("live view should declare");

    let result = workspace
        .read_live_result(request_execution, &live_view)
        .expect("live read should execute");
    let receipt = result
        .receipt()
        .live_graph_read_access()
        .expect("live read receipt should expose graph access proof");
    let counters = receipt.maintenance_counters();

    assert!(receipt.proves_no_caller_owned_n_plus_one());
    assert!(!receipt.mutation_delta_scope_digest().is_empty());
    assert_eq!(counters.per_result_neighbor_lookup_count(), 0);
    assert_eq!(counters.strategy_recompute_count(), 0);
    assert_eq!(counters.background_index_build_count(), 0);
    assert_eq!(counters.touched_edge_count(), 2);
    assert_eq!(counters.touched_frontier_count(), 1);
    assert_eq!(counters.affected_requirement_row_count(), 2);
    assert_eq!(counters.skipped_unaffected_requirement_count(), 0);
    assert_eq!(counters.index_update_count(), 2);
    assert_eq!(counters.live_view_update_count(), 0);
}

#[test]
fn live_mutation_delivery_carries_graph_read_maintenance_receipt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "phase-thirteen-live-mutation-maintenance",
    );
    let live_view = workspace
        .live_view::<WorthQueryUnrefinedLiveShape>(request_execution, "tasks.table", |query| {
            query
                .from("Task")
                .select([
                    crate::facade::foundation::AspectFieldKey::from_authoring_parts(
                        "identity", "id",
                    )
                    .unwrap(),
                    crate::facade::foundation::AspectFieldKey::from_authoring_parts(
                        "title", "value",
                    )
                    .unwrap(),
                ])
                .order_by(
                    crate::facade::foundation::AspectFieldKey::from_authoring_parts(
                        "title", "value",
                    )
                    .unwrap(),
                )
                .schema_basis("phase-thirteen-live-mutation-maintenance")
        })
        .expect("live view should declare");

    workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(touch("identity.id"), authored_text("task-maintained"))
                .set_aspect(touch("title.value"), authored_text("Maintained task"))
        })
        .expect("insert should execute through the public runtime");

    let patches = workspace.observe(&live_view);
    let delivery = patches
        .query_delivery_batches
        .first()
        .expect("projected insert should deliver one live batch");
    let maintenance = delivery
        .live_graph_read_maintenance()
        .expect("relational live delivery should carry graph read maintenance proof");
    let counters = maintenance.maintenance_counters();

    assert_eq!(delivery.patch_group_width(), 2);
    assert_eq!(
        delivery.patch_group_kind(),
        QueryPatchGroupKind::CollectionMembershipPatchGroup
    );
    assert!(!maintenance.digest().is_empty());
    assert!(!maintenance.maintenance_delta_for_reporting().is_empty());
    assert!(!delivery.delivery_cause_for_reporting().is_empty());
    assert!(!maintenance.live_access_plan_digest().is_empty());
    assert!(!maintenance.mutation_delta_scope_digest().is_empty());
    assert_eq!(counters.mutation_delta_count(), 1);
    assert_eq!(counters.affected_requirement_row_count(), 2);
    assert_eq!(counters.touched_edge_count(), 2);
    assert_eq!(counters.touched_frontier_count(), 1);
    assert_eq!(counters.skipped_unaffected_requirement_count(), 0);
    assert_eq!(counters.index_update_count(), 2);
    assert_eq!(counters.live_view_update_count(), 1);
    assert_eq!(counters.per_result_neighbor_lookup_count(), 0);
    assert_eq!(counters.strategy_recompute_count(), 0);
    assert_eq!(counters.background_index_build_count(), 0);
}

#[test]
fn live_maintenance_receipt_tracks_projected_updates_without_hidden_overdelivery() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(request_execution, "phase-thirteen-live-update-maintenance");
    let live_view = workspace
        .live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.update.table",
            |query| {
                query
                    .from("Task")
                    .select([
                        crate::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "identity", "id",
                        )
                        .unwrap(),
                        crate::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "title", "value",
                        )
                        .unwrap(),
                    ])
                    .order_by(
                        crate::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "title", "value",
                        )
                        .unwrap(),
                    )
                    .schema_basis("phase-thirteen-live-update-maintenance")
            },
        )
        .expect("live view should declare");
    let seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(touch("identity.id"), authored_text("task-updated"))
                .set_aspect(touch("title.value"), authored_text("Original title"))
                .set_aspect(touch("description.value"), authored_text("hidden"))
        })
        .expect("seed insert should execute");
    let entity_identity = seed.deltas()[0].entity_identity().clone();
    let _ = workspace.observe(&live_view);

    workspace
        .update(request_execution, entity_identity.clone(), |task| {
            task.set_aspect(touch("title.value"), authored_text("Updated title"))
        })
        .expect("projected update should execute");
    let projected = workspace.observe(&live_view);
    let projected_maintenance = projected.query_delivery_batches[0]
        .live_graph_read_maintenance()
        .expect("projected update should carry maintenance proof");
    let projected_counters = projected_maintenance.maintenance_counters();

    workspace
        .update(request_execution, entity_identity, |task| {
            task.set_aspect(touch("description.value"), authored_text("hidden again"))
        })
        .expect("hidden update should execute");
    let hidden_only = workspace.observe(&live_view);

    assert_eq!(
        projected.query_delivery_batches[0].patch_group_kind(),
        QueryPatchGroupKind::DetailFieldPatchGroup
    );
    assert_eq!(
        projected_maintenance
            .maintenance_counters()
            .per_result_neighbor_lookup_count(),
        0
    );
    assert_eq!(projected_counters.mutation_delta_count(), 1);
    assert_eq!(projected_counters.affected_requirement_row_count(), 2);
    assert_eq!(projected_counters.touched_edge_count(), 1);
    assert_eq!(projected_counters.touched_frontier_count(), 1);
    assert_eq!(projected_counters.skipped_unaffected_requirement_count(), 1);
    assert_eq!(projected_counters.index_update_count(), 1);
    assert_eq!(projected_counters.live_view_update_count(), 1);
    assert_eq!(projected_counters.strategy_recompute_count(), 0);
    assert_eq!(projected_counters.background_index_build_count(), 0);
    assert!(hidden_only.query_delivery_batches.is_empty());
}

fn authored_text(value: impl Into<String>) -> crate::runtime::WorthQueryAuthoredAspectValue {
    crate::runtime::WorthQueryAuthoredAspectValue::string(value)
}
