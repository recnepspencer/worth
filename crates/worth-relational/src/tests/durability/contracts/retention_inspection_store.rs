use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn durability_contract_recovery_rebuilds_branch_head_root_obligations() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let source = create_entity_outcome(execution, &runtime, "source");
    let source_entity = changed_entities(&source)[0];
    let target = create_entity_outcome(execution, &runtime, "target");
    let target_entity = changed_entities(&target)[0];
    let _relation =
        create_relation_outcome(execution, &runtime, source_entity, target_entity, "r1");
    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let _deleted = delete_entity(execution, &runtime, source_entity);
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = durable_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    let retention = recovered.retention().inspect_plan(execution).unwrap();
    assert_eq!(retention.active_snapshot_count, 0);
    assert_eq!(retention.branch_pinned_entities, 0);
    assert_eq!(retention.branch_pinned_relations, 0);
    let feature = recovered
        .branch_identity(&BranchId("feature".to_owned()))
        .unwrap();
    let (_, feature_basis) = recovered.observe_branch(&feature).unwrap();
    let feature_truth = recovered
        .read_truth()
        .read_observation(execution, &feature_basis.observation())
        .expect("read request remains live")
        .unwrap();
    assert!(feature_truth.get_entity(source_entity).is_some());
    assert!(feature_truth
        .relations()
        .iter()
        .any(|relation| { relation.source == source_entity && relation.target == target_entity }));
}

#[test]
fn durability_contract_recovery_preserves_inspection_truth_bundle() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "source");
    let entity = changed_entities(&created)[0];
    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let _main_update = update_entity(execution, &runtime, entity, "main");
    let _feature_update = {
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_branch(
            &runtime,
            BranchId("feature".to_string()),
        );
        txn.push_batch(
            WorkerIntentBatch::new("feature-update").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: crate::tests::support::single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "feature",
                    ),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
        txn.commit(execution, &runtime).unwrap()
    };
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let expected = capture_inspection_truth_bundle(
        execution,
        &runtime,
        &BranchId("feature".to_string()),
        entity,
        created.version_id,
    );

    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = durable_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let actual = capture_inspection_truth_bundle(
        execution,
        &recovered,
        &BranchId("feature".to_string()),
        entity,
        created.version_id,
    );

    assert_eq!(expected, actual);
}

#[test]
fn durability_contract_branch_heads_do_not_mutate_legacy_record_pins() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "source");
    let entity = changed_entities(&created)[0];
    let inspection = runtime.inspect_what_happened();
    assert_eq!(
        inspection
            .inspect_record_retention(execution, RecordRef::Entity(entity))
            .unwrap()
            .expect("entity retention after create")
            .pins
            .branch_pins,
        0
    );

    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let inspection = runtime.inspect_what_happened();
    assert_eq!(
        inspection
            .inspect_record_retention(execution, RecordRef::Entity(entity))
            .unwrap()
            .expect("entity retention after branch create")
            .pins
            .branch_pins,
        0
    );

    update_entity(execution, &runtime, entity, "main");
    let inspection = runtime.inspect_what_happened();
    assert_eq!(
        inspection
            .inspect_record_retention(execution, RecordRef::Entity(entity))
            .unwrap()
            .expect("entity retention after main update")
            .pins
            .branch_pins,
        0
    );

    update_entity_on_branch(
        execution,
        &runtime,
        entity,
        "feature",
        BranchId("feature".to_string()),
    );
    let inspection = runtime.inspect_what_happened();
    assert_eq!(
        inspection
            .inspect_record_retention(execution, RecordRef::Entity(entity))
            .unwrap()
            .expect("entity retention after feature update")
            .pins
            .branch_pins,
        0
    );
}
