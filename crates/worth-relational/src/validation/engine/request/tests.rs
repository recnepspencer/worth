use super::filtering::relation_rule_kind;
use super::test_support::{
    create_entity, create_relation_of_kind, relation_integrity_runtime, request_for_plan,
};
use crate::identity::data::KindId;
use crate::identity::data::PartitionId;
use crate::transactions::data::EntityReference;
use crate::transactions::data::{
    CreateIntent, DeleteEntityIntent, DeleteRelationIntent, EntityMutationIntent, EntitySpec,
    MaterializationMutationIntent, MergedCommitPlan, MutationIntent, RelationMutationIntent,
    RematerializeRelationIntent, ReplaceEntityIntent, SuspendRelationMaterializationIntent,
    TransactionId, WorkerIntentBatch,
};

#[test]
fn request_excludes_unrelated_relation_kind_registrations_for_relation_create() {
    let runtime = relation_integrity_runtime();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(11),
        merged_intents: vec![MutationIntent::Create(CreateIntent::Relation(
            crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("planned"),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(target),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        ))],
    };

    let request = request_for_plan(&runtime, &plan);
    let included_relation_kinds = runtime
        .schema_contract_runtime
        .relation_integrity_registrations
        .iter()
        .filter(|registration| request.includes_registration(registration))
        .filter_map(|registration| relation_rule_kind(&registration.rule))
        .collect::<Vec<_>>();

    assert_eq!(included_relation_kinds, vec![KindId(2)]);
}

#[test]
fn request_excludes_unrelated_relation_kind_registrations_for_entity_delete() {
    let runtime = relation_integrity_runtime();
    let anchor = create_entity(&runtime, "anchor");
    let target = create_entity(&runtime, "target");
    let isolated_a = create_entity(&runtime, "isolated-a");
    let isolated_b = create_entity(&runtime, "isolated-b");
    create_relation_of_kind(&runtime, KindId(2), anchor, target, "adjacent-kind2");
    create_relation_of_kind(&runtime, KindId(3), isolated_a, isolated_b, "remote-kind3");

    let plan = MergedCommitPlan {
        transaction_id: TransactionId(12),
        merged_intents: vec![MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id: anchor },
        ))],
    };

    let request = request_for_plan(&runtime, &plan);
    let included_relation_kinds = runtime
        .schema_contract_runtime
        .relation_integrity_registrations
        .iter()
        .filter(|registration| request.includes_registration(registration))
        .filter_map(|registration| relation_rule_kind(&registration.rule))
        .collect::<Vec<_>>();

    assert_eq!(included_relation_kinds, vec![KindId(2)]);
}

#[test]
fn request_excludes_unrelated_relation_kind_registrations_for_entity_replace() {
    let runtime = relation_integrity_runtime();
    let anchor = create_entity(&runtime, "anchor");
    let target = create_entity(&runtime, "target");
    let isolated_a = create_entity(&runtime, "isolated-a");
    let isolated_b = create_entity(&runtime, "isolated-b");
    create_relation_of_kind(&runtime, KindId(2), anchor, target, "adjacent-kind2");
    create_relation_of_kind(&runtime, KindId(3), isolated_a, isolated_b, "remote-kind3");

    let plan = MergedCommitPlan {
        transaction_id: TransactionId(13),
        merged_intents: vec![MutationIntent::Entity(EntityMutationIntent::Replace(
            ReplaceEntityIntent {
                entity_id: anchor,
                replacement: EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("replacement"),
                    fields: crate::transactions::data::AspectFieldPatch::default(),
                },
            },
        ))],
    };

    let request = request_for_plan(&runtime, &plan);
    let included_relation_kinds = runtime
        .schema_contract_runtime
        .relation_integrity_registrations
        .iter()
        .filter(|registration| request.includes_registration(registration))
        .filter_map(|registration| relation_rule_kind(&registration.rule))
        .collect::<Vec<_>>();

    assert_eq!(included_relation_kinds, vec![KindId(2)]);
}

#[test]
fn request_includes_deleted_relation_kind_scope_for_delete_only_commits() {
    let runtime = relation_integrity_runtime();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let relation_id = create_relation_of_kind(&runtime, KindId(2), source, target, "edge");

    let plan = MergedCommitPlan {
        transaction_id: TransactionId(14),
        merged_intents: vec![MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent { relation_id },
        ))],
    };

    let request = request_for_plan(&runtime, &plan);
    let included_relation_kinds = runtime
        .schema_contract_runtime
        .relation_integrity_registrations
        .iter()
        .filter(|registration| request.includes_registration(registration))
        .filter_map(|registration| relation_rule_kind(&registration.rule))
        .collect::<Vec<_>>();

    assert_eq!(included_relation_kinds, vec![KindId(2)]);
}

#[test]
fn request_includes_rematerialized_relation_as_a_planned_edge() {
    let runtime = relation_integrity_runtime();
    let source = create_entity(&runtime, "restored-source");
    let target = create_entity(&runtime, "restored-target");
    let relation_id = create_relation_of_kind(&runtime, KindId(2), source, target, "restored-edge");
    let services = runtime.owner_component_services();
    let (_, basis) = services
        .basis_port()
        .observe_branch(&runtime.main_branch_identity())
        .expect("the relation has an owner basis");
    let mut suspension = runtime
        .begin_branch_transaction(
            &basis,
            crate::mvcc::RelationalTransactionIntent::materialization(
                crate::mvcc::RelationalMaterializationTransactionMode::Suspend,
            ),
        )
        .expect("owner materialization suspension is admitted");
    suspension
        .push_batch(
            WorkerIntentBatch::new("suspend-edge").push(MutationIntent::Materialization(
                MaterializationMutationIntent::SuspendRelation(
                    SuspendRelationMaterializationIntent { relation_id },
                ),
            )),
        )
        .expect("the exact relation suspension is staged");
    suspension
        .commit(&runtime)
        .expect("the exact relation becomes unavailable");
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(15),
        merged_intents: vec![MutationIntent::Materialization(
            MaterializationMutationIntent::RematerializeRelation(RematerializeRelationIntent {
                relation_id,
                kind_id: KindId(2),
                source,
                target,
                fields: crate::transactions::data::AspectFieldPatch::default(),
            }),
        )],
    };

    let request = request_for_plan(&runtime, &plan);
    let scope = request
        .relation_integrity_scopes()
        .and_then(|scopes| scopes.scope_for(KindId(2)))
        .expect("rematerialization prepares its relation-integrity scope");
    assert_eq!(scope.planned_edges.len(), 1);
    assert_eq!(scope.source_counts.values().sum::<usize>(), 1);
    assert_eq!(scope.target_counts.values().sum::<usize>(), 1);
    let included_relation_kinds = runtime
        .schema_contract_runtime
        .relation_integrity_registrations
        .iter()
        .filter(|registration| request.includes_registration(registration))
        .filter_map(|registration| relation_rule_kind(&registration.rule))
        .collect::<Vec<_>>();

    assert_eq!(included_relation_kinds, vec![KindId(2)]);
}
