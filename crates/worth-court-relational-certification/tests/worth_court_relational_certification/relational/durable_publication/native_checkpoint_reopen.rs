//! Captured Native checkpoints restore through their public owner authority.
use super::branch_history_publications::{changed_relations, transaction};
use super::branch_history_recovery::current_head;
use super::checkpoint_recovery_observation::snapshot_for_branch;
use worth_relational::facade::{
    durability::RecoveryFailureClass,
    history::BranchId,
    identity::{KindId, PartitionId},
    runtime::RelationalRuntimeApi,
    transactions::{
        CreateIntent, CreatedEntityRef, EntityReference, EntitySpec, MutationIntent, RelationSpec,
        WorkerIntentBatch,
    },
};
use worth_testkit_relational::aspect_storage::publications::changed_entities;
use worth_testkit_relational::{
    durable_medium::TestDurableMedium,
    durable_publication::{create_entity_outcome, durable_runtime, field_patch},
};
#[test]
fn native_checkpoint_round_trip_restores_a_live_editable_world() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let committed = create_entity_outcome(execution, &runtime, "native-checkpoint");
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(
            execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the canonical committed world encodes as one opaque checkpoint");

    let mut recovered = durable_runtime(&medium);

    let pre_recovery_identity = recovered.runtime_instance_id();
    let outcome = recovered
        .durability_recovery()
        .restore_native_checkpoint(execution, &checkpoint)
        .expect("the opaque native checkpoint restores through durability authority");

    assert_ne!(recovered.runtime_instance_id(), pre_recovery_identity);
    assert_eq!(outcome.latest_commit, Some(committed.commit.clone()));
    assert_eq!(
        current_head(&recovered, &BranchId("main".to_owned())),
        Some(committed.commit.clone())
    );
    let later = create_entity_outcome(execution, &recovered, "post-restore-edit");
    assert_eq!(
        current_head(&recovered, &BranchId("main".to_owned())),
        Some(later.commit.clone())
    );
}

#[test]
fn recovery_authority_only_admits_a_basis_from_the_recovered_image() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let source = durable_runtime(&medium);
    create_entity_outcome(execution, &source, "recovered-authority-source");
    let checkpoint = source
        .durability_authority()
        .native_checkpoint(
            execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    let mut exact_recovery = durable_runtime(&medium);

    let (_, exact_authority) = exact_recovery
        .durability_recovery()
        .restore_native_checkpoint_with_authority(execution, &checkpoint)
        .unwrap();
    let (_, exact_basis) = exact_recovery
        .observe_branch(&exact_recovery.main_branch_identity())
        .unwrap();
    assert!(exact_authority.admit_basis(exact_basis).is_ok());

    let mut moved_recovery = durable_runtime(&medium);

    let (_, moved_authority) = moved_recovery
        .durability_recovery()
        .restore_native_checkpoint_with_authority(execution, &checkpoint)
        .unwrap();
    create_entity_outcome(execution, &moved_recovery, "post-recovery-movement");
    let (_, moved_basis) = moved_recovery
        .observe_branch(&moved_recovery.main_branch_identity())
        .unwrap();
    assert!(moved_authority.admit_basis(moved_basis).is_err());
}

#[test]
fn native_checkpoint_round_trip_restores_current_relation_adjacency() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let source_key = worth_relational::facade::symbols::ClientKey::raw("native-relation-source");
    let mut transaction = transaction(&runtime, &BranchId("main".into()));
    transaction
        .push_batch(
            WorkerIntentBatch::new("native-relation")
                .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: source_key.clone(),
                    fields: field_patch("name", "name", "native-relation-source"),
                })))
                .push(MutationIntent::Create(CreateIntent::Relation(
                    RelationSpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(2),
                        client_key: worth_relational::facade::symbols::ClientKey::raw(
                            "native-relation",
                        ),
                        source: EntityReference::Created(CreatedEntityRef {
                            partition_id: PartitionId::main(),
                            kind_id: KindId(1),
                            client_key: source_key.clone(),
                        }),
                        target: EntityReference::Created(CreatedEntityRef {
                            partition_id: PartitionId::main(),
                            kind_id: KindId(1),
                            client_key: source_key,
                        }),
                        fields: Default::default(),
                    },
                ))),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let created = transaction.commit(execution, &runtime).unwrap();
    let source = changed_entities(&created)[0];
    let target = source;
    let relation = changed_relations(&created)[0];
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(
            execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    let mut recovered = durable_runtime(&medium);

    recovered
        .durability_recovery()
        .restore_native_checkpoint(execution, &checkpoint)
        .unwrap();
    let rows = recovered
        .read_truth()
        .bounded_outgoing_relations_of_kind_at_version(
            execution,
            source,
            worth_relational::facade::identity::KindId(2),
            super::branch_history_recovery::main_version(&recovered),
            4,
        )
        .expect("read request remains live")
        .unwrap()
        .into_records();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].relation_id, relation);
    assert_eq!(rows[0].target, target);
    let snapshot = snapshot_for_branch(&recovered, &BranchId("main".to_owned()));
    let projected = recovered
        .read_truth()
        .project_snapshot(&snapshot)
        .unwrap()
        .bounded_outgoing_relations_for_frontier(
            execution,
            &std::collections::BTreeSet::from([source]),
            KindId(2),
            Some(4),
        )
        .expect("read request remains live")
        .unwrap()
        .into_records();
    assert_eq!(projected.len(), 1);
    assert_eq!(projected[0].relation_id, relation);
}

#[test]
fn native_checkpoint_rejects_a_foreign_runtime_name_without_publishing_state() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity_outcome(execution, &runtime, "native-checkpoint-runtime-name");
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(
            execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut recovered = RelationalRuntimeApi::builder()
        .schema_registry(worth_testkit_relational::durable_publication::schema())
        .runtime_name("foreign-runtime")
        .build_durable(TestDurableMedium::new().attachment())
        .expect("valid native runtime configuration");

    let error = recovered
        .durability_recovery()
        .restore_native_checkpoint(execution, &checkpoint)
        .expect_err("a foreign runtime checkpoint cannot be admitted");
    let error = super::checkpoint_recovery_observation::failure(error);

    assert_eq!(error.class, RecoveryFailureClass::RuntimeNameMismatch);
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}
