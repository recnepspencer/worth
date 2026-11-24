use super::filtering::relation_rule_kind;
use super::test_support::RequestGraph;
use crate::contracts::identity::data::primitives::{KindId, PartitionId};
use crate::contracts::transactions::data::intents::materialization_intent::{
    MaterializationMutationIntent, RematerializeRelationIntent,
};
use crate::contracts::transactions::data::intents::mutation_intent::{
    CreateIntent, DeleteEntityIntent, DeleteRelationIntent, EntityMutationIntent, MutationIntent,
    RelationMutationIntent, ReplaceEntityIntent,
};
use crate::contracts::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use crate::contracts::transactions::data::primitives::{
    EntityReference, EntitySpec, TransactionId,
};
use worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;

#[test]
fn request_excludes_unrelated_relation_kind_registrations_for_relation_create() {
    let graph = RequestGraph::new(2, &[]);
    let (source, target) = (graph.entities[0], graph.entities[1]);
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(11),
        merged_intents: vec![MutationIntent::Create(CreateIntent::Relation(
            crate::contracts::transactions::data::primitives::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::contracts::symbols::data::client_key::ClientKey::raw("planned"),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(target),
                fields: crate::contracts::transactions::data::aspect_field_patch::AspectFieldPatch::default(),
            },
        ))],
    };

    let instrumentation = RuntimeInstrumentation::default();
    let view = RequestGraph::view(&instrumentation);
    let request = graph.request(&view, Some(&plan));
    let included_relation_kinds = view
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
    let graph = RequestGraph::new(4, &[(0, 1, KindId(2)), (2, 3, KindId(3))]);
    let anchor = graph.entities[0];
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(12),
        merged_intents: vec![MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id: anchor },
        ))],
    };

    let instrumentation = RuntimeInstrumentation::default();
    let view = RequestGraph::view(&instrumentation);
    let request = graph.request(&view, Some(&plan));
    let included_relation_kinds = view
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
    let graph = RequestGraph::new(4, &[(0, 1, KindId(2)), (2, 3, KindId(3))]);
    let anchor = graph.entities[0];
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(13),
        merged_intents: vec![MutationIntent::Entity(EntityMutationIntent::Replace(
            ReplaceEntityIntent {
                entity_id: anchor,
                replacement: EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::contracts::symbols::data::client_key::ClientKey::raw("replacement"),
                    fields:
                        crate::contracts::transactions::data::aspect_field_patch::AspectFieldPatch::default(),
                },
            },
        ))],
    };

    let instrumentation = RuntimeInstrumentation::default();
    let view = RequestGraph::view(&instrumentation);
    let request = graph.request(&view, Some(&plan));
    let included_relation_kinds = view
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
    let graph = RequestGraph::new(2, &[(0, 1, KindId(2))]);
    let relation_id = graph.relations[0];
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(14),
        merged_intents: vec![MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent { relation_id },
        ))],
    };

    let instrumentation = RuntimeInstrumentation::default();
    let view = RequestGraph::view(&instrumentation);
    let request = graph.request(&view, Some(&plan));
    let included_relation_kinds = view
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
    let mut graph = RequestGraph::new(2, &[(0, 1, KindId(2))]);
    let (source, target) = (graph.entities[0], graph.entities[1]);
    let relation_id = graph.relations[0];
    graph.suspend(relation_id, (source, target));
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(15),
        merged_intents: vec![MutationIntent::Materialization(
            MaterializationMutationIntent::RematerializeRelation(RematerializeRelationIntent {
                relation_id,
                kind_id: KindId(2),
                source,
                target,
                fields: crate::contracts::transactions::data::aspect_field_patch::AspectFieldPatch::default(),
            }),
        )],
    };

    let instrumentation = RuntimeInstrumentation::default();
    let view = RequestGraph::view(&instrumentation);
    let request = graph.request(&view, Some(&plan));
    let scope = request
        .relation_integrity_scopes()
        .and_then(|scopes| scopes.scope_for(KindId(2)))
        .expect("rematerialization prepares its relation-integrity scope");
    assert_eq!(scope.planned_edges.len(), 1);
    assert_eq!(scope.source_counts.values().sum::<usize>(), 1);
    assert_eq!(scope.target_counts.values().sum::<usize>(), 1);
    let included_relation_kinds = view
        .schema_contract_runtime
        .relation_integrity_registrations
        .iter()
        .filter(|registration| request.includes_registration(registration))
        .filter_map(|registration| relation_rule_kind(&registration.rule))
        .collect::<Vec<_>>();

    assert_eq!(included_relation_kinds, vec![KindId(2)]);
}
