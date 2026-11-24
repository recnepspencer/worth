use super::filtering::relation_rule_kind;
use super::test_support::{
    create_entity, create_relation_of_kind, relation_integrity_runtime, request_for_plan,
};
use crate::identity::data::KindId;
use crate::identity::data::PartitionId;
use crate::transactions::data::intents::materialization_intent::MaterializationMutationIntent;
use crate::transactions::data::intents::materialization_intent::RematerializeRelationIntent;
use crate::transactions::data::intents::materialization_intent::SuspendRelationMaterializationIntent;
use crate::transactions::data::intents::mutation_intent::CreateIntent;
use crate::transactions::data::intents::mutation_intent::DeleteEntityIntent;
use crate::transactions::data::intents::mutation_intent::DeleteRelationIntent;
use crate::transactions::data::intents::mutation_intent::EntityMutationIntent;
use crate::transactions::data::intents::mutation_intent::MutationIntent;
use crate::transactions::data::intents::mutation_intent::RelationMutationIntent;
use crate::transactions::data::intents::mutation_intent::ReplaceEntityIntent;
use crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use crate::transactions::data::primitives::EntityReference;
use crate::transactions::data::primitives::EntitySpec;
use crate::transactions::data::primitives::TransactionId;
use crate::transactions::data::primitives::WorkerIntentBatch;

#[test]
fn request_excludes_unrelated_relation_kind_registrations_for_relation_create() {
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = relation_integrity_runtime();
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(11),
        merged_intents: vec![MutationIntent::Create(CreateIntent::Relation(
            crate::transactions::data::primitives::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("planned"),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(target),
                fields: crate::transactions::data::aspect_field_patch::AspectFieldPatch::default(),
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
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = relation_integrity_runtime();
    let anchor = create_entity(execution, &runtime, "anchor");
    let target = create_entity(execution, &runtime, "target");
    let isolated_a = create_entity(execution, &runtime, "isolated-a");
    let isolated_b = create_entity(execution, &runtime, "isolated-b");
    create_relation_of_kind(
        execution,
        &runtime,
        KindId(2),
        anchor,
        target,
        "adjacent-kind2",
    );
    create_relation_of_kind(
        execution,
        &runtime,
        KindId(3),
        isolated_a,
        isolated_b,
        "remote-kind3",
    );

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
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = relation_integrity_runtime();
    let anchor = create_entity(execution, &runtime, "anchor");
    let target = create_entity(execution, &runtime, "target");
    let isolated_a = create_entity(execution, &runtime, "isolated-a");
    let isolated_b = create_entity(execution, &runtime, "isolated-b");
    create_relation_of_kind(
        execution,
        &runtime,
        KindId(2),
        anchor,
        target,
        "adjacent-kind2",
    );
    create_relation_of_kind(
        execution,
        &runtime,
        KindId(3),
        isolated_a,
        isolated_b,
        "remote-kind3",
    );

    let plan = MergedCommitPlan {
        transaction_id: TransactionId(13),
        merged_intents: vec![MutationIntent::Entity(EntityMutationIntent::Replace(
            ReplaceEntityIntent {
                entity_id: anchor,
                replacement: EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("replacement"),
                    fields:
                        crate::transactions::data::aspect_field_patch::AspectFieldPatch::default(),
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
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = relation_integrity_runtime();
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");
    let relation_id =
        create_relation_of_kind(execution, &runtime, KindId(2), source, target, "edge");

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
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = relation_integrity_runtime();
    let source = create_entity(execution, &runtime, "restored-source");
    let target = create_entity(execution, &runtime, "restored-target");
    let relation_id = create_relation_of_kind(
        execution,
        &runtime,
        KindId(2),
        source,
        target,
        "restored-edge",
    );
    let services = runtime.owner_component_services();
    let (_, basis) = services
        .basis_port()
        .observe_branch(&runtime.main_branch_identity())
        .expect("the relation has an owner basis");
    let mut suspension = runtime
        .begin_branch_transaction(
            &basis,
            crate::mvcc::RelationalTransactionIntent::materialization(
                crate::mvcc::transaction::intent::materialization_mode::RelationalMaterializationTransactionMode::Suspend,
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
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the exact relation suspension is staged");
    suspension
        .commit(execution, &runtime)
        .expect("the exact relation becomes unavailable");
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(15),
        merged_intents: vec![MutationIntent::Materialization(
            MaterializationMutationIntent::RematerializeRelation(RematerializeRelationIntent {
                relation_id,
                kind_id: KindId(2),
                source,
                target,
                fields: crate::transactions::data::aspect_field_patch::AspectFieldPatch::default(),
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
