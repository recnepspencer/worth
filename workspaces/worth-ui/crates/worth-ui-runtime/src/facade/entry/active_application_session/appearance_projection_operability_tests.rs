use crate::facade::intent::*;
use worth_ui_host_contract::*;

#[path = "appearance_projection_operability_fixture.rs"]
mod fixture;

#[path = "appearance_projection_operability_lifecycle_tests.rs"]
mod lifecycle_tests;

#[path = "intent_operability_observation_tests.rs"]
mod observation_tests;

#[path = "pointer_confirmation_tests.rs"]
mod pointer_confirmation_tests;
#[path = "pointer_affordance_projection_tests.rs"]
mod pointer_projection_tests;

#[path = "appearance_projection_operability_output.rs"]
mod output;
use output::project;

#[test]
fn operability_evaluation_selects_only_its_mounted_neighborhood() {
    let role = fixture::role();
    let (mut session, host) = fixture::session(&role, 1);
    let (surface, graph) = super::mounting_fixture::mount(&mut session, 1_000);
    let node = session.mounted_graph_node(graph).unwrap();
    let mut surfaces = vec![surface];
    for _ in 0..2 {
        let surface = session.create_semantic_surface().unwrap();
        session
            .register_host_surface(
                surface,
                crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
                crate::facade::mounted::UiSurfaceBindingProfile::new(
                    1_000,
                    crate::facade::mounted::UiSurfaceBindingCoordinatePosture::LogicalPoints,
                    1,
                )
                .unwrap(),
            )
            .unwrap();
        session.mount_instance(node, surface).unwrap();
        crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(
            &mut session,
            surface,
        );
        surfaces.push(surface);
    }
    close(&mut session, &role, 1, "operability-initial");
    session.advance_mounted_identity_frame().unwrap();
    let frame = prepare(&mut session);
    assert_eq!(
        frame
            .appearance_selection_cost_report()
            .selected_instance_count(),
        3
    );
    publish(&mut session, &host, frame, 1);
    assert_eq!(
        session
            .intent_admission
            .operability_standing_snapshot()
            .unwrap()
            .facts()
            .len(),
        0,
        "mounting cannot manufacture an operability decision"
    );

    let (target, decision) = activate(&mut session, surfaces[0], 1);
    assert_eq!(decision.primary_cause(), None);
    let first = session
        .intent_admission
        .operability_standing_snapshot()
        .unwrap();
    assert_eq!(
        first
            .fact_for(graph, target, fixture::ROUTE)
            .unwrap()
            .decision(),
        &decision
    );
    let (_, repeated) = activate(&mut session, surfaces[0], 3);
    assert_eq!(repeated, decision);
    assert_eq!(
        session
            .intent_admission
            .operability_standing_snapshot()
            .unwrap(),
        first,
        "identical evaluation on the same receipt must not advance owner evidence"
    );
    close(&mut session, &role, 1, "operability-ready");
    let frame = project(
        &mut session,
        &[(target, 10)],
        &[(
            surfaces[0],
            Some((target, UiPointerAffordanceFamily::Activation)),
        )],
    );
    publish(&mut session, &host, frame, 2);

    session
        .update_intent_boolean_fact(&fixture::fact(fixture::MUTABLE), false)
        .unwrap();
    let (_, denied) = activate(&mut session, surfaces[0], 5);
    assert_eq!(
        denied.primary_cause(),
        Some(UiIntentInoperableCause::Readonly)
    );
    close(&mut session, &role, 1, "operability-readonly");
    let frame = project(
        &mut session,
        &[(target, 20)],
        &[(
            surfaces[0],
            Some((target, UiPointerAffordanceFamily::Default)),
        )],
    );
    publish(&mut session, &host, frame, 3);
    let previous = session
        .intent_admission
        .operability_standing_snapshot()
        .unwrap();

    // Same appearance class, different complete owner decision must retain its evidence.
    session
        .update_intent_boolean_fact(&fixture::fact(fixture::POLICY), false)
        .unwrap();
    let (_, policy) = activate(&mut session, surfaces[0], 7);
    let next = session
        .intent_admission
        .operability_standing_snapshot()
        .unwrap();
    assert_ne!(policy, denied);
    assert_eq!(next.changed_instances(&previous).as_ref(), &[target]);
    assert_eq!(
        next.fact_for(graph, target, fixture::ROUTE)
            .unwrap()
            .decision(),
        &policy
    );
    assert_eq!(
        previous
            .fact_for(graph, target, fixture::ROUTE)
            .unwrap()
            .decision(),
        &denied
    );
    close(&mut session, &role, 1, "operability-policy");
    let frame = project(&mut session, &[(target, 20)], &[]);
    publish(&mut session, &host, frame, 4);
    let query = worth_ui_inspection::UiAppearanceInspectionQuery::new(
        session.appearance_inspection_world(surfaces[0]),
        graph.digest(),
        worth_ui_dsl::UiAppearanceAspect::Background,
    );
    let worth_ui_inspection::UiAppearanceInspectionOutcome::Found(explanation) =
        session.why_appearance(query)
    else {
        panic!("changed decision must be inspectable");
    };
    assert!(explanation.input_evidence_changed());
    assert!(!explanation.resolved_aspect_value_changed());
    assert!(!explanation.mounted_mechanical_output_changed());
    close(&mut session, &role, 1, "operability-no-change");
    let frame = prepare(&mut session);
    assert_eq!(
        frame
            .appearance_selection_cost_report()
            .selected_instance_count(),
        0
    );
    drop(frame);

    let (peer, _) = activate(&mut session, surfaces[1], 9);
    assert_ne!(peer, target);
    close(&mut session, &role, 1, "operability-peer");
    let frame = project(
        &mut session,
        &[(peer, 20)],
        &[
            (surfaces[0], None),
            (
                surfaces[1],
                Some((peer, UiPointerAffordanceFamily::Default)),
            ),
        ],
    );
    publish(&mut session, &host, frame, 5);
    session.unmount_instance(target).unwrap();
    let retired = session
        .intent_admission
        .operability_standing_snapshot()
        .unwrap();
    assert!(retired.fact_for(graph, target, fixture::ROUTE).is_none());
    assert!(retired.fact_for(graph, peer, fixture::ROUTE).is_some());
    assert!(first.fact_for(graph, target, fixture::ROUTE).is_some());
    let _ = session.shutdown();
}

#[test]
fn operability_missing_and_ambiguous_declared_routes_remain_distinct() {
    use worth_ui_inspection::UiAppearanceInspectionDenialPosture as Denial;
    for (routes, expected) in [
        (0, Denial::MissingOperabilityRoute),
        (2, Denial::AmbiguousOperabilityRoute { routes: 2 }),
    ] {
        let role = fixture::role();
        let (mut session, host) = fixture::session(&role, routes);
        let (surface, graph) = super::mounting_fixture::mount(&mut session, 1_000);
        close(&mut session, &role, routes, "operability-route-denial");
        session.advance_mounted_identity_frame().unwrap();
        let frame = prepare(&mut session);
        publish(&mut session, &host, frame, 1);
        let query = worth_ui_inspection::UiAppearanceInspectionQuery::new(
            session.appearance_inspection_world(surface),
            graph.digest(),
            worth_ui_dsl::UiAppearanceAspect::Background,
        );
        let worth_ui_inspection::UiAppearanceInspectionOutcome::Found(explanation) =
            session.why_appearance(query)
        else {
            panic!("route denial must be inspectable");
        };
        assert_eq!(explanation.denial_posture(), Some(expected));
        assert!(explanation.denied_before_effects());
        let _ = session.shutdown();
    }
}

fn close(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    routes: usize,
    name: &str,
) {
    let source = fixture::candidate(session, role, routes, name);
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(source).unwrap();
    let admitted = turn.seal().unwrap();
    session.classify_observations(admitted).unwrap();
}

fn activate(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
    sequence: u64,
) -> (UiMountedInstanceIdentity, UiIntentOperabilityDecision) {
    let presentation = session
        .mounted
        .current_publication()
        .unwrap()
        .presentation_for_surface(surface)
        .unwrap();
    let hit = session
        .mounted
        .interaction_hit_test_basis(presentation)
        .unwrap();
    assert_eq!(hit.rows().len(), 1);
    let row = hit.rows()[0];
    let target = row.mounted_instance();
    let bounds = row.bounds();
    let position = UiHostSurfacePosition::viewport_logical(
        ((bounds.x() + bounds.width() / 2.0) * 1_000.0) as i64,
        ((bounds.y() + bounds.height() / 2.0) * 1_000.0) as i64,
    );
    let mut interaction = None;
    for (offset, transition) in [
        UiHostPointerButtonTransition::Pressed,
        UiHostPointerButtonTransition::Released,
    ]
    .into_iter()
    .enumerate()
    {
        let batch = super::pointer_tests::pointer_batch(
            session.host_session.identity().as_u64(),
            presentation,
            sequence + offset as u64,
            UiHostPointerIdentity::new(1),
            position,
            Some(transition),
            offset == 0,
        );
        let crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) =
            session.admit_host_interaction_batch(batch)
        else {
            panic!("real pointer ingress must admit");
        };
        for transition in receipt.into_transitions().into_vec() {
            if let crate::facade::interaction::UiInteractionTransition::Semantic(value) = transition
            {
                interaction = Some(value);
            }
        }
    }
    let interaction = interaction.expect("press/release must produce a real activation");
    assert_eq!(interaction.target().mounted_instance(), target);
    let route = session
        .resolve_intent_route(
            crate::facade::interaction::UiIntentRouteSource::mounted_interaction(interaction),
        )
        .unwrap();
    let UiIntentRouteResolution::Product(route) = route else {
        panic!("authored product route");
    };
    let candidate = session.prepare_intent_payload(route).unwrap();
    let decision = match session.evaluate_intent_operability(candidate) {
        UiIntentOperabilityOutcome::Operable(proof) => proof.decision().clone(),
        UiIntentOperabilityOutcome::Inoperable(candidate) => candidate.decision().clone(),
    };
    (target, decision)
}

fn prepare(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
) -> crate::mounting::UiPreparedMountedFrame {
    session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|denial| {
            use crate::facade::entry::WorthUiMountedFrameExecutionStop as Stop;
            match denial {
                Stop::Preparation(denial) => panic!("operability preparation: {denial:?}"),
                Stop::PublicationLease(denial) => panic!("operability lease: {denial:?}"),
                Stop::HostMeasurement(denial) => panic!("operability measurement: {denial:?}"),
                Stop::HostMeasurementTransition(denial) => {
                    panic!("operability measurement transition: {denial:?}")
                }
                Stop::OccurrenceGeometry(denial) => {
                    panic!("operability occurrence geometry: {denial:?}")
                }
                Stop::FrameworkTransition(_) => panic!("operability framework transition"),
            }
        })
}

fn publish(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    frame: crate::mounting::UiPreparedMountedFrame,
    now: u64,
) {
    for _ in frame.surfaces() {
        host.push_native_display_settled_without_effects();
    }
    let outcome = session.present_prepared_mounted_frame_internal(
        frame,
        UiPresentationDeadline::at_tick(100),
        now,
    );
    match &outcome {
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => {
            panic!(
                "operability publication rejected at {now}: {:?}",
                rejected.rejections()
            );
        }
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(rejected) => {
            panic!("operability admission denied at {now}: {rejected:?}");
        }
        crate::mounting::UiMountedFrameOutcome::CompletionDenied(denial) => {
            panic!("operability completion denied at {now}: {denial:?}");
        }
        _ => {}
    }
    assert!(
        matches!(
            outcome,
            crate::mounting::UiMountedFrameOutcome::Published(_)
                | crate::mounting::UiMountedFrameOutcome::Unchanged(_)
        ),
        "operability publication failed at {now}"
    );
}
