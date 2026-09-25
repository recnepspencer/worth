use worth_runtime_bridge::facade::BridgeMixedCauseOrderingInput;
use worth_signal::facade::NodeId;
use worth_ui::facade::intent::{
    UiIntentApplicationFact, UiIntentBoolean, UiIntentDeclaration, UiIntentInputOwnerRevision,
    UiIntentPayloadSource, UiIntentRouteResolution, UiIntentRouteSource, UiIntentText,
    UiIntentUnsigned64,
};
use worth_ui::facade::interaction::{
    UiHostInteractionIngressOutcome, UiInteractionTransition, UiSemanticInteraction,
};
use worth_ui::facade::observation::UiChangeClassificationOutcome;
use worth_ui::facade::observation_report::UiHostPointerButtonTransition;
use worth_ui::facade::rebind::{
    UiRebindExecutionPolicy, UiRebindExecutionRequest, UiRebindOutcome,
};
use worth_ui_dsl::WorthUiIntentInteractionFamily;
use worth_ui_query_binding::{
    UiProjectionFieldRequirement, UiProjectionObservation, UiScalarProjectionRegistration,
    WorthUiQueryWorkspaceExt,
};

use super::payload_types::{
    ApplicationIntent, QueryTextIntent, WideIntent, APPLICATION_BOOLEAN_FIELD,
    APPLICATION_TEXT_FIELD, APPLICATION_UNSIGNED_FIELD, QUERY_TEXT_FIELD, WIDE_FIELDS,
};
use super::world::{
    launch, routed_input, PayloadApplicationFacts, PayloadProjectionRegistration, PayloadWorld,
    DECLARATION,
};
use crate::projection_lifecycle::support::ScalarLifecycleWorld;

mod focus_reveal_frame;
mod selection_identity;
mod selection_portal_coordination;

#[test]
fn ia_05_zero_one_and_sixty_four_fields_follow_declared_width_not_world_width() {
    assert_zero_field_file_world();
    assert_one_query_field();
    assert_sixty_four_constant_fields();
}

#[test]
fn ia_05_application_facts_seal_exact_owner_revisions() {
    let text = UiIntentApplicationFact::<UiIntentText>::text("phase3.fact.message", 32).unwrap();
    let boolean = UiIntentApplicationFact::<UiIntentBoolean>::boolean("phase3.fact.allowed")
        .expect("boolean fact identity");
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
        PayloadApplicationFacts::standard(text.clone(), boolean.clone(), unsigned.clone()),
    );
    for (identity, receipt) in [
        (
            "phase3.fact.message",
            world
                .interaction
                .session
                .update_intent_text_fact(&text, "application-current")
                .expect("registered text fact updates"),
        ),
        (
            "phase3.fact.allowed",
            world
                .interaction
                .session
                .update_intent_boolean_fact(&boolean, true)
                .expect("registered boolean fact updates"),
        ),
        (
            "phase3.fact.revision",
            world
                .interaction
                .session
                .update_intent_unsigned64_fact(&unsigned, 42)
                .expect("registered unsigned fact updates"),
        ),
    ] {
        assert_eq!(receipt.identity(), identity);
        assert_eq!(receipt.revision(), 2);
    }
    let interaction = activation(&mut world, [10, 20]);
    let route = product_route(&mut world.interaction, interaction);
    let prepared = world
        .interaction
        .session
        .prepare_intent_payload(route)
        .expect("three exact application facts project");
    let cost = prepared.input_basis().cost();
    assert_eq!(cost.declared_fields(), 3);
    assert_eq!(cost.application_inputs_read(), 3);
    assert_eq!(cost.query_inputs_read(), 0);
    assert_eq!(cost.admitted_utf8_bytes(), "application-current".len());
    assert_eq!(prepared.retained_owner_reference_count(), 4);
    assert_eq!(prepared.input_basis().owner_revisions().len(), 3);
    for (expected, observed) in [
        ("phase3.fact.message", 2),
        ("phase3.fact.allowed", 2),
        ("phase3.fact.revision", 2),
    ]
    .into_iter()
    .zip(prepared.input_basis().owner_revisions())
    {
        let UiIntentInputOwnerRevision::Application(revision) = observed else {
            panic!("application sources retain application revisions")
        };
        assert_eq!(revision.identity(), expected.0);
        assert_eq!(revision.revision(), expected.1);
    }
    drop(prepared);
    let _ = world.interaction.session.shutdown();
}

fn assert_zero_field_file_world() {
    let mut world = super::super::declaration::support::file_world();
    let interaction = activation_interaction(&mut world, [20, 20]);
    let route = product_route(&mut world, interaction);
    let prepared = world
        .session
        .prepare_intent_payload(route)
        .expect("the real filesystem empty payload seals");
    assert_eq!(prepared.input_basis().cost().declared_fields(), 0);
    assert_eq!(prepared.input_basis().owner_revisions(), &[]);
    assert_eq!(prepared.retained_owner_reference_count(), 1);
    assert_eq!(prepared.retained_payload_count(), 1);
    drop(prepared);
    let _ = world.session.shutdown();
}

fn assert_one_query_field() {
    let (mut query, completion) =
        ScalarLifecycleWorld::standard(NodeId::new(31405, 0), "query-current");
    let registration = scalar_registration(&query);
    let projection = registration.view().identity().clone();
    let declaration = UiIntentDeclaration::<QueryTextIntent>::activate(DECLARATION)
        .unwrap()
        .bind_payload(
            QUERY_TEXT_FIELD,
            UiIntentPayloadSource::<UiIntentText>::projection(&projection),
        );
    let mut world = launch::<QueryTextIntent>(
        routed_input(declaration, WorthUiIntentInteractionFamily::Activate),
        PayloadProjectionRegistration::Scalar(registration),
        PayloadApplicationFacts::default(),
    );
    let pending = query.initial().into_fact_and_predecessor().0;
    let current = query
        .advance(
            BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
            Some(pending),
        )
        .into_fact_and_predecessor()
        .0;
    publish_scalar(&mut world, current.into_observation(), 314_050);
    world.interaction.publish_successor();

    let interaction = activation(&mut world, [10, 20]);
    let route = product_route(&mut world.interaction, interaction);
    let prepared = world
        .interaction
        .session
        .prepare_intent_payload(route)
        .expect("intent-only Query projection reaches the payload basis");
    let cost = prepared.input_basis().cost();
    assert_eq!(cost.declared_fields(), 1);
    assert_eq!(cost.query_inputs_read(), 1);
    assert_eq!(cost.application_inputs_read(), 0);
    assert_eq!(cost.admitted_utf8_bytes(), "query-current".len());
    assert_eq!(prepared.retained_owner_reference_count(), 2);
    let [UiIntentInputOwnerRevision::Query(revision)] = prepared.input_basis().owner_revisions()
    else {
        panic!("the exact Query owner revision is sealed")
    };
    assert_eq!(revision.field(), QUERY_TEXT_FIELD.descriptor());
    assert_eq!(revision.revision().projection_identity(), &projection);
    drop(prepared);
    let _ = world.interaction.session.shutdown();
}

fn assert_sixty_four_constant_fields() {
    let mut declaration = UiIntentDeclaration::<WideIntent>::activate(DECLARATION).unwrap();
    for (index, field) in WIDE_FIELDS.into_iter().enumerate() {
        declaration = declaration.bind_payload(
            field,
            UiIntentPayloadSource::<UiIntentUnsigned64>::constant(index as u64),
        );
    }
    let mut world = launch::<WideIntent>(
        routed_input(declaration, WorthUiIntentInteractionFamily::Activate),
        PayloadProjectionRegistration::None,
        PayloadApplicationFacts::default(),
    );
    let interaction = activation(&mut world, [10, 20]);
    let route = product_route(&mut world.interaction, interaction);
    let prepared = world
        .interaction
        .session
        .prepare_intent_payload(route)
        .expect("the contractual maximum payload width projects");
    let cost = prepared.input_basis().cost();
    assert_eq!(cost.declared_fields(), 64);
    assert_eq!(cost.query_inputs_read(), 0);
    assert_eq!(cost.application_inputs_read(), 0);
    assert_eq!(cost.admitted_utf8_bytes(), 0);
    assert_eq!(prepared.input_basis().owner_revisions(), &[]);
    drop(prepared);
    let _ = world.interaction.session.shutdown();
}

fn scalar_registration(world: &ScalarLifecycleWorld) -> UiScalarProjectionRegistration {
    let domain = world
        .workspace
        .worth_ui()
        .expect("Worth UI domain installed");
    UiScalarProjectionRegistration::text(
        domain
            .projection_view("platform.pulse.status")
            .expect("fixture projection is installed"),
        UiProjectionFieldRequirement::declared("status").unwrap(),
    )
}

fn publish_scalar(
    world: &mut PayloadWorld,
    observation: worth_ui_query_binding::UiScalarProjectionObservation,
    request: u64,
) {
    publish_projection(world, UiProjectionObservation::Scalar(observation), request);
}

fn publish_projection(
    world: &mut PayloadWorld,
    observation: UiProjectionObservation,
    request: u64,
) {
    let mut turn = world.interaction.session.begin_observation_turn().unwrap();
    turn.admit_projection_query(observation).unwrap();
    let admitted = turn.seal().unwrap();
    let changed = match world
        .interaction
        .session
        .classify_observations(admitted)
        .unwrap()
    {
        UiChangeClassificationOutcome::Changed(changed) => changed,
        UiChangeClassificationOutcome::ObservedNoChange(_) => {
            panic!("intent-only Query fact cannot classify as no-change")
        }
        UiChangeClassificationOutcome::EvidenceOnly(_) => {
            panic!("intent-only Query fact must retain payload semantics")
        }
    };
    let lifecycle = world
        .interaction
        .session
        .resolve_affected_scope(changed)
        .unwrap()
        .resolve_identity_lifecycle()
        .unwrap();
    let plan = world
        .interaction
        .session
        .compile_rebind_plan(lifecycle, UiRebindExecutionPolicy::ordinary())
        .unwrap();
    let prepared = world
        .interaction
        .session
        .prepare_rebind(plan, UiRebindExecutionRequest::new(request))
        .expect("intent-only Query fact prepares a mounted successor");
    assert!(matches!(
        prepared.execute(request),
        UiRebindOutcome::Published(_)
    ));
}

fn activation(world: &mut PayloadWorld, point: [i64; 2]) -> UiSemanticInteraction {
    activation_interaction(&mut world.interaction, point)
}

fn activation_interaction(
    world: &mut super::super::interaction_world::InteractionWorld,
    point: [i64; 2],
) -> UiSemanticInteraction {
    let _ = world.button(1, 1, UiHostPointerButtonTransition::Pressed, point);
    let released = world.button(1, 1, UiHostPointerButtonTransition::Released, point);
    let UiHostInteractionIngressOutcome::Applied(receipt) = released else {
        panic!("release reaches the production interaction compiler")
    };
    receipt
        .into_transitions()
        .into_vec()
        .into_iter()
        .find_map(|transition| match transition {
            UiInteractionTransition::Semantic(interaction) => Some(interaction),
            _ => None,
        })
        .expect("one press/release pair mints one semantic activation")
}

fn product_route(
    world: &mut super::super::interaction_world::InteractionWorld,
    interaction: UiSemanticInteraction,
) -> worth_ui::facade::intent::UiResolvedProductIntentRoute {
    match world
        .session
        .resolve_intent_route(UiIntentRouteSource::mounted_interaction(interaction))
        .expect("current mounted interaction resolves its declared route")
    {
        UiIntentRouteResolution::Product(route) => route,
        UiIntentRouteResolution::Confirmation(_) => {
            panic!("IA-05 payload route cannot cross into confirmation")
        }
    }
}
