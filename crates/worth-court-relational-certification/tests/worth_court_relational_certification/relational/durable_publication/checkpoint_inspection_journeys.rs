use super::branch_history_publications::create_relation_outcome;
use super::branch_history_publications::delete_entity;
use super::branch_history_publications::fork;
use super::branch_history_publications::transaction;
use super::branch_history_publications::update_entity;
use super::checkpoint_inspection_observation::capture_inspection_truth_bundle;
use super::checkpoint_source_capture::capture_durably;
use super::merge_execution_authoring::single_string_aspect_field_patch;
use worth_relational::facade::durability::RecoveryVerificationMode as RecoveryMode;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::transactions::EntityMutationIntent;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;
use worth_testkit_relational::aspect_storage::definition::aspect_key;
use worth_testkit_relational::aspect_storage::definition::field_key;
use worth_testkit_relational::aspect_storage::publications::changed_entities;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::create_entity_outcome;
use worth_testkit_relational::durable_publication::durable_runtime;

#[test]
fn durability_contract_recovery_rebuilds_branch_head_root_obligations() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let source = create_entity_outcome(execution, &runtime, "source");
    let source_entity = changed_entities(&source)[0];
    let target = create_entity_outcome(execution, &runtime, "target");
    let target_entity = changed_entities(&target)[0];
    let _relation =
        create_relation_outcome(execution, &runtime, source_entity, target_entity, "r1");
    fork(execution, &runtime, "feature");
    let _deleted = delete_entity(execution, &runtime, source_entity);
    capture_durably(execution, &runtime, &medium).unwrap();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = durable_runtime(&medium);
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

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
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let created = create_entity_outcome(execution, &runtime, "source");
    let entity = changed_entities(&created)[0];
    fork(execution, &runtime, "feature");
    let _main_update = update_entity(execution, &runtime, entity, "main");
    let _feature_update = {
        let mut txn = transaction(&runtime, &BranchId("feature".to_string()));
        txn.push_batch(
            WorkerIntentBatch::new("feature-update").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: single_string_aspect_field_patch(
                        aspect_key("name"),
                        field_key("name"),
                        "feature",
                    ),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
        txn.commit(execution, &runtime).unwrap()
    };
    capture_durably(execution, &runtime, &medium).unwrap();
    let expected = capture_inspection_truth_bundle(
        execution,
        &runtime,
        &BranchId("feature".to_string()),
        entity,
        created.version_id,
    );

    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = durable_runtime(&medium);
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
