use worth_query_declaration::facade::application_schema::ApplicationReadableScalarValueBinding;
use worth_relational::facade::identity::{EntityId, KindId};

use super::super::super::fixture::{
    live_scope, CapabilityElevationApprover, CapabilityElevationIdentity,
    CapabilityElevationStatus, CapabilityElevationStatusBinding, CapabilityElevationStatusField,
    CapabilityReviewIdentity, CapabilityReviewStatus, CapabilityReviewStatusBinding,
    CapabilityReviewStatusField, CapabilityReviewer,
};
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrincipalResolutionMode,
};

type World = super::approval_transition::World;

pub(super) fn elevation_status(world: &World) -> CapabilityElevationStatus {
    let identity = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            CapabilityElevationIdentity::reference(),
            "elevation-2".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let field = CapabilityElevationStatusField::reference();
    let locator = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout()
        .field_locator(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    let value = observed_field(
        &world.application,
        identity.entity_id(),
        identity.entity_kind(),
        &locator,
    );
    CapabilityElevationStatusBinding::decode(&value).unwrap()
}

pub(super) fn review_status(world: &World) -> CapabilityReviewStatus {
    let (review, kind) = resolved_review(world);
    let field = CapabilityReviewStatusField::reference();
    let locator = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout()
        .field_locator(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    let value = observed_field(&world.application, review, kind, &locator);
    CapabilityReviewStatusBinding::decode(&value).unwrap()
}

pub(super) fn has_exact_reviewer(world: &World, reviewer: EntityId) -> bool {
    let (review, _) = resolved_review(world);
    let graph = world.application.runtime.primary_graph().unwrap();
    let relation_kind = graph
        .layout()
        .relation(CapabilityReviewer::reference().name())
        .unwrap()
        .kind;
    let selected = world.selected_product();
    graph.integration_handle().with_runtime_mut(|runtime| {
        let snapshot = selected.application_basis().snapshot_handle();
        runtime
            .read_truth()
            .bounded_incoming_relations_of_kind_at_version(
                review,
                relation_kind,
                snapshot.version_id(),
                4,
            )
            .ok()
            .is_some_and(|read| {
                let relations = read.into_records();
                relations.len() == 1
                    && relations[0].source == reviewer
                    && relations[0].target == review
            })
    })
}

pub(super) fn has_exact_approver(world: &World, approver: EntityId) -> bool {
    let elevation = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            CapabilityElevationIdentity::reference(),
            "elevation-2".to_owned(),
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
    let selected = world.selected_product();
    graph.integration_handle().with_runtime_mut(|runtime| {
        let snapshot = selected.application_basis().snapshot_handle();
        runtime
            .read_truth()
            .bounded_incoming_relations_of_kind_at_version(
                elevation.entity_id(),
                relation_kind,
                snapshot.version_id(),
                4,
            )
            .ok()
            .is_some_and(|read| {
                let relations = read.into_records();
                relations.len() == 1
                    && relations[0].source == approver
                    && relations[0].target == elevation.entity_id()
            })
    })
}

fn resolved_review(world: &World) -> (EntityId, KindId) {
    let identity = world
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
    (identity.entity_id(), identity.entity_kind())
}

fn observed_field<Schema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    entity: EntityId,
    kind: KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
) -> worth_foundational::facade::AspectValue
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    let graph = runtime.runtime.primary_graph().unwrap();
    let selected = runtime
        .select_product_branch(runtime.product_runtime().default_branch())
        .expect("the selected product branch remains admitted");
    graph.integration_handle().with_runtime_mut(|relational| {
        let snapshot = selected.application_basis().snapshot_handle();
        crate::domain_computation::primary_graph::application_attempt::observe_field_value(
            relational, snapshot, entity, kind, locator,
        )
        .unwrap()
    })
}
