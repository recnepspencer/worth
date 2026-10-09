use crate::diagnostics::data::RelationalDiagnosticValue;
use crate::tests::support::*;

// CONTRACT: retention_plan
// LANES: success, adversarial, recovery

#[test]
fn retention_plan_reports_active_root_obligations_without_per_record_snapshot_pins() {
    let runtime = runtime_with_test_schema();
    let entity_created = create_entity_outcome(&runtime, "entity-pinned");
    let entity_created_snapshot = runtime.visibility_authority().snapshot();
    let entity = changed_entities(&entity_created)[0];
    let _deleted_entity = delete_entity(&runtime, entity);
    let deleted_entity_snapshot = runtime.visibility_authority().snapshot();

    let relation_source = create_entity(&runtime, "relation-left");
    let relation_target = create_entity(&runtime, "relation-right");
    let relation_created =
        create_relation_outcome(&runtime, relation_source, relation_target, "r1");
    let relation_created_snapshot = runtime.visibility_authority().snapshot();
    let relation = changed_relations(&relation_created)[0];
    let _deleted_relation = {
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
        txn.push_batch(
            WorkerIntentBatch::new("delete-relation").push(MutationIntent::Relation(
                RelationMutationIntent::Delete(DeleteRelationIntent {
                    relation_id: relation,
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
        txn.commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
    };
    let deleted_relation_snapshot = runtime.visibility_authority().snapshot();

    let plan = runtime.retention().inspect_plan();

    assert!(plan.active_snapshot_count >= 4);
    assert_eq!(plan.snapshot_pinned_entities, 0);
    assert_eq!(plan.snapshot_pinned_relations, 0);
    assert!(plan.reclaimable_entities >= 1);

    let _maintenance = runtime.retention().run_pass();
    let retained_entity_view = runtime
        .read_truth()
        .read_snapshot(&entity_created_snapshot)
        .expect("the retained exact root remains readable after maintenance");
    assert!(retained_entity_view.get_entity(entity).is_some());
    let retained_relation_view = runtime
        .read_truth()
        .read_snapshot(&relation_created_snapshot)
        .expect("the retained exact root remains readable after maintenance");
    assert!(retained_relation_view.get_relation(relation).is_some());

    assert!(runtime
        .visibility_authority()
        .release_snapshot(&entity_created_snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&deleted_entity_snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&relation_created_snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&deleted_relation_snapshot)
        .is_ok());
}

#[test]
fn retention_plan_turns_deleted_records_reclaimable_after_snapshot_release() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "reclaimable");
    let created_snapshot = runtime.visibility_authority().snapshot();
    let entity = changed_entities(&created)[0];
    let _deleted = delete_entity(&runtime, entity);
    let deleted_snapshot = runtime.visibility_authority().snapshot();

    assert!(runtime
        .visibility_authority()
        .release_snapshot(&created_snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&deleted_snapshot)
        .is_ok());

    let plan = runtime.retention().inspect_plan();
    let pass = runtime.retention().run_pass();

    assert_eq!(plan.active_snapshot_count, 0);
    assert_eq!(plan.snapshot_pinned_entities, 0);
    assert!(plan.reclaimable_entities >= 1);
    assert!(pass.entity_reclaimable >= 1);
}

#[test]
fn lagging_sibling_root_survives_without_per_record_branch_pins() {
    let runtime = runtime_with_test_schema();
    let source = create_entity_outcome(&runtime, "source");
    let source_entity = changed_entities(&source)[0];
    let target = create_entity_outcome(&runtime, "target");
    let target_entity = changed_entities(&target)[0];
    let relation_created = create_relation_outcome(&runtime, source_entity, target_entity, "r1");
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let deleted = delete_entity(&runtime, source_entity);

    assert!(runtime
        .visibility_authority()
        .release_snapshot(&source.snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&target.snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&relation_created.snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&deleted.snapshot)
        .is_ok());

    let plan = runtime.retention().inspect_plan();

    assert_eq!(plan.snapshot_pinned_entities, 0);
    assert_eq!(plan.snapshot_pinned_relations, 0);
    assert_eq!(plan.branch_pinned_entities, 0);
    assert_eq!(plan.branch_pinned_relations, 0);
    let _ = runtime.retention().run_pass();
    let feature = runtime
        .branch_identity(&BranchId("feature".to_owned()))
        .unwrap();
    let (_, feature_basis) = runtime.observe_branch(&feature).unwrap();
    let feature_truth = runtime
        .read_truth()
        .read_observation(&feature_basis.observation())
        .unwrap();
    assert!(feature_truth.get_entity(source_entity).is_some());
}

#[test]
fn retention_inspection_keeps_legacy_branch_pin_counts_zero() {
    let runtime = runtime_with_test_schema();
    let source = create_entity_outcome(&runtime, "source");
    let source_entity = changed_entities(&source)[0];
    let target = create_entity_outcome(&runtime, "target");
    let target_entity = changed_entities(&target)[0];
    let relation_created = create_relation_outcome(&runtime, source_entity, target_entity, "r1");
    let relation = changed_relations(&relation_created)[0];
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let deleted = delete_entity(&runtime, source_entity);

    assert!(runtime
        .visibility_authority()
        .release_snapshot(&source.snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&target.snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&relation_created.snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&deleted.snapshot)
        .is_ok());

    let inspection = runtime.inspect_what_happened();
    let entity_retention = inspection
        .inspect_record_retention(RecordRef::Entity(source_entity))
        .expect("deleted entity retention");
    let relation_retention = inspection
        .inspect_record_retention(RecordRef::Relation(relation))
        .expect("retained relation retention");

    assert_eq!(entity_retention.pins.snapshot_pins, 0);
    assert_eq!(entity_retention.pins.replay_pins, 0);
    assert_eq!(entity_retention.pins.branch_pins, 0);

    assert_eq!(relation_retention.pins.snapshot_pins, 0);
    assert_eq!(relation_retention.pins.replay_pins, 0);
    assert_eq!(relation_retention.pins.branch_pins, 0);
    let feature = runtime
        .branch_identity(&BranchId("feature".to_owned()))
        .unwrap();
    let (_, feature_basis) = runtime.observe_branch(&feature).unwrap();
    let feature_truth = runtime
        .read_truth()
        .read_observation(&feature_basis.observation())
        .unwrap();
    assert!(feature_truth.get_entity(source_entity).is_some());
    assert!(feature_truth.get_relation(relation).is_some());
}

#[test]
fn retention_plan_reports_explicit_replay_pins_until_released() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "replay-pinned");
    let entity = changed_entities(&created)[0];
    let deleted = delete_entity(&runtime, entity);

    assert!(runtime
        .visibility_authority()
        .release_snapshot(&created.snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&deleted.snapshot)
        .is_ok());
    assert!(runtime
        .history_authority()
        .retain_version_for_replay(created.version_id));

    let pinned = runtime.retention().inspect_plan();
    assert!(pinned.replay_pinned_entities >= 1);
    assert_eq!(pinned.reclaimable_entities, 0);

    assert!(runtime
        .history_authority()
        .release_version_replay_retention(created.version_id));
    let released = runtime.retention().inspect_plan();
    assert_eq!(released.replay_pinned_entities, 0);
    assert!(released.reclaimable_entities >= 1);
}

#[test]
fn replay_retention_preserves_historical_live_entity_aspects_across_updates() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "replay-history");
    let entity = changed_entities(&created)[0];

    assert!(runtime
        .visibility_authority()
        .release_snapshot(&created.snapshot)
        .is_ok());
    assert!(runtime
        .history_authority()
        .retain_version_for_replay(created.version_id));

    let _updated = update_entity(&runtime, entity, "replay-history-updated");
    let historical = runtime.read_truth().read_version(created.version_id);

    assert_eq!(historical.entities.len(), 1);
    assert_eq!(
        read_entity_name(&historical.entities[0]),
        Some("replay-history".into())
    );

    assert!(runtime
        .history_authority()
        .release_version_replay_retention(created.version_id));
}

#[test]
fn retention_plan_reports_explicit_replay_pins_for_deleted_relations_until_released() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "replay-left");
    let target = create_entity(&runtime, "replay-right");
    let created = create_relation_outcome(&runtime, source, target, "replay-r1");
    let relation = changed_relations(&created)[0];
    let deleted = {
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
        txn.push_batch(
            WorkerIntentBatch::new("delete-relation").push(MutationIntent::Relation(
                RelationMutationIntent::Delete(DeleteRelationIntent {
                    relation_id: relation,
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
        txn.commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
    };

    assert!(runtime
        .visibility_authority()
        .release_snapshot(&created.snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&deleted.snapshot)
        .is_ok());
    assert!(runtime
        .history_authority()
        .retain_version_for_replay(created.version_id));

    let pinned = runtime.retention().inspect_plan();
    let diagnostics = runtime.publication().diagnostics();
    let retention_artifacts = diagnostics.by_scope(DiagnosticsScope::Retention);
    let latest_retention = retention_artifacts.last().unwrap();
    let latest_entry = latest_retention.entries.last().unwrap();
    assert!(retention_artifacts
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .any(|entry| entry.code == DiagnosticCode::ReplayRetentionPinned));
    assert!(pinned.replay_pinned_relations >= 1);
    assert_eq!(pinned.reclaimable_relations, 0);
    assert_eq!(latest_entry.code, DiagnosticCode::RetentionPlanInspected);
    assert_eq!(
        diagnostic_field(latest_entry, "replay_pinned_relations"),
        &RelationalDiagnosticValue::Unsigned(pinned.replay_pinned_relations as u64)
    );
    assert_eq!(
        diagnostic_field(latest_entry, "branch_pinned_relations"),
        &RelationalDiagnosticValue::Unsigned(pinned.branch_pinned_relations as u64)
    );
    assert_eq!(
        diagnostic_unsigned_field(latest_entry, "branch_replay_overlap_relations"),
        0
    );

    assert!(runtime
        .history_authority()
        .release_version_replay_retention(created.version_id));
    let released = runtime.retention().inspect_plan();
    assert_eq!(released.replay_pinned_relations, 0);
    assert_eq!(released.branch_pinned_relations, 0);
}

fn diagnostic_unsigned_field(
    entry: &crate::facade::diagnostics::RelationalDiagnosticsEntry,
    field: &str,
) -> u64 {
    match diagnostic_field(entry, field) {
        RelationalDiagnosticValue::Unsigned(value) => *value,
        other => panic!("expected unsigned diagnostic field {field}, got {other:?}"),
    }
}
