use std::collections::BTreeMap;

use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::AspectValue;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::identity::RelationId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::DeleteRelationIntent;
use worth_relational::facade::transactions::EntityMutationIntent;
use worth_relational::facade::transactions::EntityReference;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::RelationMutationIntent;
use worth_relational::facade::transactions::RelationSpec;
use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
use worth_relational::facade::transactions::UpdateRelationEndpointsIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityParent;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub fn field<F: worth_query_declaration::facade::application_schema::fragment::RecordSelection>(
    world: &AuthorizationWorld,
    field: F,
) -> AspectFieldLocator {
    installed_field(world, field.entity(), field.aspect(), field.field())
}

pub fn installed_field(
    world: &AuthorizationWorld,
    entity: &str,
    aspect: &str,
    field: &str,
) -> AspectFieldLocator {
    (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap()
        .layout_for_test()
        .field_locator_for_test(entity, aspect, field)
        .unwrap()
        .clone()
}

pub fn grant(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    key: &str,
) -> EntityId {
    world
        .application
        .select_product_branch_for_test(
            execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            &phase_custody.phase_for_application_for_test(&world.application),
            CapabilityIdentity::at("CapabilityIdentity"),
            key.into(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id_for_test()
}

pub fn account(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    key: &str,
) -> EntityId {
    world
        .application
        .select_product_branch_for_test(
            execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            &phase_custody.phase_for_application_for_test(&world.application),
            AccountIdentity::at("AccountIdentity"),
            key.to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id_for_test()
}

pub fn update_grant_field(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    key: &str,
    field: AspectFieldLocator,
    value: AspectValue,
) {
    let grant = grant(
        &(phase_custody).test_host_request_for_test(),
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

pub fn add_parent(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
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

pub fn replace_parent(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    child: &str,
    old_parent: &str,
    new_parent: &str,
) {
    let child = grant(
        &(phase_custody).test_host_request_for_test(),
        &phase_custody.borrowed_request_for_test(),
        world,
        child,
    );
    let kind = relation_kind(world, CapabilityParent::reference().name());
    replace_relation_target(
        &(phase_custody).test_host_request_for_test(),
        &phase_custody.borrowed_request_for_test(),
        world,
        kind,
        child,
        grant(
            &(phase_custody).test_host_request_for_test(),
            &phase_custody.borrowed_request_for_test(),
            world,
            old_parent,
        ),
        grant(
            &(phase_custody).test_host_request_for_test(),
            &phase_custody.borrowed_request_for_test(),
            world,
            new_parent,
        ),
        "alternate-parent",
    );
}

pub fn relation_kind(world: &AuthorizationWorld, name: &str) -> KindId {
    *(*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap()
        .layout_for_test()
        .relation(name)
        .unwrap()
        .kind_for_test()
}

pub fn relation_source(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    kind: KindId,
    target: EntityId,
) -> EntityId {
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    graph
        .integration_handle_for_test()
        .with_runtime_for_test(execution, |runtime| {
            let snapshot = selected.application_basis_for_test().snapshot_handle();

            runtime
                .read_truth()
                .visible_relations_of_kind(execution, kind, snapshot.version_id())
                .expect("fixture read request remains live")
                .into_iter()
                .find(|record| record.target == target)
                .unwrap()
                .source
        })
}

pub fn replace_relation_source(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
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

pub fn replace_relation_target(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
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

pub fn retarget_relation_without_replacing_id(
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

pub fn replace_relation_kind(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    old_kind: KindId,
    new_kind: KindId,
    source: EntityId,
    target: EntityId,
    key: &str,
) {
    let relation = current_relation(
        &(phase_custody).test_host_request_for_test(),
        execution,
        world,
        old_kind,
        source,
        target,
    );
    replace_relation(execution, world, relation, new_kind, source, target, key);
}

pub fn current_relation(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    kind: KindId,
    source: EntityId,
    target: EntityId,
) -> RelationId {
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    graph
        .integration_handle_for_test()
        .with_runtime_for_test(execution, |runtime| {
            let snapshot = selected.application_basis_for_test().snapshot_handle();

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

pub fn create_relation(
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

pub fn replace_relation(
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

pub fn mutate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    build: impl FnOnce(WorkerIntentBatch) -> WorkerIntentBatch,
) {
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
        execution,
        world,
        build(WorkerIntentBatch::new("delegation-hostility")),
    );
}
