use crate::facade::transactions::{EntitySpec, MutationIntent};
use crate::tests::support::*;

#[test]
fn bulk_mutation_plan_normalizes_client_keys_and_tracks_locality() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let target = create_entity_in_partition(&runtime, "target", PartitionId(7));

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("bulk-plan")
            .with_partition_key("planner-main")
            .push(MutationIntent::Create(CreateIntent::BulkEntities(
                crate::facade::transactions::BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_keys: vec![
                        crate::symbols::data::ClientKey::raw("bulk-a"),
                        crate::symbols::data::ClientKey::raw("bulk-b"),
                    ],
                    field_patches: vec![
                        crate::tests::support::single_string_aspect_field_patch(
                            crate::tests::support::aspect_key("name"),
                            crate::tests::support::field_key("name"),
                            "bulk-a",
                        ),
                        crate::tests::support::single_string_aspect_field_patch(
                            crate::tests::support::aspect_key("name"),
                            crate::tests::support::field_key("name"),
                            "bulk-b",
                        ),
                    ],
                },
            )))
            .push(MutationIntent::Create(CreateIntent::BulkRelations(
                crate::facade::transactions::BulkRelationCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_keys: vec![crate::symbols::data::ClientKey::raw("cross-edge")],
                    endpoints: vec![(
                        crate::transactions::data::EntityReference::Existing(source),
                        crate::transactions::data::EntityReference::Existing(target),
                    )],
                    field_patches: vec![crate::transactions::data::AspectFieldPatch::default()],
                },
            ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let plan = txn
        .plan_bulk_mutation_batch(&runtime)
        .expect("planning succeeds")
        .expect("planned batch");

    assert_eq!(
        plan.scope,
        crate::facade::transactions::BulkMutationScope::BulkMixedMutation
    );
    assert_eq!(plan.locality.entity_target_count, 2);
    assert_eq!(plan.locality.relation_target_count, 1);
    assert_eq!(plan.locality.cross_partition_relation_count, 1);
    assert_eq!(
        plan.locality.touched_partitions.as_ref(),
        &[PartitionId::main(), PartitionId(7)]
    );
    assert_eq!(
        plan.provenance.worker_batch_names.as_ref(),
        &["bulk-plan".to_string()]
    );
    assert_eq!(
        plan.provenance.worker_partition_keys.as_ref(),
        &[Some("planner-main".to_string())]
    );
    assert!(!plan.naming.naming_digest.is_empty());
    assert!(!plan.provenance.provenance_digest.is_empty());
    assert_eq!(plan.naming.normalized_client_keys.len(), 3);
    if runtime.config().identity.client_key_symbol_policy
        != crate::symbols::data::ClientKeySymbolPolicy::Disabled
    {
        assert!(plan
            .naming
            .normalized_client_keys
            .iter()
            .all(|value: &crate::symbols::data::ClientKey| value.as_symbol().is_some()));
    }
}

#[test]
fn bulk_mutation_plan_captures_lineage_and_provenance_for_topology_rewrite() {
    let runtime = runtime_with_test_schema();
    let original = create_entity(&runtime, "original");
    let peer = create_entity(&runtime, "peer");
    let relation = create_relation(&runtime, original, peer, "original-edge");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("rewrite")
            .push(MutationIntent::Entity(EntityMutationIntent::Replace(
                ReplaceEntityIntent {
                    entity_id: original,
                    replacement: EntitySpec {
                        partition_id: PartitionId(9),
                        kind_id: KindId(1),
                        client_key: crate::symbols::data::ClientKey::raw("replacement"),
                        fields: crate::tests::support::single_string_aspect_field_patch(
                            crate::tests::support::aspect_key("name"),
                            crate::tests::support::field_key("name"),
                            "replacement",
                        ),
                    },
                },
            )))
            .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                DeleteRelationIntent {
                    relation_id: relation,
                },
            ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let plan = txn
        .plan_bulk_mutation_batch(&runtime)
        .expect("planning succeeds")
        .expect("planned batch");

    assert_eq!(
        plan.scope,
        crate::facade::transactions::BulkMutationScope::TopologyRegionRewrite
    );
    assert!(plan.lineage.transitions.iter().any(|transition| {
        matches!(
            transition,
            crate::facade::transactions::PlannedLineageTransition::ReplaceEntity {
                entity_id,
                replacement_partition_id,
                ..
            } if *entity_id == original && *replacement_partition_id == PartitionId(9)
        )
    }));
    assert!(plan.lineage.transitions.iter().any(|transition| {
        matches!(
            transition,
            crate::facade::transactions::PlannedLineageTransition::DeleteRelation {
                relation_id
            } if *relation_id == relation
        )
    }));
    assert!(plan
        .provenance
        .worker_batch_names
        .iter()
        .any(|name| name == "rewrite"));
    assert!(!plan.lineage.lineage_scope_digest.is_empty());
}

#[test]
fn bulk_mutation_plan_is_absent_for_empty_staging() {
    let runtime = runtime_with_test_schema();
    let txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);

    assert!(txn
        .plan_bulk_mutation_batch(&runtime)
        .expect("planning succeeds")
        .is_none());
}

#[test]
fn bulk_mutation_commit_records_admission_counters() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let target = create_entity_in_partition(&runtime, "target", PartitionId(4));

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("bulk-counters").push(MutationIntent::Create(
            CreateIntent::BulkRelations(crate::facade::transactions::BulkRelationCreateIntent {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_keys: vec![crate::symbols::data::ClientKey::raw("edge-a")],
                endpoints: vec![(
                    crate::transactions::data::EntityReference::Existing(source),
                    crate::transactions::data::EntityReference::Existing(target),
                )],
                field_patches: vec![crate::transactions::data::AspectFieldPatch::default()],
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    assert_eq!(outcome.complexity_delta().bulk_mutation_batch_count, 1);
    assert_eq!(
        outcome.complexity_delta().bulk_mutation_entity_target_count,
        0
    );
    assert_eq!(
        outcome
            .complexity_delta()
            .bulk_mutation_relation_target_count,
        1
    );
    assert_eq!(
        outcome
            .complexity_delta()
            .bulk_mutation_cross_partition_relation_count,
        1
    );
    assert_eq!(
        outcome
            .complexity_delta()
            .bulk_mutation_naming_normalization_count,
        1
    );
    assert_eq!(
        outcome
            .complexity_delta()
            .bulk_mutation_lineage_transition_count,
        1
    );
    assert_eq!(
        outcome
            .complexity_delta()
            .bulk_mutation_provenance_record_count,
        1
    );
}
