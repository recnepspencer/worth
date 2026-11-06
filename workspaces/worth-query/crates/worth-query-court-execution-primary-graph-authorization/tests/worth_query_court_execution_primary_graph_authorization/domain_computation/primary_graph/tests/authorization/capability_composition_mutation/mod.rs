use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::identity::RelationId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::DeleteRelationIntent;
use worth_relational::facade::transactions::EntityReference;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::RelationMutationIntent;
use worth_relational::facade::transactions::RelationSpec;
use worth_relational::facade::transactions::WorkerIntentBatch;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityActionRecord;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::CapabilityActionRecordIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::PrincipalIdentityField;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub fn add_policy_relation<Relation>(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
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

pub fn add_action_policy_relation<Relation>(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
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

pub fn remove_policy_relation<Relation>(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
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

pub fn actor_and_account(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
) -> (EntityId, EntityId) {
    let principal = actor(phase_custody, execution, world);
    let scope = live_scope();
    let account = world
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
            phase_custody,
            AccountIdentity::reference(),
            "account-1".to_owned(),
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id_for_test();
    (principal, account)
}

pub fn actor(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
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
            phase_custody,
            PrincipalIdentityField::reference(),
            1_u64,
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id_for_test()
}

pub fn resolve_action_record(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    record: &str,
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
            phase_custody,
            CapabilityActionRecordIdentity::reference(),
            record.to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id_for_test()
}

pub fn relation_kind(
    world: &AuthorizationWorld,
    name: &str,
) -> worth_relational::facade::identity::KindId {
    *(*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap()
        .layout_for_test()
        .relation(name)
        .unwrap()
        .kind_for_test()
}

pub fn current_relation(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    kind: worth_relational::facade::identity::KindId,
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

pub fn mutate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    build: impl FnOnce(WorkerIntentBatch) -> WorkerIntentBatch,
) {
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
        execution,
        world,
        build(WorkerIntentBatch::new("capability-composition-hostility")),
    );
}
