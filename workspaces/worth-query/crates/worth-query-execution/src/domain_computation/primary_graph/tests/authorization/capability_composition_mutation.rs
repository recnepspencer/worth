use worth_relational::facade::identity::{EntityId, PartitionId, RelationId};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, DeleteRelationIntent, EntityReference, MutationIntent,
    RelationMutationIntent, RelationSpec, WorkerIntentBatch,
};

use super::super::fixture::capability::{CapabilityActionRecord, CapabilityActionRecordIdentity};
use super::super::fixture::{
    live_scope, Account, AccountIdentity, AuthorizationWorld, IdentityExecutionSchema, Principal,
    PrincipalIdentityField,
};
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;

pub(super) fn add_policy_relation<Relation>(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    relation: worth_query_declaration::facade::application_schema::ApplicationRelationRef<
        IdentityExecutionSchema,
        Relation,
        Principal,
        Account,
    >,
    key: &str,
) {
    let (source, target) = actor_and_account(
        phase_custody,
        &phase_custody.borrowed_request_for_test(),
        world,
    );
    let kind = relation_kind(world, relation.name());
    mutate(&phase_custody.borrowed_request_for_test(), world, |batch| {
        batch.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: kind,
                client_key: ClientKey::raw(key),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(target),
                fields: AspectFieldPatch::default(),
            },
        )))
    });
}

pub(super) fn add_action_policy_relation<Relation>(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    relation: worth_query_declaration::facade::application_schema::ApplicationRelationRef<
        IdentityExecutionSchema,
        Relation,
        Principal,
        CapabilityActionRecord,
    >,
    key: &str,
    record: &str,
) {
    let source = actor(
        phase_custody,
        &phase_custody.borrowed_request_for_test(),
        world,
    );
    let target = resolve_action_record(
        phase_custody,
        &phase_custody.borrowed_request_for_test(),
        world,
        record,
    );
    let kind = relation_kind(world, relation.name());
    mutate(&phase_custody.borrowed_request_for_test(), world, |batch| {
        batch.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: kind,
                client_key: ClientKey::raw(key),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(target),
                fields: AspectFieldPatch::default(),
            },
        )))
    });
}

pub(super) fn remove_policy_relation<Relation>(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    relation: worth_query_declaration::facade::application_schema::ApplicationRelationRef<
        IdentityExecutionSchema,
        Relation,
        Principal,
        Account,
    >,
) {
    let (source, target) = actor_and_account(phase_custody, execution, world);
    let kind = relation_kind(world, relation.name());
    let relation = current_relation(phase_custody, execution, world, kind, source, target);
    mutate(execution, world, |batch| {
        batch.push(MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent {
                relation_id: relation,
            },
        )))
    });
}

fn actor_and_account(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
) -> (EntityId, EntityId) {
    let principal = actor(phase_custody, execution, world);
    let scope = live_scope();
    let account = world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            phase_custody,
            AccountIdentity::reference(),
            "account-1".to_owned(),
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    (principal, account)
}

fn actor(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
) -> EntityId {
    world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            phase_custody,
            PrincipalIdentityField::reference(),
            1_u64,
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id()
}

fn resolve_action_record(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    record: &str,
) -> EntityId {
    world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            phase_custody,
            CapabilityActionRecordIdentity::reference(),
            record.to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id()
}

fn relation_kind(
    world: &AuthorizationWorld,
    name: &str,
) -> worth_relational::facade::identity::KindId {
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout()
        .relation(name)
        .unwrap()
        .kind
}

fn current_relation(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    kind: worth_relational::facade::identity::KindId,
    source: EntityId,
    target: EntityId,
) -> RelationId {
    let graph = world.application.runtime.primary_graph().unwrap();
    let selected = world.selected_product(&(phase_custody).test_host_request());
    graph
        .integration_handle()
        .with_runtime_mut(execution, |runtime| {
            let snapshot = selected.application_basis().snapshot_handle();

            runtime
                .read_truth()
                .visible_relations_of_kind(execution, kind, snapshot.version_id())
                .expect("fixture read request remains live")
                .into_iter()
                .find(|record| record.source == source && record.target == target)
                .unwrap()
                .relation_id
        })
}

fn mutate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    build: impl FnOnce(WorkerIntentBatch) -> WorkerIntentBatch,
) {
    super::super::fixture::publish_relational_mutation(
        execution,
        world,
        build(WorkerIntentBatch::new("capability-composition-hostility")),
    );
}
