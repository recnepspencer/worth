use worth_runtime_bridge::facade::BridgeMixedCauseOrderingInput;
use worth_signal::facade::NodeId;
use worth_ui::facade::app::WorthUiApplicationPreparationDenial;
use worth_ui::facade::intent::{
    UiIntentCatalogPreparationDenial, UiIntentDeclaration, UiIntentPayloadSource,
    UiIntentPayloadStop, UiIntentText,
};
use worth_ui_dsl::WorthUiIntentInteractionFamily;
use worth_ui_query_binding::{
    UiCollectionProjectionRegistration, UiProjectionFieldRequirement, UiProjectionInputPosture,
    UiProjectionRetainedActivityKind, UiProjectionUnavailableKind, WorthUiQueryWorkspaceExt,
};

use super::super::payload_types::{QueryTextIntent, QUERY_TEXT_FIELD};
use super::super::world::{
    launch, routed_input, PayloadApplicationFacts, PayloadProjectionRegistration, DECLARATION,
};
use crate::projection_lifecycle::async_fixture::authoritative_async_basis;
use crate::projection_lifecycle::support::ScalarLifecycleWorld;

#[test]
fn ia_05_missing_pending_and_retained_stale_query_inputs_stop_exactly() {
    assert_missing_projection_stops();
    assert_pending_projection_stops();
    assert_retained_stale_projection_stops();
    assert_collection_text_shape_stops_at_catalog();
}

fn assert_missing_projection_stops() {
    let (query, _) = ScalarLifecycleWorld::standard(NodeId::new(314_051, 0), "query-current");
    let registration = super::scalar_registration(&query);
    let projection = registration.view().identity().clone();
    let mut world = query_world(registration, &projection);

    assert_eq!(
        prepare_stop(&mut world),
        UiIntentPayloadStop::ProjectionUnavailable {
            field: QUERY_TEXT_FIELD.descriptor().stable_name(),
            projection,
        }
    );
    let _ = world.interaction.session.shutdown();
}

fn assert_pending_projection_stops() {
    let (mut query, _) = ScalarLifecycleWorld::standard(NodeId::new(314_052, 0), "query-current");
    let registration = super::scalar_registration(&query);
    let projection = registration.view().identity().clone();
    let mut world = query_world(registration, &projection);
    let pending = query.initial().into_fact_and_predecessor().0;
    super::publish_scalar(&mut world, pending.into_observation(), 314_052);
    world.interaction.publish_successor();

    assert_eq!(
        prepare_stop(&mut world),
        UiIntentPayloadStop::ProjectionNotCurrent {
            field: QUERY_TEXT_FIELD.descriptor().stable_name(),
            posture: UiProjectionInputPosture::Unavailable(UiProjectionUnavailableKind::Pending),
        }
    );
    let _ = world.interaction.session.shutdown();
}

fn assert_retained_stale_projection_stops() {
    let (mut query, completion) =
        ScalarLifecycleWorld::standard(NodeId::new(314_053, 0), "query-current");
    let registration = super::scalar_registration(&query);
    let projection = registration.view().identity().clone();
    let mut world = query_world(registration, &projection);
    let pending = query.initial().into_fact_and_predecessor().0;
    let current = query
        .advance(
            BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
            Some(pending),
        )
        .into_fact_and_predecessor()
        .0;
    let revalidation = query
        .bridge
        .revalidate_async_request(
            &query.request,
            authoritative_async_basis("phase3-change", "phase3-snapshot"),
        )
        .expect("the Query bridge issues exact revalidation lineage");
    let stale = query
        .advance(
            BridgeMixedCauseOrderingInput::AsyncRevalidationLineage(revalidation),
            Some(current),
        )
        .into_fact_and_predecessor()
        .0;
    super::publish_scalar(&mut world, stale.into_observation(), 314_053);
    world.interaction.publish_successor();

    assert_eq!(
        prepare_stop(&mut world),
        UiIntentPayloadStop::ProjectionNotCurrent {
            field: QUERY_TEXT_FIELD.descriptor().stable_name(),
            posture: UiProjectionInputPosture::RetainedStale(
                UiProjectionRetainedActivityKind::Revalidating,
            ),
        }
    );
    let _ = world.interaction.session.shutdown();
}

fn assert_collection_text_shape_stops_at_catalog() {
    let (query, _) = worth_ui_query_binding::certification::seeded_collection_projection_workspace(
        vec![("pulse.alpha".to_owned(), "Alpha".to_owned())],
        worth_ui_query_binding::certification::WorthUiCollectionProjectionSeedPosture::Complete,
    );
    let domain = query.worth_ui().expect("Worth UI Query domain installed");
    let registration = UiCollectionProjectionRegistration::text(
        domain
            .projection_view("platform.pulse.status")
            .expect("collection fixture view is installed"),
        UiProjectionFieldRequirement::declared("identity.id").unwrap(),
        [UiProjectionFieldRequirement::declared("status").unwrap()],
        false,
        false,
    )
    .expect("identity-preserving collection registration");
    let projection = registration.view().identity().clone();
    let declaration = UiIntentDeclaration::<QueryTextIntent>::activate(DECLARATION)
        .unwrap()
        .bind_payload(
            QUERY_TEXT_FIELD,
            UiIntentPayloadSource::<UiIntentText>::projection(&projection),
        );
    let denial = match super::super::world::prepare::<QueryTextIntent>(
        routed_input(declaration, WorthUiIntentInteractionFamily::Activate),
        PayloadProjectionRegistration::Collection(registration),
        PayloadApplicationFacts::default(),
    ) {
        Ok(_) => panic!("a collection cannot enter a scalar-text payload declaration"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial,
        WorthUiApplicationPreparationDenial::IntentCatalog(Box::new(
            UiIntentCatalogPreparationDenial::UnknownPayloadProjection {
                declaration: DECLARATION.into(),
                field: QUERY_TEXT_FIELD.descriptor().stable_name().into(),
                projection: projection.as_str().into(),
                required_shape: "scalar-text",
            }
        ))
    );
}

fn query_world(
    registration: worth_ui_query_binding::UiScalarProjectionRegistration,
    projection: &worth_ui_query_binding::WorthUiQueryViewIdentity,
) -> super::super::world::PayloadWorld {
    let declaration = UiIntentDeclaration::<QueryTextIntent>::activate(DECLARATION)
        .unwrap()
        .bind_payload(
            QUERY_TEXT_FIELD,
            UiIntentPayloadSource::<UiIntentText>::projection(projection),
        );
    launch::<QueryTextIntent>(
        routed_input(declaration, WorthUiIntentInteractionFamily::Activate),
        PayloadProjectionRegistration::Scalar(registration),
        PayloadApplicationFacts::default(),
    )
}

fn prepare_stop(
    world: &mut super::super::world::PayloadWorld,
) -> worth_ui::facade::intent::UiIntentPayloadStop {
    let interaction = super::activation(world, [10, 20]);
    let route = super::product_route(&mut world.interaction, interaction);
    super::expect_payload_stop(
        world.interaction.session.prepare_intent_payload(route),
        "a non-current Query input must not seal a payload",
    )
}
