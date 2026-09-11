use std::collections::BTreeMap;

use worth_query_installation::facade::ApplicationScalarValueBinding;
use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, DeleteRelationIntent, EntityMutationIntent, EntityReference,
    MutationIntent, RelationMutationIntent, RelationSpec, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

use super::super::super::fixture::{
    live_scope, AccountIdentity, CapabilityElevationApprover, CapabilityElevationIdentity,
    CapabilityElevationResource, CapabilityElevationStatus, CapabilityElevationStatusField,
};
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;

pub(super) use super::review_support_mutation::{
    complete_review_out_of_band, replace_support_grantor_with_custodian,
};

pub(super) fn set_status(
    world: &super::super::super::fixture::AuthorizationWorld,
    elevation_identity: &str,
    status: CapabilityElevationStatus,
) {
    let elevation = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
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
        super::super::super::fixture::CapabilityElevationStatusBinding::encode(&status).unwrap(),
    )]));
    super::super::super::fixture::publish_relational_mutation(
        world,
        WorkerIntentBatch::new("set-elevation-status").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: elevation.entity_id(),
                fields,
            }),
        )),
    );
}

pub(super) fn add_self_approver(
    world: &super::super::super::fixture::AuthorizationWorld,
    elevation_identity: &str,
    requester: EntityId,
) {
    let elevation = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
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
    super::super::super::fixture::publish_relational_mutation(
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

pub(super) fn replace_elevation_resource(
    world: &super::super::super::fixture::AuthorizationWorld,
    elevation_identity: &str,
    replacement_account: Option<&str>,
) {
    let elevation = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            CapabilityElevationIdentity::reference(),
            elevation_identity.to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let replacement = replacement_account.map(|identity| {
        world
            .application
            .select_product_branch(world.application.product_runtime().default_branch())
            .expect("the selected product branch remains admitted")
            .resolve_entity(
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
    let selected = world.selected_product();
    let relation = handle.with_runtime_mut(|runtime| {
        let snapshot = selected.application_basis().snapshot_handle();
        let relation = runtime
            .read_truth()
            .visible_relations_of_kind(relation_kind, snapshot.version_id())
            .into_iter()
            .find(|record| record.source == elevation.entity_id())
            .expect("the elevation has one current direct resource")
            .relation_id;
        relation
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
    super::super::super::fixture::publish_relational_mutation(world, batch);
}

pub(super) fn add_elevation_resource(
    world: &super::super::super::fixture::AuthorizationWorld,
    elevation_identity: &str,
    account_identity: &str,
) {
    let elevation = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            CapabilityElevationIdentity::reference(),
            elevation_identity.to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
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
    super::super::super::fixture::publish_relational_mutation(
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
