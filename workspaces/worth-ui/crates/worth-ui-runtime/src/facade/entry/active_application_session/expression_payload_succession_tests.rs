//! A prepared payload stays admissible only while every expression it read
//! holds the outcome revision it read, in the generation it was read in; a
//! successor whose payload source cannot fit its field is refused whole.

use super::*;

#[test]
fn a_recomputed_payload_expression_stops_admission_as_a_changed_input() {
    let mut world = payload_world(Label::Derived(LABEL));
    let changes: [(&str, fn(&mut PayloadWorld)); 3] = [
        ("derived text", |world| set_name(world, "beta")),
        ("condition", |world| set_switch(world, false)),
        ("derived integer", |world| set_tally(world, 5)),
    ];
    for (sequence, (source, change)) in (1u64..).step_by(2).zip(changes) {
        let proof = proof(&mut world, sequence);

        change(&mut world);

        let UiIntentAdmissionDecision::Stopped(stop) = admit(&mut world, proof) else {
            panic!("a payload over a recomputed {source} cannot admit");
        };
        assert_eq!(
            stop.reason(),
            &UiIntentAdmissionStopReason::PayloadInputChanged,
            "{source}"
        );
    }
    let _ = world.session.shutdown();
}

#[test]
fn an_expression_that_settles_unchanged_keeps_the_payload_current() {
    let mut world = payload_world(Label::Derived(LABEL));
    let proof = proof(&mut world, 1);
    let work = world.session.expression_work_counters();

    set_name(&mut world, "gamma");

    let followed = world.session.expression_work_counters();
    assert_eq!(
        followed.suppressed_unchanged,
        work.suppressed_unchanged + 1,
        "the label maps `gamma` to the `alpha` it already holds"
    );
    assert!(
        matches!(
            admit(&mut world, proof),
            UiIntentAdmissionDecision::Admitted(_)
        ),
        "an unchanged outcome revision keeps the payload current"
    );
    let _ = world.session.shutdown();
}

#[test]
fn a_payload_prepared_before_a_successor_is_refused_at_admission() {
    let mut world = payload_world(Label::Derived(LABEL));
    let proof = proof(&mut world, 1);
    let predecessor = world.session.active_generation_identity();

    let candidate = payload_submission(&world.session, Label::Derived(LABEL), "payload-follow");
    let mut turn = world.session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let observations = turn.seal().unwrap();
    let crate::runtime::observation::UiChangeClassificationOutcome::EvidenceOnly(evidence) =
        world.session.classify_observations(observations).unwrap()
    else {
        panic!("an unchanged source is evidence only");
    };
    let plan = world
        .session
        .compile_preservation_rebind(
            evidence,
            crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap();
    let prepared = world
        .session
        .prepare_rebind(
            plan,
            crate::runtime::rebind::UiRebindExecutionRequest::new(3),
        )
        .unwrap();
    if prepared.prepared_frame().is_some() {
        world.host.push_native_display_settled_without_effects();
    }
    assert!(matches!(
        prepared.execute(3),
        crate::runtime::rebind::UiRebindOutcome::Published(_)
    ));
    assert_ne!(world.session.active_generation_identity(), predecessor);

    let UiIntentAdmissionDecision::Stopped(stop) = admit(&mut world, proof) else {
        panic!("a payload of the retired generation cannot admit");
    };
    assert_eq!(
        stop.reason(),
        &UiIntentAdmissionStopReason::ApplicationGenerationChanged
    );
    let _ = world.session.shutdown();
}

#[test]
fn a_successor_whose_payload_source_cannot_fit_its_field_is_refused_whole() {
    let mut world = payload_world(Label::Derived(LABEL));
    let _ = proof(&mut world, 1);
    let predecessor = world.session.active_generation_identity();

    let candidate = payload_submission(&world.session, Label::Condition, "payload-mismatch");
    let mut turn = world.session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let observations = turn.seal().unwrap();
    let Err(crate::runtime::observation::UiChangeClassificationDenial::SourcePreparation(denial)) =
        world.session.classify_observations(observations)
    else {
        panic!("a label that reads a condition cannot prepare");
    };
    let crate::facade::lifecycle::WorthUiApplicationPreparationDenial::IntentCatalog(denial) =
        *denial
    else {
        panic!("the intent catalog refuses the payload source: {denial:?}");
    };
    assert_eq!(
        *denial,
        crate::declaration::UiIntentCatalogPreparationDenial::PayloadExpressionKindMismatch {
            declaration: fixture::ROUTE.into(),
            field: "label".into(),
            expression: ENABLED.into(),
            field_kind: crate::capability::UiIntentPayloadFieldKind::Text,
            role: worth_ui_dsl::WorthUiExpressionRole::Condition,
        }
    );

    assert_eq!(world.session.active_generation_identity(), predecessor);
    let proof = proof(&mut world, 3);
    assert_eq!(projected(), Some(("alpha".into(), true, 1)));
    assert!(
        matches!(
            admit(&mut world, proof),
            UiIntentAdmissionDecision::Admitted(_)
        ),
        "the prior generation stays installed and operable"
    );
    let _ = world.session.shutdown();
}
