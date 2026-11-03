use std::collections::BTreeMap;

use worth_query_installation::facade::ApplicationScalarValueBinding;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::PartitionId;
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
use worth_relational::facade::transactions::WorkerIntentBatch;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationApprover;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationResource;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationStatusField;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub(super) use crate::domain_computation::primary_graph::tests::authorization::capability_elevation::review_support_mutation::complete_review_out_of_band;
pub(super) use crate::domain_computation::primary_graph::tests::authorization::capability_elevation::review_support_mutation::replace_support_grantor_with_custodian;

pub fn set_status(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    elevation_identity: &str,
    status: CapabilityElevationStatus,
) {
    let elevation = world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            &phase_custody.phase_for_application(&world.application),
            CapabilityElevationIdentity::reference(),
            elevation_identity.to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let field = CapabilityElevationStatusField::reference();
    let graph = world.application.runtime.primary_graph().unwrap();
    let locator = graph
        .layout()
        .field_locator(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    let fields = AspectFieldPatch::from(BTreeMap::from([(
        locator,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationStatusBinding::encode(&status).unwrap(),
    )]));
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new("set-elevation-status").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: elevation.entity_id(),
                fields,
            }),
        )),
    );
}

pub fn add_self_approver(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    elevation_identity: &str,
    requester: EntityId,
) {
    let elevation = world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            &phase_custody.phase_for_application(&world.application),
            CapabilityElevationIdentity::reference(),
            elevation_identity.to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let graph = world.application.runtime.primary_graph().unwrap();
    let relation_kind = graph
        .layout()
        .relation(CapabilityElevationApprover::reference().name())
        .unwrap()
        .kind;
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new("add-self-approver").push(MutationIntent::Create(
            CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: relation_kind,
                client_key: ClientKey::raw("elevation-self-approver"),
                source: EntityReference::Existing(requester),
                target: EntityReference::Existing(elevation.entity_id()),
                fields: AspectFieldPatch::default(),
            }),
        )),
    );
}

pub fn replace_elevation_resource(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    elevation_identity: &str,
    replacement_account: Option<&str>,
) {
    let elevation = world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            &phase_custody.phase_for_application(&world.application),
            CapabilityElevationIdentity::reference(),
            elevation_identity.to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let replacement = replacement_account.map(|identity| {
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
                identity.to_owned(),
                &live_scope(),
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap()
            .entity_id()
    });
    let graph = world.application.runtime.primary_graph().unwrap();
    let relation_kind = graph
        .layout()
        .relation(CapabilityElevationResource::reference().name())
        .unwrap()
        .kind;
    let handle = graph.integration_handle();
    let selected = world.selected_product(&(phase_custody).test_host_request());
    let relation = handle.with_runtime_mut(execution, |runtime| {
        let snapshot = selected.application_basis().snapshot_handle();

        runtime
            .read_truth()
            .visible_relations_of_kind(execution, relation_kind, snapshot.version_id())
            .expect("fixture read request remains live")
            .into_iter()
            .find(|record| record.source == elevation.entity_id())
            .expect("the elevation has one current direct resource")
            .relation_id
    });
    let mut batch = WorkerIntentBatch::new("replace-elevation-resource").push(
        MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
            relation_id: relation,
        })),
    );
    if let Some(account) = replacement {
        batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: relation_kind,
                client_key: ClientKey::raw("replacement-elevation-resource"),
                source: EntityReference::Existing(elevation.entity_id()),
                target: EntityReference::Existing(account),
                fields: AspectFieldPatch::default(),
            },
        )));
    }
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(execution, world, batch);
}

pub fn add_elevation_resource(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    elevation_identity: &str,
    account_identity: &str,
) {
    let elevation = world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            &phase_custody.phase_for_application(&world.application),
            CapabilityElevationIdentity::reference(),
            elevation_identity.to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            &phase_custody.phase_for_application(&world.application),
            AccountIdentity::reference(),
            account_identity.to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let graph = world.application.runtime.primary_graph().unwrap();
    let relation_kind = graph
        .layout()
        .relation(CapabilityElevationResource::reference().name())
        .unwrap()
        .kind;
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new("add-elevation-resource").push(MutationIntent::Create(
            CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: relation_kind,
                client_key: ClientKey::raw("additional-elevation-resource"),
                source: EntityReference::Existing(elevation.entity_id()),
                target: EntityReference::Existing(account.entity_id()),
                fields: AspectFieldPatch::default(),
            }),
        )),
    );
}
