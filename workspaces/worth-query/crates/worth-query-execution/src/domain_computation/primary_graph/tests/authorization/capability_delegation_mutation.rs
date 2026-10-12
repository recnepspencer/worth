use std::collections::BTreeMap;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::identity::{EntityId, KindId, PartitionId, RelationId};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, DeleteRelationIntent, EntityMutationIntent, EntityReference,
    MutationIntent, RelationMutationIntent, RelationSpec, UpdateEntityFieldsIntent,
    UpdateRelationEndpointsIntent, WorkerIntentBatch,
};

use super::super::fixture::capability::CapabilityParent;
use super::super::fixture::{live_scope, AccountIdentity, AuthorizationWorld, CapabilityIdentity};
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;

pub(super) fn field<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>(
    world: &AuthorizationWorld,
    field: worth_query_declaration::facade::application_schema::ApplicationFieldRef<
        Schema,
        Entity,
        Aspect,
        Field,
        Value,
        Write,
        Equality,
        Unit,
    >,
) -> AspectFieldLocator
where
    Unit: worth_query_declaration::facade::application_schema::ApplicationFieldUnit,
{
    installed_field(world, field.entity(), field.aspect(), field.field())
}

pub(super) fn installed_field(
    world: &AuthorizationWorld,
    entity: &str,
    aspect: &str,
    field: &str,
) -> AspectFieldLocator {
    world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout()
        .field_locator(entity, aspect, field)
        .unwrap()
        .clone()
}

pub(super) fn grant(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    key: &str,
) -> EntityId {
    world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            &phase_custody.phase_for_application(&world.application),
            CapabilityIdentity::reference(),
            key.to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id()
}

pub(super) fn account(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    key: &str,
) -> EntityId {
    world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            &phase_custody.phase_for_application(&world.application),
            AccountIdentity::reference(),
            key.to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id()
}

pub(super) fn update_grant_field(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    key: &str,
    field: AspectFieldLocator,
    value: AspectValue,
) {
    let grant = grant(
        &(phase_custody).test_host_request(),
        &phase_custody.borrowed_request_for_test(),
        world,
        key,
    );
    mutate(&phase_custody.borrowed_request_for_test(), world, |batch| {
        batch.push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
            UpdateEntityFieldsIntent {
                entity_id: grant,
                fields: AspectFieldPatch::from(BTreeMap::from([(field, value)])),
            },
        )))
    });
}

pub(super) fn add_parent(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    world: &AuthorizationWorld,
    child: &str,
    parent: &str,
    key: &str,
) {
    create_relation(
        &phase_custody.borrowed_request_for_test(),
        world,
        relation_kind(world, CapabilityParent::reference().name()),
        grant(
            phase_custody,
            &phase_custody.borrowed_request_for_test(),
            world,
            child,
        ),
        grant(
            phase_custody,
            &phase_custody.borrowed_request_for_test(),
            world,
            parent,
        ),
        key,
    );
}

pub(super) fn replace_parent(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    child: &str,
    old_parent: &str,
    new_parent: &str,
) {
    let child = grant(
        &(phase_custody).test_host_request(),
        &phase_custody.borrowed_request_for_test(),
        world,
        child,
    );
    let kind = relation_kind(world, CapabilityParent::reference().name());
    replace_relation_target(
        &(phase_custody).test_host_request(),
        &phase_custody.borrowed_request_for_test(),
        world,
        kind,
        child,
        grant(
            &(phase_custody).test_host_request(),
            &phase_custody.borrowed_request_for_test(),
            world,
            old_parent,
        ),
        grant(
            &(phase_custody).test_host_request(),
            &phase_custody.borrowed_request_for_test(),
            world,
            new_parent,
        ),
        "alternate-parent",
    );
}

pub(super) fn relation_kind(world: &AuthorizationWorld, name: &str) -> KindId {
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

pub(super) fn relation_source(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    kind: KindId,
    target: EntityId,
) -> EntityId {
    let graph = world.application.runtime.primary_graph().unwrap();
    let selected = world.selected_product(&(phase_custody).test_host_request());
    graph
        .integration_handle()
        .with_runtime_mut(execution, |runtime| {
            let snapshot = selected.application_basis().snapshot_handle();
            let source = runtime
                .read_truth()
                .visible_relations_of_kind(execution, kind, snapshot.version_id())
                .into_iter()
                .find(|record| record.target == target)
                .unwrap()
                .source;
            source
        })
}

pub(super) fn replace_relation_source(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    kind: KindId,
    old_source: EntityId,
    new_source: EntityId,
    target: EntityId,
    key: &str,
) {
    let relation = current_relation(phase_custody, execution, world, kind, old_source, target);
    replace_relation(execution, world, relation, kind, new_source, target, key);
}

pub(super) fn replace_relation_target(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    kind: KindId,
    source: EntityId,
    old_target: EntityId,
    new_target: EntityId,
    key: &str,
) {
    let relation = current_relation(phase_custody, execution, world, kind, source, old_target);
    replace_relation(execution, world, relation, kind, source, new_target, key);
}

pub(super) fn retarget_relation_without_replacing_id(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    relation: RelationId,
    kind: KindId,
    source: EntityId,
    target: EntityId,
) {
    mutate(execution, world, |batch| {
        batch.push(MutationIntent::Relation(
            RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                relation_id: relation,
                kind_id: kind,
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(target),
            }),
        ))
    });
}

pub(super) fn replace_relation_kind(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    old_kind: KindId,
    new_kind: KindId,
    source: EntityId,
    target: EntityId,
    key: &str,
) {
    let relation = current_relation(
        &(phase_custody).test_host_request(),
        execution,
        world,
        old_kind,
        source,
        target,
    );
    replace_relation(execution, world, relation, new_kind, source, target, key);
}

fn current_relation(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    kind: KindId,
    source: EntityId,
    target: EntityId,
) -> RelationId {
    let graph = world.application.runtime.primary_graph().unwrap();
    let selected = world.selected_product(&(phase_custody).test_host_request());
    graph
        .integration_handle()
        .with_runtime_mut(execution, |runtime| {
            let snapshot = selected.application_basis().snapshot_handle();
            let relation = runtime
                .read_truth()
                .visible_relations_of_kind(execution, kind, snapshot.version_id())
                .into_iter()
                .find(|record| record.source == source && record.target == target)
                .unwrap()
                .relation_id;
            relation
        })
}

fn create_relation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    kind: KindId,
    source: EntityId,
    target: EntityId,
    key: &str,
) {
    mutate(execution, world, |batch| {
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

fn replace_relation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    relation: RelationId,
    kind: KindId,
    source: EntityId,
    target: EntityId,
    key: &str,
) {
    mutate(execution, world, |batch| {
        batch
            .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                DeleteRelationIntent {
                    relation_id: relation,
                },
            )))
            .push(MutationIntent::Create(CreateIntent::Relation(
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

fn mutate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    build: impl FnOnce(WorkerIntentBatch) -> WorkerIntentBatch,
) {
    super::super::fixture::publish_relational_mutation(
        execution,
        world,
        build(WorkerIntentBatch::new("delegation-hostility")),
    );
}
