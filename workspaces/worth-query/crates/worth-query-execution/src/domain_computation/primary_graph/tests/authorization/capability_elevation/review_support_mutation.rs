use std::collections::BTreeMap;

use worth_query_installation::facade::ApplicationScalarValueBinding;
use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, DeleteRelationIntent, EntityMutationIntent, EntityReference,
    MutationIntent, RelationMutationIntent, RelationSpec, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

use super::super::super::fixture::capability::{CapabilityCustodian, CapabilityGrantor};
use super::super::super::fixture::{
    live_scope, CapabilityIdentity, CapabilityReviewIdentity, CapabilityReviewStatus,
    CapabilityReviewStatusField, CapabilityReviewer,
};
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;

pub(super) fn complete_review_out_of_band(
    world: &super::super::super::fixture::AuthorizationWorld,
    reviewer: EntityId,
) {
    let review = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            CapabilityReviewIdentity::reference(),
            "review-2".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let field = CapabilityReviewStatusField::reference();
    let graph = world.application.runtime.primary_graph().unwrap();
    let locator = graph
        .layout()
        .field_locator(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    let relation_kind = graph
        .layout()
        .relation(CapabilityReviewer::reference().name())
        .unwrap()
        .kind;
    let fields = AspectFieldPatch::from(BTreeMap::from([(
        locator,
        super::super::super::fixture::CapabilityReviewStatusBinding::encode(
            &CapabilityReviewStatus::Completed,
        )
        .unwrap(),
    )]));
    super::super::super::fixture::publish_relational_mutation(
        world,
        WorkerIntentBatch::new("complete-review-out-of-band")
            .push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                UpdateEntityFieldsIntent {
                    entity_id: review.entity_id(),
                    fields,
                },
            )))
            .push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: relation_kind,
                    client_key: ClientKey::raw("out-of-band-reviewer"),
                    source: EntityReference::Existing(reviewer),
                    target: EntityReference::Existing(review.entity_id()),
                    fields: AspectFieldPatch::default(),
                },
            ))),
    );
}

pub(super) fn replace_support_grantor_with_custodian(
    world: &super::super::super::fixture::AuthorizationWorld,
    principal: EntityId,
) {
    let grant = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            CapabilityIdentity::reference(),
            "capability-1".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let graph = world.application.runtime.primary_graph().unwrap();
    let grantor_kind = graph
        .layout()
        .relation(CapabilityGrantor::reference().name())
        .unwrap()
        .kind;
    let custodian_kind = graph
        .layout()
        .relation(CapabilityCustodian::reference().name())
        .unwrap()
        .kind;
    let handle = graph.integration_handle();
    let selected = world.selected_product();
    let grantor = handle.with_runtime_mut(|runtime| {
        let snapshot = selected.application_basis().snapshot_handle();
        let grantor = runtime
            .read_truth()
            .visible_relations_of_kind(grantor_kind, snapshot.version_id())
            .into_iter()
            .find(|record| record.source == principal && record.target == grant.entity_id())
            .expect("the request support has one current grantor path")
            .relation_id;
        grantor
    });
    super::super::super::fixture::publish_relational_mutation(
        world,
        WorkerIntentBatch::new("replace-elevation-support-policy-path")
            .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                DeleteRelationIntent {
                    relation_id: grantor,
                },
            )))
            .push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: custodian_kind,
                    client_key: ClientKey::raw("capability-1-custodian"),
                    source: EntityReference::Existing(principal),
                    target: EntityReference::Existing(grant.entity_id()),
                    fields: AspectFieldPatch::default(),
                },
            ))),
    );
}
