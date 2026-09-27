use worth_ui::facade::intent::{
    UiIntentDeclaration, UiIntentPayloadSource, UiIntentPayloadStop, UiIntentText,
};
use worth_ui_dsl::WorthUiIntentInteractionFamily;
use worth_ui_runtime::facade::mounted::{
    UiHostSurfaceCancellationOutcome, UiMountedFrameOutcome, UiPresentationDeadline,
};
use worth_ui_test_support::{
    WorthUiMountedInteractionLifecycleCertificationExt, WorthUiMountedPublicationCertificationExt,
};

use super::super::payload_types::{QueryTextIntent, QUERY_TEXT_FIELD};
use super::super::world::{
    launch, launch_with_host, routed_input, PayloadApplicationFacts, PayloadProjectionRegistration,
};
use crate::mounted_host_protocol::scripted_host::presented_completion;

#[test]
fn ia_05_foreign_world_and_retired_target_routes_stop_before_projection() {
    assert_foreign_world_stops();
    assert_retired_target_stops();
}

#[test]
fn ia_05_payload_assembly_stops_while_publication_is_transitioning() {
    let host = crate::host_measurement_fixture::measurement_host();
    host.push_presented();
    let mut world = launch_with_host::<QueryTextIntent, _>(
        constant_route_input(),
        PayloadProjectionRegistration::None,
        PayloadApplicationFacts::default(),
        host.clone(),
    );
    let route = current_route(&mut world);
    let frame = crate::mounted_application_lifecycle::in_flight_presentation_world::prepared(
        &mut world.interaction.session,
    );
    host.push_in_flight(
        vec![presented_completion()],
        UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun,
    );
    let UiMountedFrameOutcome::InFlight(in_flight) = world
        .interaction
        .session
        .present_prepared_mounted_frame(frame, UiPresentationDeadline::at_tick(10), 0)
    else {
        panic!("the production host must retain one active presentation")
    };

    assert_eq!(
        super::expect_payload_stop(
            world.interaction.session.prepare_intent_payload(route),
            "payload assembly must not overlap publication",
        ),
        UiIntentPayloadStop::PublicationTransitionInFlight,
    );
    assert!(matches!(
        world
            .interaction
            .session
            .complete_mounted_presentation(in_flight, 1),
        UiMountedFrameOutcome::Published(_)
    ));
    let _ = world.interaction.session.shutdown();
}

fn assert_foreign_world_stops() {
    let mut source = launch::<QueryTextIntent>(
        constant_route_input(),
        PayloadProjectionRegistration::None,
        PayloadApplicationFacts::default(),
    );
    let mut foreign = launch::<QueryTextIntent>(
        constant_route_input(),
        PayloadProjectionRegistration::None,
        PayloadApplicationFacts::default(),
    );
    let route = current_route(&mut source);
    assert_eq!(
        super::expect_payload_stop(
            foreign.interaction.session.prepare_intent_payload(route),
            "a foreign application generation must not consume the route",
        ),
        UiIntentPayloadStop::ApplicationGenerationChanged,
    );
    let _ = source.interaction.session.shutdown();
    let _ = foreign.interaction.session.shutdown();
}

fn assert_retired_target_stops() {
    let mut world = launch::<QueryTextIntent>(
        constant_route_input(),
        PayloadProjectionRegistration::None,
        PayloadApplicationFacts::default(),
    );
    let interaction = super::activation(&mut world, [10, 20]);
    let mounted = interaction.target().mounted_instance();
    let route = super::product_route(&mut world.interaction, interaction);
    world
        .interaction
        .session
        .unmount_instance_with_interaction_receipt(mounted)
        .expect("the exact mounted incarnation retires");
    assert_eq!(
        super::expect_payload_stop(
            world
                .interaction
                .session
                .prepare_intent_payload(route),
            "a retired target must not enter payload projection",
        ),
        UiIntentPayloadStop::Targeting(
            worth_ui::facade::interaction::UiInteractionTargetingDenial::
                MountedInstanceNoLongerCurrent,
        ),
    );
    let _ = world.interaction.session.shutdown();
}

fn constant_route_input() -> worth_ui_dsl::WorthUiRustAuthoredArtifactInput {
    let declaration =
        UiIntentDeclaration::<QueryTextIntent>::activate(super::super::world::DECLARATION)
            .unwrap()
            .bind_payload(
                QUERY_TEXT_FIELD,
                UiIntentPayloadSource::<UiIntentText>::constant("query-current"),
            );
    routed_input(declaration, WorthUiIntentInteractionFamily::Activate)
}

fn current_route(
    world: &mut super::super::world::PayloadWorld,
) -> worth_ui::facade::intent::UiResolvedProductIntentRoute {
    let interaction = super::activation(world, [10, 20]);
    super::product_route(&mut world.interaction, interaction)
}
