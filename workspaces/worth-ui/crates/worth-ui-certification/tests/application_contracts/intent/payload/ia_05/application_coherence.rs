use worth_ui::facade::intent::{
    UiIntentApplicationFact, UiIntentBoolean, UiIntentDeclaration, UiIntentInputOwnerRevision,
    UiIntentPayloadProjectionViolation, UiIntentPayloadSource, UiIntentPayloadStop, UiIntentText,
    UiIntentUnsigned64,
};
use worth_ui_dsl::WorthUiIntentInteractionFamily;

use super::super::payload_types::{
    ApplicationIntent, BudgetTextIntent, APPLICATION_BOOLEAN_FIELD, APPLICATION_TEXT_FIELD,
    APPLICATION_UNSIGNED_FIELD, BUDGET_TEXT_FIELD,
};
use super::super::world::{
    launch, routed_input, PayloadApplicationFacts, PayloadProjectionRegistration, DECLARATION,
};

#[test]
fn ia_05_unicode_byte_boundaries_and_application_revisions_are_exact() {
    let fact = UiIntentApplicationFact::<UiIntentText>::text("phase3.fact.bounded-text", 8)
        .expect("the application contract admits the wider source budget");
    let declaration = UiIntentDeclaration::<BudgetTextIntent>::activate(DECLARATION)
        .unwrap()
        .bind_payload(
            BUDGET_TEXT_FIELD,
            UiIntentPayloadSource::<UiIntentText>::application_fact(&fact),
        );
    let mut world = launch::<BudgetTextIntent>(
        routed_input(declaration, WorthUiIntentInteractionFamily::Activate),
        PayloadProjectionRegistration::None,
        PayloadApplicationFacts::text(fact.clone(), ""),
    );

    let empty = prepare(&mut world);
    assert_eq!(empty.input_basis().cost().admitted_utf8_bytes(), 0);
    assert_revision(&empty, 1);

    let exact_update = world
        .interaction
        .session
        .update_intent_text_fact(&fact, "éé")
        .expect("four UTF-8 bytes fit both owner and payload budgets");
    assert_eq!(exact_update.revision(), 2);
    let exact = prepare(&mut world);
    assert_eq!(exact.input_basis().cost().admitted_utf8_bytes(), 4);
    assert_revision(&exact, 2);
    assert_revision(&empty, 1);

    let over_update = world
        .interaction
        .session
        .update_intent_text_fact(&fact, "ééx")
        .expect("five bytes remain valid at the application-fact owner");
    assert_eq!(over_update.revision(), 3);
    let route = route(&mut world);
    assert_eq!(
        super::expect_payload_stop(
            world.interaction.session.prepare_intent_payload(route),
            "the narrower payload field must reject five UTF-8 bytes",
        ),
        UiIntentPayloadStop::TextByteBudgetExceeded {
            field: BUDGET_TEXT_FIELD.descriptor().stable_name(),
            observed: 5,
            maximum: 4,
        }
    );
    assert_revision(&exact, 2);
    drop(exact);
    drop(empty);
    let _ = world.interaction.session.shutdown();
}

#[test]
fn ia_05_malformed_typed_projector_stops_before_a_payload_is_sealed() {
    let text = UiIntentApplicationFact::<UiIntentText>::text("phase3.fact.message", 32).unwrap();
    let boolean =
        UiIntentApplicationFact::<UiIntentBoolean>::boolean("phase3.fact.allowed").unwrap();
    let unsigned =
        UiIntentApplicationFact::<UiIntentUnsigned64>::unsigned64("phase3.fact.revision").unwrap();
    let declaration = UiIntentDeclaration::<ApplicationIntent>::activate(DECLARATION)
        .unwrap()
        .bind_payload(
            APPLICATION_TEXT_FIELD,
            UiIntentPayloadSource::<UiIntentText>::application_fact(&text),
        )
        .bind_payload(
            APPLICATION_BOOLEAN_FIELD,
            UiIntentPayloadSource::<UiIntentBoolean>::application_fact(&boolean),
        )
        .bind_payload(
            APPLICATION_UNSIGNED_FIELD,
            UiIntentPayloadSource::<UiIntentUnsigned64>::application_fact(&unsigned),
        );
    let mut world = launch::<ApplicationIntent>(
        routed_input(declaration, WorthUiIntentInteractionFamily::Activate),
        PayloadProjectionRegistration::None,
        PayloadApplicationFacts::standard(text, boolean, unsigned.clone()),
    );
    world
        .interaction
        .session
        .update_intent_unsigned64_fact(&unsigned, 43)
        .expect("the application owner admits a new typed value");

    let malformed_route = route(&mut world);
    assert_eq!(
        super::expect_payload_stop(
            world
                .interaction
                .session
                .prepare_intent_payload(malformed_route),
            "the concrete payload projector must reject its malformed value",
        ),
        UiIntentPayloadStop::PayloadProjection(
            UiIntentPayloadProjectionViolation::MalformedField {
                slot: APPLICATION_UNSIGNED_FIELD.descriptor().slot(),
            },
        )
    );
    let _ = world.interaction.session.shutdown();
}

fn prepare(
    world: &mut super::super::world::PayloadWorld,
) -> worth_ui::facade::intent::UiPreparedIntentPayload {
    let route = route(world);
    world
        .interaction
        .session
        .prepare_intent_payload(route)
        .expect("the declared application text projects")
}

fn route(
    world: &mut super::super::world::PayloadWorld,
) -> worth_ui::facade::intent::UiResolvedProductIntentRoute {
    let interaction = super::activation(world, [10, 20]);
    super::product_route(&mut world.interaction, interaction)
}

fn assert_revision(prepared: &worth_ui::facade::intent::UiPreparedIntentPayload, expected: u64) {
    let [UiIntentInputOwnerRevision::Application(revision)] =
        prepared.input_basis().owner_revisions()
    else {
        panic!("the bounded text payload retains exactly one application owner")
    };
    assert_eq!(revision.identity(), "phase3.fact.bounded-text");
    assert_eq!(revision.revision(), expected);
}
